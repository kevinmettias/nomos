//! Source text never carries trailing whitespace.
//!
//! This is the direct Nomos form of code-standards'
//! `style/formatting/multiline-generic-and-where-formatting/no-trailing-whitespace.md`:
//! a line's break-point character is its last non-whitespace character. Unlike the
//! syntax-backed rules in this crate, the deciding evidence is exactly the text the caller
//! already handed to the rule; routing it through a capability would invent a provider
//! question where none exists.
//!
//! [`Check_Deprecation_Carries_A_Reason`] imports code-standards' `deprecation` rule for
//! its two text-decidable forms: a bare Rust `#[deprecated]` (or one whose arguments close
//! on the same line and say nothing under `note`), and a Go `// Deprecated:` marker with
//! nothing after the colon. A multi-line `#[deprecated(...)]` is left unjudged rather than
//! guessed at, the same conservative stance [`Check_No_Decorative_Section_Dividers`] takes
//! toward prose that merely quotes a divider.
//!
//! [`Check_No_Single_Line_Function_Bodies`] imports code-standards'
//! `no-single-line-function-bodies`: a Rust function whose signature, opening brace, body
//! and closing brace collapse onto one line, including an empty `{}` trailing the
//! signature. Scoped to Rust only for now — code-standards names a distinct C# strategy for
//! the same rule id, and Go's own body-collapsing shape is left unattempted rather than
//! guessed at. Judges only a line's own code: a string literal (plain, byte, or raw, any of
//! which may carry a fake `"fn a() {}"`-shaped fixture, and a plain or raw string may span
//! several physical lines), a char or byte-char literal (`'"'`, `b'"'` — this crate's own
//! `Quote_State_After` writes exactly that to name the double-quote byte, and was one of the
//! false positives measured below until this rule could tell it apart from a lifetime, which
//! is never closed by a second `'` and so is left alone rather than guessed at), and a
//! `//`/`///` comment are all blanked out first, tracked across lines the same way
//! [`Check_Executed_Scripts_Set_Nounset`]'s own shell-quote state already carries forward —
//! composing this rule with none of that measured 287 false positives out of 288 findings
//! against this workspace's own tree. `/* ... */` block comments and a `\u{...}` char-literal
//! escape are not tracked: this workspace's own measured false positives carry neither
//! shape, so the gap is left silent rather than guessed at.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const NO_TRAILING_WHITESPACE: &str = "no-trailing-whitespace";
/// This rule's own identifier, matching the code-standards rule id.
pub const TODO_FORMAT: &str = "todo-format-is-todo-name-description-ticket";
/// This rule's own identifier, matching the code-standards rule id.
pub const NO_DECORATIVE_SECTION_DIVIDERS: &str = "no-decorative-section-dividers";
/// This rule's own identifier, matching the code-standards rule id.
pub const DEPRECATION: &str = "deprecation";
/// This rule's own identifier, matching the code-standards rule id.
pub const NO_SINGLE_LINE_FUNCTION_BODIES: &str = "no-single-line-function-bodies";

