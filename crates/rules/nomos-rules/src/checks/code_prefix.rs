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
//! scanner can see — the same boundary `Has_A_Previous_Comment_Block` already accepts for a
//! `//` comment block spanning lines it walks explicitly rather than one this function
//! tracks silently. What this function adds is single-line correctness: a raw string that
//! opens and closes within one line is read correctly, unbalanced quote and all.
//!
//! # The narrower fix this file's own history pointed at: [`Code_With_String_Bodies_Masked`]
//!
//! `P66-UNSAFE-JUSTIFICATION-STRING-BLINDNESS` measured the gap this module doc already
//! named: `unsafe-justification`'s `Has_Unsafe_Construct` is a bare substring search over
//! `Code_Prefix`'s own output, and `crates/languages/nomos-lang-rust-scan/src/item_kind.rs`
//! declares a table entry, `("unsafe impl", Self::Implementation)`, whose own quoted text
//! spells the exact phrase that search looks for — read as a real declaration, not the
//! string data it is. Blanking every string uniformly already broke three real rules once
//! (this file's own history, above); the fix this second function offers instead is the
//! same scanner, reused for a caller that has already decided it wants a keyword search
//! blind to string content, rather than changing what `Code_Prefix` itself returns to every
//! caller. Nothing before this line changes what `Code_Prefix` does or who calls it.

/// The two bytes a backslash escape occupies: the backslash itself and the byte it escapes.
const ESCAPED_BYTE_LENGTH: usize = 2;

/// Where a char literal's closing quote sits, counted from its opening quote: two bytes on
/// for the single character, three on for the backslash of an escaped one.
const SINGLE_CHAR_LITERAL_CLOSING_OFFSET: usize = 2;
const ESCAPED_CHAR_LITERAL_CLOSING_OFFSET: usize = 3;

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
    while let Some(step) = Code_Step_At(bytes, index)
    {
        output.extend_from_slice(bytes.get(step.start..step.end).unwrap_or_default());
        index = step.end;
    }

    // Every byte pushed above is copied verbatim from `line`'s own valid UTF-8, so this can
    // never fail on real input -- the fallback exists only so this stays a total function
    // rather than one that could panic on a construction its own scanner cannot produce.
    return String::from_utf8(output).unwrap_or_default();
}

/// [`Code_Prefix`]'s own comment-boundary scan, with every string, raw string and char
/// literal's own body replaced by one space per byte rather than copied through — for a
/// caller that searches the result for a bare keyword or construct (`"unsafe {"`, `"unsafe
/// fn "`) and must not read a string literal's own quoted text as that construct. See this
/// file's own module doc for why this is a second, narrowly-scoped function rather than a
/// change to what `Code_Prefix` itself returns: a caller that needs a literal's real value
/// — `#[path = "value"]`'s own path, `.expect("message")`'s own argument — must keep calling
/// `Code_Prefix`, and this function exists only for the opposite, narrower need.
///
/// One space per masked byte, not the byte's own length in some other unit, so a substring
/// search for a multi-character construct spanning a masked region still fails to match
/// (`"unsafe { "` cannot appear inside a run of spaces) without disturbing any position a
/// caller that does not search for one never asks this function to preserve.
#[must_use]
pub(crate) fn Code_With_String_Bodies_Masked(line: &str) -> String
{
    let bytes = line.as_bytes();
    let mut output: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(step) = Code_Step_At(bytes, index)
    {
        Mask_Step(&mut output, bytes, step);
        index = step.end;
    }

    return String::from_utf8(output).unwrap_or_default();
}

/// One step of either scan above: the byte range to consume, and whether that range is a
/// literal's own body. Carried as a value because it is the only thing that tells
/// [`Code_With_String_Bodies_Masked`] a one-byte literal from a plain byte.
#[derive(Clone, Copy)]
struct CodeStep
{
    start: usize,
    end: usize,
    is_literal: bool,
}

/// Writes one step's contribution to [`Code_With_String_Bodies_Masked`]'s output: one space
/// per byte of a literal's own body, or the plain byte itself.
fn Mask_Step(output: &mut Vec<u8>, bytes: &[u8], step: CodeStep)
{
    if step.is_literal
    {
        Mask_Range(output, bytes, step.start, step.end);
        return;
    }

    output.push(bytes.get(step.start).copied().unwrap_or_default());
}

