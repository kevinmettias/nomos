//! The one place this crate decides where code ends and a `//` comment begins, aware of
//! the literals a bare substring split cannot tell from the code they interrupt.
//!
//! `line.split("//").next().unwrap_or(line)` was written eleven times across this crate's
//! own syntax-shaped text rules, and all eleven shared the identical defect
//! `P45-CODE-PREFIX-KNOWS-STRINGS` measured against the built binary: a line whose string
//! literal holds a URL (`"https://example.com"`) is truncated at the scheme's own double
//! slash, and everything after it — a real `unsafe` block, a real atomic ordering, a real
//! anything — becomes invisible to every rule that copied this line.
//!
//! This function is a real, minimal scanner rather than another substring split: it walks
//! `line` byte by byte, tracks whether the cursor sits inside a double-quoted string, a raw
//! string (`r"..."`, `r#"..."#`, ..., including the `b`-prefixed byte-string forms), or a
//! char literal, and only reads a `//` as a comment start when none of those are open.
//!
//! # What this does not do, on purpose
//!
//! It does not blind a content-shaped rule to the text of a string. `security_text.rs`'s
//! own module doc already reasons through why: a rule whose entire subject is what a string
//! literal *contains* — a hardcoded credential, a secret in a URL — must keep reading that
//! text, and a shared helper that stripped every string before a rule ever saw it would
//! defeat exactly the rules that need to see inside one. This function answers a narrower
//! question, "where does an unquoted `//` comment begin", which a syntax-shaped rule needs
//! and a content-shaped rule does not call at all.
//!
//! It also does not blank a literal's own body once it has found one — an earlier version
//! of this function did, so that a syntax detector's bare `.contains("unsafe {")` could not
//! match a string that merely quotes the words, and that earlier version's own doc argued
//! for it at length. It broke real, legitimate rules the moment it was tried against this
//! crate's own suite: `A_Rust_Path_Stays_Within_Its_Own_Subtree` reads `#[path = "value"]`'s
//! own quoted value as the path being judged, `Unwrap_Expect_Discipline` reads
//! `.expect("message")`'s own quoted argument, and `A_Skipped_Test_States_Why` reads
//! `t.Skip("reason")`'s own quoted argument — three real, passing tests, each reading a
//! specific string literal's value as the very syntax being judged, not as arbitrary quoted
//! content a bare substring search could be fooled by. Blanking every string uniformly
//! cannot tell those two cases apart, and this crate's own "verify, do not assume"
//! discipline caught the regression before it shipped. A string's own text still reading as
//! an unrelated construct's keyword is a real, narrower gap this function does not close —
//! see this file's own history for the one thing that was tried and did not survive contact
//! with a real test suite.
//!
//! It does not resolve a multi-line raw string or a multi-line block comment. Every syntax-
//! shaped rule in this crate reads one line at a time with no cross-line state, and a raw
//! string that opens on one line and closes on another is already outside what a per-line
//! scanner can see — the same boundary `Previous_Comment_Block_Has` already accepts for a
//! `//` comment block spanning lines it walks explicitly rather than one this function
//! tracks silently. What this function adds is single-line correctness: a raw string that
//! opens and closes within one line is read correctly, unbalanced quote and all.

/// `line`, with any trailing `//` line comment removed — but a `//` only ends the line when
/// it appears outside a string, a raw string, or a char literal, unlike the plain
/// `line.split("//").next()` this replaces everywhere it was copied. See this file's own
/// module doc for why a literal's own body is copied through unchanged rather than blanked.
///
/// Returns an owned `String` rather than a borrowed slice of `line` so a future, narrower
/// fix (blanking only where a specific detector proves it safe) does not have to change
/// this function's signature again; every real call site already reads the result with a
/// `&str` method, which `String`'s own `Deref` already supports identically to a borrowed
/// slice.
#[must_use]
pub(crate) fn Code_Prefix(line: &str) -> String
{
    let bytes = line.as_bytes();
    let mut output: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while let Some(&byte) = bytes.get(index)
    {
        if let Some(hashes) = Raw_String_Opens_At(bytes, index)
        {
            let body_start = Raw_String_Body_Start(bytes, index, hashes);
            let end = Skip_Raw_String_Body(bytes, body_start, hashes);
            output.extend_from_slice(bytes.get(index..end).unwrap_or_default());
            index = end;
            continue;
        }

        match byte
        {
            b'/' if bytes.get(index.saturating_add(1)) == Some(&b'/') => break,
            b'"' =>
            {
                let end = Skip_String_Body(bytes, index.saturating_add(1));
                output.extend_from_slice(bytes.get(index..end).unwrap_or_default());
                index = end;
            }
            b'\'' if Opens_Char_Literal(bytes, index) =>
            {
                let end = Skip_Char_Literal_Body(bytes, index.saturating_add(1));
                output.extend_from_slice(bytes.get(index..end).unwrap_or_default());
                index = end;
            }
            other =>
            {
                output.push(other);
                index = index.saturating_add(1);
            }
        }
    }

    // Every byte pushed above is copied verbatim from `line`'s own valid UTF-8, so this can
    // never fail on real input -- the fallback exists only so this stays a total function
    // rather than one that could panic on a construction its own scanner cannot produce.
    return String::from_utf8(output).unwrap_or_default();
}