/// Reports every line in `sources` whose content ends in a space or tab.
#[must_use]
pub fn Check_No_Trailing_Whitespace(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports every deferred-work marker comment in `sources` that does not carry owner,
/// description and ticket.
///
/// The marker itself and the exact shape are the rule id's own words. This prose does not
/// spell the marker, because detection and acceptance agree on what counts as one: the
/// marker must open the comment. A comment that merely mentions it in passing is not a
/// marker and is not judged, so a comment that does open with it and is missing owner,
/// description or ticket always has a conforming edit.
#[must_use]
pub fn Check_Todo_Format(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Todo_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports standalone decorative comment dividers such as `// ===== Setup =====`.
#[must_use]
pub fn Check_No_Decorative_Section_Dividers(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Divider_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a `#[deprecated]`/`#[deprecated(...)]` attribute with no `note`, or a Go
/// `// Deprecated:` marker with nothing after the colon.
#[must_use]
pub fn Check_Deprecation_Carries_A_Reason(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Deprecation_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports a Rust function whose signature, opening brace, body and closing brace all
/// collapse onto one line — including an empty `{}` trailing the signature, which carries
/// no body at all.
#[must_use]
pub fn Check_No_Single_Line_Function_Bodies(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Single_Line_Body_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut line_number = 1usize;

    for line in source.text.split_inclusive('\n')
    {
        if Has_Trailing_Whitespace(line)
        {
            findings.push(Finding_For_Line(source, NO_TRAILING_WHITESPACE, line_number, "ends with trailing whitespace"));
        }

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }

    return findings;
}

fn Todo_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut line_number = 1usize;

    for line in source.text.split_inclusive('\n')
    {
        if let Some(comment) = Comment_Text_Of(line)
        {
            if Starts_With_Todo(comment) && !Has_Valid_Todo_Format(comment)
            {
                findings.push(Finding_For_Line(
                    source,
                    TODO_FORMAT,
                    line_number,
                    "contains a TODO without `TODO(owner): description (#ticket)` format",
                ));
            }
        }

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }

    return findings;
}

fn Divider_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut line_number = 1usize;

    for line in source.text.split_inclusive('\n')
    {
        if Is_Decorative_Divider(line)
        {
            findings.push(Finding_For_Line(
                source,
                NO_DECORATIVE_SECTION_DIVIDERS,
                line_number,
                "is a decorative section divider",
            ));
        }

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }

    return findings;
}

fn Deprecation_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut line_number = 1usize;

    for line in source.text.split_inclusive('\n')
    {
        if Rust_Deprecated_Without_Reason(line)
        {
            findings.push(Finding_For_Line(
                source,
                DEPRECATION,
                line_number,
                "marks `#[deprecated]` with no `note` for the caller",
            ));
        }
        else if let Some(comment) = Comment_Text_Of(line)
        {
            if Go_Deprecated_Without_Reason(comment)
            {
                findings.push(Finding_For_Line(
                    source,
                    DEPRECATION,
                    line_number,
                    "marks `// Deprecated:` with nothing after the colon",
                ));
            }
        }

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }

    return findings;
}

fn Single_Line_Body_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut line_number = 1usize;
    let mut literal_state = RustLiteralState::None;

    for line in source.text.split_inclusive('\n')
    {
        let (code, next_state) = Code_Only(line, literal_state);
        literal_state = next_state;

        if Has_Single_Line_Function_Body(&code)
        {
            findings.push(Finding_For_Line(
                source,
                NO_SINGLE_LINE_FUNCTION_BODIES,
                line_number,
                "collapses a function's signature, body and closing brace onto one line",
            ));
        }

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }

    return findings;
}

/// A `fn ` keyword followed, later in `code`, by a `{` and then a `}` — the shape a properly
/// multi-line (Allman-braced) function never has, since its own opening brace starts a new
/// line with nothing after it. Matches an empty `{}` too: the standard names that collapse
/// as a violation in its own right, not only a non-empty collapsed body. `code` is already
/// [`Code_Only`]'s output, not raw source text — this function never sees a string literal
/// or a comment, so it cannot mistake either for a real declaration.
fn Has_Single_Line_Function_Body(code: &[u8]) -> bool
{
    let Some(fn_index) = Index_Of_Subslice(code, b"fn ") else { return false };
    let after_fn = code.get(fn_index..).unwrap_or(&[]);
    let Some(open_brace) = after_fn.iter().position(|&byte| return byte == b'{') else { return false };
    let after_open = after_fn.get(open_brace.saturating_add(1)..).unwrap_or(&[]);
    return after_open.contains(&b'}');
}

/// The Rust string literal `line` is left inside at its own end, given whichever was open
/// when it began. A raw string's own hash count has to travel with the state because closing
/// one takes exactly that many `#` after the `"`, which a bare "am I in a string" bool cannot
/// carry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RustLiteralState
{
    /// No literal is open; the next line begins in code.
    None,
    /// A `"..."` (plain or `b"..."` byte-string — the `b` prefix takes no special handling,
    /// since it is ordinary code up to the `"` that actually opens the literal) literal is
    /// open. Backslash escapes inside these, including a trailing `\` that continues the
    /// literal onto the next line unclosed rather than escaping nothing.
    Double,
    /// A `r"..."`/`r#"..."#`/... literal is open, closing only on a `"` followed by exactly
    /// this many `#`. Raw strings take no escapes at all, not even of `"` itself.
    Raw(usize),
}

