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

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const NO_TRAILING_WHITESPACE: &str = "no-trailing-whitespace";
/// This rule's own identifier, matching the code-standards rule id.
pub const TODO_FORMAT: &str = "todo-format-is-todo-name-description-ticket";
/// This rule's own identifier, matching the code-standards rule id.
pub const NO_DECORATIVE_SECTION_DIVIDERS: &str = "no-decorative-section-dividers";
/// This rule's own identifier, matching the code-standards rule id.
pub const DEPRECATION: &str = "deprecation";

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

/// Reports every TODO comment in `sources` that does not carry owner, description and ticket.
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
            if Contains_Todo(comment) && !Has_Valid_Todo_Format(comment)
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

fn Has_Trailing_Whitespace(line: &str) -> bool
{
    let content = line.trim_end_matches(['\r', '\n']);

    return content.ends_with(' ') || content.ends_with('\t');
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();

    for marker in ["//", "/*", "*"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }

    return None;
}

fn Contains_Todo(comment: &str) -> bool
{
    return comment.contains("TODO");
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

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