/// Advances past a double-quoted string's own body, honoring a backslash escape so an
/// escaped `"` does not end it early, and returns the index one past the closing `"` (or
/// `bytes.len()` if the string is unbalanced on this line — the same "read what is here,
/// do not guess past the line" discipline every other rule in this file already keeps).
fn Skip_String_Body(bytes: &[u8], mut index: usize) -> usize
{
    while let Some(&byte) = bytes.get(index)
    {
        match byte
        {
            b'\\' => index = index.saturating_add(2),
            b'"' => return index.saturating_add(1),
            _ => index = index.saturating_add(1),
        }
    }

    return index;
}

/// Whether `bytes[index]` (a `'`) opens a char literal rather than a lifetime. A lifetime
/// (`'a`, `'static`) has no matching closing `'`; a char literal does, at exactly the
/// position a one-character or an escaped literal would put it — `'"'` closes two bytes
/// later, `'\''` three. Checking for a real closing quote at that exact position, rather
/// than assuming every `'` opens a literal, is what keeps `fn F<'a>(x: &'a str)` from being
/// misread as an unterminated char literal that swallows the rest of the line.
fn Opens_Char_Literal(bytes: &[u8], index: usize) -> bool
{
    if bytes.get(index.saturating_add(1)) == Some(&b'\\')
    {
        return bytes.get(index.saturating_add(3)) == Some(&b'\'');
    }

    return bytes.get(index.saturating_add(1)).is_some() && bytes.get(index.saturating_add(2)) == Some(&b'\'');
}

/// Advances past a char literal's own body (already confirmed real by
/// [`Opens_Char_Literal`]), returning the index one past its closing `'`.
fn Skip_Char_Literal_Body(bytes: &[u8], mut index: usize) -> usize
{
    if bytes.get(index) == Some(&b'\\')
    {
        index = index.saturating_add(1);
    }
    index = index.saturating_add(1);

    return if bytes.get(index) == Some(&b'\'') { index.saturating_add(1) } else { index };
}

/// The number of `#` characters a raw string opening at `index` uses (`r"` is zero,
/// `r#"` is one, `r##"` is two, ...), covering the `b`-prefixed byte-string forms
/// (`br"..."`, `br#"..."#`) as well — `None` when `bytes[index..]` does not open a raw
/// string at all, so a caller can tell "not a raw string" apart from "a raw string with
/// zero hashes".
fn Raw_String_Opens_At(bytes: &[u8], index: usize) -> Option<usize>
{
    let mut cursor = index;
    if bytes.get(cursor) == Some(&b'b')
    {
        cursor = cursor.saturating_add(1);
    }
    if bytes.get(cursor) != Some(&b'r')
    {
        return None;
    }
    cursor = cursor.saturating_add(1);

    let mut hashes = 0usize;
    while bytes.get(cursor) == Some(&b'#')
    {
        hashes = hashes.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }

    return if bytes.get(cursor) == Some(&b'"') { Some(hashes) } else { None };
}

/// The index of the first byte of a raw string's own body — one past its opening
/// `r`/`br` prefix, `hashes` `#` characters, and its opening `"`.
fn Raw_String_Body_Start(bytes: &[u8], index: usize, hashes: usize) -> usize
{
    let mut cursor = index;
    if bytes.get(cursor) == Some(&b'b')
    {
        cursor = cursor.saturating_add(1);
    }
    cursor = cursor.saturating_add(1); // the `r`
    cursor = cursor.saturating_add(hashes);
    cursor = cursor.saturating_add(1); // the opening `"`

    return cursor;
}

/// Advances past a raw string's own body — no escape sequence to honor, since a raw string
/// has none — to one past the first `"` immediately followed by `hashes` `#` characters, or
/// to `bytes.len()` if the line ends first, the honest answer for the fixture this rule was
/// written against: a raw string holding both an unsafe block and an unbalanced quote,
/// which closes only when the real `"`-plus-hashes delimiter is found, never on the stray
/// quote alone.
fn Skip_Raw_String_Body(bytes: &[u8], mut index: usize, hashes: usize) -> usize
{
    while bytes.get(index).is_some()
    {
        if bytes.get(index) == Some(&b'"') && Raw_String_Closes_Here(bytes, index, hashes)
        {
            return index.saturating_add(1).saturating_add(hashes);
        }
        index = index.saturating_add(1);
    }

    return index;
}