/// `line`'s own code, with every string literal and every `//`/`///` line comment blanked
/// out — the bytes [`Has_Single_Line_Function_Body`] actually judges, so a `"fn a() {}"`
/// fixture string or a doc comment quoting the same shape is never mistaken for a real
/// declaration. Returns the literal state `line` leaves open for whichever line comes next,
/// which is how a raw string or a backslash-continued plain string spanning several physical
/// lines is still read as one literal rather than several unrelated ones.
fn Code_Only(line: &str, entering: RustLiteralState) -> (Vec<u8>, RustLiteralState)
{
    let bytes = line.as_bytes();
    let mut state = entering;
    let mut index = 0usize;
    let mut code = Vec::with_capacity(line.len());

    while let Some(&byte) = bytes.get(index)
    {
        match state
        {
            RustLiteralState::Double =>
            {
                if byte == b'\\'
                {
                    index = index.saturating_add(2); // the escaped character cannot close anything
                    continue;
                }
                if byte == b'"'
                {
                    state = RustLiteralState::None;
                }
                index = index.saturating_add(1);
            }
            RustLiteralState::Raw(hashes) =>
            {
                if byte == b'"' && Closes_Raw_String(bytes, index, hashes)
                {
                    index = index.saturating_add(1).saturating_add(hashes);
                    state = RustLiteralState::None;
                    continue;
                }
                index = index.saturating_add(1);
            }
            RustLiteralState::None =>
            {
                if byte == b'/' && bytes.get(index.saturating_add(1)) == Some(&b'/')
                {
                    return (code, state); // a line comment: nothing after it is code
                }
                if let Some(length) = Char_Literal_Length(bytes, index)
                {
                    index = index.saturating_add(length); // e.g. `'"'`: its own `"` opens nothing
                    continue;
                }
                if let Some(hashes) = Raw_String_Opener_Hashes(bytes, index)
                {
                    state = RustLiteralState::Raw(hashes);
                    index = index.saturating_add(2).saturating_add(hashes); // past `r`, its `#`s, and the opening `"`
                    continue;
                }
                if byte == b'"'
                {
                    state = RustLiteralState::Double;
                    index = index.saturating_add(1);
                    continue;
                }
                code.push(byte);
                index = index.saturating_add(1);
            }
        }
    }

    return (code, state);
}

/// The byte length of the char (or byte-char) literal at `bytes[index]` — `'a'`, `'"'`,
/// `'\n'`, `'\''`, `'\x41'` — or `None` if `bytes[index]` is not `'` or what follows is not
/// one of those shapes. This crate's own `Quote_State_After` (`script_discipline.rs`) writes
/// `b'"'` to name the double-quote byte, and without this a scan in [`RustLiteralState::
/// None`] reads that `"` as opening a real string — the reason this exists at all.
///
/// Deliberately not a lifetime reader: `'a`, `'static` and `'_` all fail every branch below
/// (nothing closes them with a second `'`) and fall through as an ordinary byte, which is
/// the correct outcome — a lifetime's own apostrophe carries no quoting meaning to skip past.
/// A `\u{...}` escape is the one real char-literal shape this does not recognize, since its
/// length varies; nothing in this workspace's own measured false positives has that shape,
/// so the gap is left silent rather than guessed at.
fn Char_Literal_Length(bytes: &[u8], index: usize) -> Option<usize>
{
    if bytes.get(index) != Some(&b'\'')
    {
        return None;
    }

    let escape_or_char = index.saturating_add(1);
    if bytes.get(escape_or_char) != Some(&b'\\')
    {
        let closing = escape_or_char.saturating_add(1);
        return (bytes.get(escape_or_char).is_some() && bytes.get(closing) == Some(&b'\'')).then_some(3);
    }

    let escaped = escape_or_char.saturating_add(1);
    if bytes.get(escaped) == Some(&b'x')
    {
        let closing = escaped.saturating_add(3);
        return (bytes.get(closing) == Some(&b'\'')).then_some(6);
    }

    let closing = escaped.saturating_add(1);
    return (bytes.get(escaped).is_some() && bytes.get(closing) == Some(&b'\'')).then_some(4);
}