/// Pushes one space for every byte of `bytes[start..end]`, the masked stand-in for a
/// literal's own body [`Code_With_String_Bodies_Masked`] never copies through.
fn Mask_Range(output: &mut Vec<u8>, bytes: &[u8], start: usize, end: usize)
{
    let length = bytes.get(start..end).unwrap_or_default().len();
    let spaces = std::iter::repeat_n(b' ', length);
    output.extend(spaces);
}

/// The step either scan takes at `index`, or `None` once a `//` comment ends the line or the
/// line itself runs out: a whole raw string, string or char literal, else the single byte
/// that is not one.
fn Code_Step_At(bytes: &[u8], index: usize) -> Option<CodeStep>
{
    let Some(&byte) = bytes.get(index)
    else
    {
        return None;
    };
    if byte == b'/' && bytes.get(index.saturating_add(1)) == Some(&b'/')
    {
        return None;
    }
    if let Some((start, end)) = Raw_String_Range_At(bytes, index)
    {
        return Some(CodeStep { start, end, is_literal: true });
    }
    if Is_Opening_A_Literal_Body(bytes, index)
    {
        let end = Literal_Body_End(bytes, index);
        return Some(CodeStep { start: index, end, is_literal: true });
    }

    return Some(CodeStep { start: index, end: index.saturating_add(1), is_literal: false });
}

/// Whether the byte at `index` opens a literal body: every `"` does, and a `'` only when
/// [`Is_Opening_A_Char_Literal`] confirms a real char literal rather than a lifetime.
fn Is_Opening_A_Literal_Body(bytes: &[u8], index: usize) -> bool
{
    if bytes.get(index) == Some(&b'"')
    {
        return true;
    }

    return bytes.get(index) == Some(&b'\'') && Is_Opening_A_Char_Literal(bytes, index);
}

/// One past the closing delimiter of the literal opening at `index`, dispatching on which
/// delimiter that is — the `"` or `'` its caller has already confirmed opens a real body.
fn Literal_Body_End(bytes: &[u8], index: usize) -> usize
{
    return if bytes.get(index) == Some(&b'\'')
    {
        Skip_Char_Literal_Body(bytes, index.saturating_add(1))
    }
    else
    {
        Skip_String_Body(bytes, index.saturating_add(1))
    };
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
            b'\\' => index = index.saturating_add(ESCAPED_BYTE_LENGTH),
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
fn Is_Opening_A_Char_Literal(bytes: &[u8], index: usize) -> bool
{
    if bytes.get(index.saturating_add(1)) == Some(&b'\\')
    {
        return bytes.get(index.saturating_add(ESCAPED_CHAR_LITERAL_CLOSING_OFFSET)) == Some(&b'\'');
    }

    return bytes.get(index.saturating_add(1)).is_some() && bytes.get(index.saturating_add(SINGLE_CHAR_LITERAL_CLOSING_OFFSET)) == Some(&b'\'');
}

/// Advances past a char literal's own body (already confirmed real by
/// [`Is_Opening_A_Char_Literal`]), returning the index one past its closing `'`.
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

/// The half-open byte range of the raw-string literal opening at `index` — the whole literal,
/// opening delimiter included — or `None` when no raw string opens there. Shared by both
/// scans below, which differ only in whether they copy that range through or mask it.
fn Raw_String_Range_At(bytes: &[u8], index: usize) -> Option<(usize, usize)>
{
    let hashes = Raw_String_Opens_At(bytes, index)?;
    let body_start = Raw_String_Body_Start(bytes, index, hashes);
    let end = Skip_Raw_String_Body(bytes, body_start, hashes);

    return Some((index, end));
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
        if bytes.get(index) == Some(&b'"') && Is_Raw_String_Closing_Here(bytes, index, hashes)
        {
            return index.saturating_add(1).saturating_add(hashes);
        }
        index = index.saturating_add(1);
    }

    return index;
}

/// Whether the closing delimiter of a `hashes`-hash raw string sits at `bytes[index]` (a
/// `"`): exactly `hashes` `#` characters immediately follow it.
fn Is_Raw_String_Closing_Here(bytes: &[u8], index: usize, hashes: usize) -> bool
{
    let start = index.saturating_add(1);
    let end = start.saturating_add(hashes);

    return bytes.get(start..end).is_some_and(|slice| return slice.iter().all(|byte| return *byte == b'#'));
}

#[cfg(test)]
mod tests;