/// Whether the closing delimiter of a `hashes`-hash raw string sits at `bytes[index]` (a
/// `"`): exactly `hashes` `#` characters immediately follow it.
fn Raw_String_Closes_Here(bytes: &[u8], index: usize, hashes: usize) -> bool
{
    let start = index.saturating_add(1);
    let end = start.saturating_add(hashes);

    return bytes.get(start..end).is_some_and(|slice| return slice.iter().all(|byte| return *byte == b'#'));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Code_Prefix_Should_Strip_A_Trailing_Comment()
    {
        assert_eq!(Code_Prefix("let x = 1; // a comment"), "let x = 1; ");
    }

    #[test]
    fn Test_Code_Prefix_Should_Return_The_Whole_Line_With_No_Comment()
    {
        assert_eq!(Code_Prefix("let x = 1;"), "let x = 1;");
    }

    /// The first real defect this rule exists to fix: a string literal holding a URL must
    /// not truncate the real code that follows it on the same line.
    #[test]
    fn Test_Code_Prefix_Should_Not_Truncate_At_A_Url_Inside_A_String()
    {
        let line = "let doc = \"https://example.com\"; unsafe { core::ptr::read(p) }";

        let prefix = Code_Prefix(line);

        assert!(
            prefix.contains("unsafe { core::ptr::read(p) }"),
            "the string's own // must not hide the real code after it: {prefix:?}"
        );
    }

    /// A literal's own body is preserved, not blanked — this crate's own module doc names
    /// why: `A_Rust_Path_Stays_Within_Its_Own_Subtree` and others need a specific string's
    /// real value, not a placeholder.
    #[test]
    fn Test_Code_Prefix_Should_Preserve_A_Strings_Own_Text()
    {
        let line = "let example = \"call unsafe { ... } to do it\";";

        assert_eq!(Code_Prefix(line), line);
    }

    #[test]
    fn Test_Code_Prefix_Should_Strip_A_Real_Comment_After_A_Url_Bearing_String()
    {
        let line = "let doc = \"https://example.com\"; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let doc = "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Honor_An_Escaped_Quote_Inside_A_String()
    {
        let line = "let s = \"a \\\" // not a comment\"; unsafe {}";

        let prefix = Code_Prefix(line);

        assert!(prefix.contains("unsafe {}"), "the escaped quote must not end the string early: {prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Not_Read_A_Quoted_Char_Literal_As_Opening_A_String()
    {
        let line = "let c = '\"'; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let c = "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Not_Read_An_Escaped_Quote_Char_Literal_As_Opening_A_String()
    {
        let line = "let c = '\\''; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let c = "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    /// A lifetime has no closing `'`; it must not be misread as an unterminated char
    /// literal that swallows the rest of the line, including a real trailing comment.
    #[test]
    fn Test_Code_Prefix_Should_Not_Read_A_Lifetime_As_A_Char_Literal()
    {
        let line = "fn F<'a>(x: &'a str) -> &'a str // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.contains("fn F<'a>(x: &'a str) -> &'a str"), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Not_Truncate_At_A_Double_Slash_Inside_A_Raw_String()
    {
        let line = "let s = r\"https://example.com\"; unsafe { core::ptr::read(p) }";

        let prefix = Code_Prefix(line);

        assert!(prefix.contains("unsafe { core::ptr::read(p) }"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Not_Truncate_At_A_Double_Slash_Inside_A_Hashed_Raw_String()
    {
        let line = "let s = r#\"a \" b // still a string\"#; unsafe {}";

        let prefix = Code_Prefix(line);

        assert!(prefix.contains("unsafe {}"), "{prefix:?}");
    }

    /// A raw string with an unbalanced quote inside it — one hash count, one real
    /// delimiter — must still close only at its own real delimiter, not at the stray `"`.
    #[test]
    fn Test_Code_Prefix_Should_Handle_A_Raw_String_With_An_Unbalanced_Quote()
    {
        let line = "let s = r#\"unsafe { let x = \" } \"#; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let s = r#\"unsafe { let x = \" } \"#; "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Strip_A_Comment_After_A_Byte_String()
    {
        let line = "let b = b\"raw // bytes\"; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let b = "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }

    #[test]
    fn Test_Code_Prefix_Should_Strip_A_Comment_After_A_Raw_Byte_String()
    {
        let line = "let b = br\"raw // bytes\"; // real comment";

        let prefix = Code_Prefix(line);

        assert!(prefix.starts_with("let b = "), "{prefix:?}");
        assert!(!prefix.contains("real comment"), "{prefix:?}");
    }
}