/// The number of `#` between `bytes[index]` and the `"` that opens a raw string there
/// (`r"`, `r#"`, `r##"`, ...), or `None` if `bytes[index]` does not begin one. `r` must
/// start a word — otherwise an identifier merely ending in `r` would be misread as one.
fn Raw_String_Opener_Hashes(bytes: &[u8], index: usize) -> Option<usize>
{
    if bytes.get(index) != Some(&b'r')
    {
        return None;
    }
    if index > 0 && bytes.get(index.saturating_sub(1)).is_some_and(|&before| return Is_Identifier_Byte(before))
    {
        return None;
    }

    let mut hashes = 0usize;
    let mut cursor = index.saturating_add(1);
    while bytes.get(cursor) == Some(&b'#')
    {
        hashes = hashes.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }

    if bytes.get(cursor) == Some(&b'"')
    {
        return Some(hashes);
    }

    return None;
}

/// Whether the `"` at `bytes[index]` closes a raw string that opened with `hashes` `#`s —
/// the next `hashes` bytes must all be `#`.
fn Closes_Raw_String(bytes: &[u8], index: usize, hashes: usize) -> bool
{
    return (0..hashes).all(|offset| return bytes.get(index.saturating_add(1).saturating_add(offset)) == Some(&b'#'));
}

fn Is_Identifier_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// The index of `needle`'s first occurrence in `haystack`, or `None`.
fn Index_Of_Subslice(haystack: &[u8], needle: &[u8]) -> Option<usize>
{
    return haystack.windows(needle.len()).position(|window| return window == needle);
}

fn Has_Trailing_Whitespace(line: &str) -> bool
{
    let content = line.trim_end_matches(['\r', '\n']);

    return content.ends_with(' ') || content.ends_with('\t');
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();

    if let Some(after_slashes) = trimmed.strip_prefix("//")
    {
        return Some(after_slashes.trim_start_matches('/').trim_start());
    }

    for marker in ["/*", "*"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }

    return None;
}

/// A marker opens the comment; a mention elsewhere in the comment's prose is not one, and is
/// left unjudged rather than reported with no route to a conforming edit.
fn Starts_With_Todo(comment: &str) -> bool
{
    return comment.starts_with("TODO");
}

fn Has_Valid_Todo_Format(comment: &str) -> bool
{
    let Some(after_marker) = comment.strip_prefix("TODO(")
    else
    {
        return false;
    };
    let Some((owner, after_owner)) = after_marker.split_once("):")
    else
    {
        return false;
    };
    if owner.trim().is_empty() || owner.chars().any(char::is_whitespace)
    {
        return false;
    }

    let description = after_owner.trim();
    if description.is_empty()
    {
        return false;
    }

    return Has_Ticket_Suffix(description);
}

fn Has_Ticket_Suffix(description: &str) -> bool
{
    let Some(ticket) = description.strip_suffix(')')
    else
    {
        return false;
    };
    let Some((before_ticket, number)) = ticket.rsplit_once("(#")
    else
    {
        return false;
    };

    return !before_ticket.trim().is_empty() && !number.is_empty() && number.chars().all(|character| return character.is_ascii_digit());
}

fn Is_Decorative_Divider(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    let content = comment.trim().trim_end_matches(['\r', '\n']).trim();
    if Divider_Punctuation_Count(content) < 6
    {
        return false;
    }

    let label = content.trim_matches(Is_Divider_Punctuation).trim();
    return label.is_empty() || Is_Short_Label(label);
}

fn Divider_Punctuation_Count(content: &str) -> usize
{
    return content
        .chars()
        .filter(|character| return Is_Divider_Punctuation(*character))
        .count();
}

fn Is_Divider_Punctuation(character: char) -> bool
{
    return matches!(character, '=' | '-' | '_' | '*' | '/');
}

fn Is_Short_Label(label: &str) -> bool
{
    let words = label.split_whitespace().count();
    return words > 0 && words <= 4 && label.chars().all(|character| {
        return character.is_ascii_alphanumeric() || character.is_ascii_whitespace() || character == '_' || character == '-';
    });
}

/// A bare `#[deprecated]`, or `#[deprecated(...)]` whose parenthesized arguments close on
/// this same line and do not mention `note`. An attribute whose arguments do not close on
/// this line is not decidable from one line alone and is left unjudged rather than guessed.
fn Rust_Deprecated_Without_Reason(line: &str) -> bool
{
    let after = line.trim_start().strip_prefix("#[deprecated");
    let Some(after) = after
    else
    {
        return false;
    };

    if after.starts_with(']')
    {
        return true;
    }

    let Some(arguments) = after.strip_prefix('(')
    else
    {
        return false;
    };
    let Some((arguments, _rest)) = arguments.split_once(")]")
    else
    {
        return false;
    };

    return !arguments.contains("note");
}

/// A Go `Deprecated:` doc-comment marker with nothing but whitespace after the colon.
fn Go_Deprecated_Without_Reason(comment: &str) -> bool
{
    let Some(after) = comment.strip_prefix("Deprecated:")
    else
    {
        return false;
    };

    return after.trim().is_empty();
}

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_No_Trailing_Whitespace_Should_Report_A_Line_Ending_In_A_Space()
    {
        let source = Source("src/lib.rs", "fn Clean()\n{\n    return; \n}\n");

        let findings = Check_No_Trailing_Whitespace(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(NO_TRAILING_WHITESPACE));
        assert_eq!(found.subject_name, "src/lib.rs:3");
        assert_eq!(found.locations, vec!["src/lib.rs:3".to_owned()]);
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_No_Trailing_Whitespace_Should_Report_A_Line_Ending_In_A_Tab()
    {
        let source = Source("src/lib.rs", "fn Clean()\n{\t\n}\n");

        let findings = Check_No_Trailing_Whitespace(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:2");
    }

    #[test]
    fn Test_Check_No_Trailing_Whitespace_Should_Report_The_Final_Line_Without_A_Newline()
    {
        let source = Source("src/lib.rs", "fn Clean() {} ");

        let findings = Check_No_Trailing_Whitespace(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:1");
    }

    #[test]
    fn Test_Check_No_Trailing_Whitespace_Should_Accept_Clean_Text()
    {
        let source = Source("src/lib.rs", "fn Clean()\n{\n    return;\n}\n");

        let findings = Check_No_Trailing_Whitespace(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Todo_Format_Should_Report_A_Todo_Without_Owner_Description_And_Ticket()
    {
        let source = Source("src/lib.rs", "// TODO fix this later\n");

        let findings = Check_Todo_Format(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(TODO_FORMAT));
        assert_eq!(found.subject_name, "src/lib.rs:1");
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_Todo_Format_Should_Accept_A_Tracked_Todo()
    {
        let source = Source("src/lib.rs", "// TODO(kevin): replace fallback path once selection lands (#142)\n");

        let findings = Check_Todo_Format(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Todo_Format_Should_Ignore_A_String_Containing_Todo()
    {
        let source = Source("src/lib.rs", "let label = \"TODO fix this\";\n");

        let findings = Check_Todo_Format(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Todo_Format_Should_Report_A_Todo_With_No_Ticket()
    {
        let source = Source("src/lib.rs", "/// TODO(kevin): replace fallback path\n");

        let findings = Check_Todo_Format(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Todo_Format_Should_Ignore_A_Todo_Mentioned_Mid_Comment()
    {
        let source = Source("src/lib.rs", "// see TODO(kevin): fix later (#142) above\n");

        let findings = Check_Todo_Format(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Decorative_Section_Dividers_Should_Report_A_Bare_Divider()
    {
        let source = Source("src/lib.rs", "// ====================\n");

        let findings = Check_No_Decorative_Section_Dividers(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_DECORATIVE_SECTION_DIVIDERS));
    }

    #[test]
    fn Test_Check_No_Decorative_Section_Dividers_Should_Report_A_Labelled_Divider()
    {
        let source = Source("src/lib.rs", "// ===== Internal Helpers =====\n");

        let findings = Check_No_Decorative_Section_Dividers(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Decorative_Section_Dividers_Should_Ignore_Prose_That_Quotes_A_Divider()
    {
        let source = Source("src/lib.rs", "// The old code used // ===== Setup ===== as a divider.\n");

        let findings = Check_No_Decorative_Section_Dividers(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_A_Bare_Rust_Marker()
    {
        let source = Source("src/lib.rs", "#[deprecated]\npub fn Old() {}\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(DEPRECATION));
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_Since_With_No_Note()
    {
        let source = Source("src/lib.rs", "#[deprecated(since = \"2.1\")]\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Accept_A_Note()
    {
        let source = Source("src/lib.rs", "#[deprecated(since = \"2.1\", note = \"use New instead\")]\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Ignore_A_Multiline_Attribute()
    {
        let source = Source("src/lib.rs", "#[deprecated(\n    note = \"use New instead\"\n)]\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Report_A_Bare_Go_Marker()
    {
        let source = Source("main.go", "// Deprecated:\nfunc Old() {}\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Accept_A_Go_Marker_With_A_Reason()
    {
        let source = Source("main.go", "// Deprecated: use New instead.\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Deprecation_Carries_A_Reason_Should_Ignore_A_String_Containing_The_Marker()
    {
        let source = Source("src/lib.rs", "let label = \"Deprecated: nothing\";\n");

        let findings = Check_Deprecation_Carries_A_Reason(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Report_A_Collapsed_Body()
    {
        let source = Source("src/lib.rs", "pub fn Add(a: i32, b: i32) -> i32 { return a + b; }\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES));
    }

    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Report_A_Collapsed_Empty_Body()
    {
        let source = Source("src/lib.rs", "pub fn Noop() {}\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Accept_An_Allman_Body()
    {
        let source = Source("src/lib.rs", "pub fn Add(a: i32, b: i32) -> i32\n{\n    return a + b;\n}\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Accept_A_Trait_Method_Declaration()
    {
        let source = Source("src/lib.rs", "trait Shape\n{\n    fn Area(&self) -> f64;\n}\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Collapsed_Closure_Inside_A_Multiline_Body()
    {
        let source = Source(
            "src/lib.rs",
            "pub fn Sum(values: &[i32]) -> i32\n{\n    return values.iter().fold(0, |acc, x| { acc + x });\n}\n",
        );

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `crates/rules/nomos-rules/src/checks/script_discipline.rs`'s own real shape: a byte
    /// char literal `b'"'` whose content is a bare `"`, found only once this rule was
    /// composed into a real run and left every line after it misread as inside an unclosed
    /// string. A lifetime (`'a`, `'_`) is not a char literal and must still fall through.
    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Not_Misread_A_Quote_Char_Literal()
    {
        let source = Source(
            "src/lib.rs",
            "pub fn Is_Quote<'a>(character: u8) -> bool\n{\n    return character == b'\"';\n}\n\npub fn Real() {}\n",
        );

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "src/lib.rs:6");
    }

    /// `crates/substrate/nomos-workspace/src/workspace.rs`'s own real shape: a fake file
    /// content string handed to a test fixture, not a real declaration.
    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Same_Line_String_Fixture()
    {
        let source = Source("src/lib.rs", "let changes = Present(\"a.rs\", \"fn a() {}\");\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `tests/contract/src/reading/source_files.rs`'s own real shape: a raw string spanning
    /// several physical lines whose content looks, line by line, like a real declaration.
    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Multiline_Raw_String()
    {
        let source = Source("src/lib.rs", "let source = r\"\npub fn Helper() {}\n\";\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `crates/languages/nomos-lang-rust/src/syntax/tests.rs`'s own real shape: a plain
    /// string literal continued onto the next physical line by a trailing `\`.
    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Backslash_Continued_String()
    {
        let source = Source("src/lib.rs", "let source = \"pub fn exported() {}\\n\\\n     fn hidden() {}\\n\";\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// This rule's own module doc, before this fix, was itself one of the false positives.
    #[test]
    fn Test_Check_No_Single_Line_Function_Bodies_Should_Ignore_A_Comment_Quoting_The_Shape()
    {
        let source = Source("src/lib.rs", "/// A collapsed body looks like `fn f() { ... }` in prose.\n");

        let findings = Check_No_Single_Line_Function_Bodies(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
