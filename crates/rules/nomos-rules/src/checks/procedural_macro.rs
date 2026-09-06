//! A function-like procedural macro with no justification, ported from code-standards'
//! `check-proc-macro` (`rules/language-specific/rust/structure/proc-macro`).
//!
//! A procedural macro is not a cheaper `macro_rules!`. It costs a separate compiled crate
//! that every dependent must build before its own code can be parsed, and it drags `syn`,
//! `quote` and `proc-macro2` onto every build that reaches it. What it buys is real Rust
//! syntax parsing, which a `macro_rules!` pattern cannot do — so the rule is not against
//! procedural macros, it is against paying that price for a pattern that did not need it.
//!
//! # Only the function-like form, and that is the point
//!
//! `#[proc_macro_derive]` and `#[proc_macro_attribute]` are the *sanctioned* forms: a derive
//! and an attribute macro both have to read the item they are attached to, which is exactly
//! the true-syntax-parsing case the rule reserves procedural macros for. Only the bare
//! `#[proc_macro]` is judged, which is why the match below requires the closing bracket
//! rather than accepting anything opening with the attribute's name — anchoring on a prefix
//! would report both sanctioned forms as violations of the rule that sanctions them.
//!
//! # A justification is an adjacent comment
//!
//! The original accepts one marker spelling. This accepts any adjacent explanatory comment,
//! the shape [`super::rust_text::Check_Inline_Always_Justification`] already established
//! here — see [`super::lifetime_discipline`], which makes the same divergence and records
//! the measurement behind it.

use super::code_prefix::Code_Prefix;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards procedural-macro rule id.
pub const PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS: &str = "prefer-macro-rules-over-procedural-macros";

/// The attribute name the rule judges. The two sanctioned forms carry longer names, which
/// the closing bracket is what excludes.
const PROCEDURAL_MACRO_ATTRIBUTE: &str = "proc_macro";

/// How far above the attribute an explanation may sit and still be its explanation.
const JUSTIFICATION_WINDOW_LINES: usize = 3;

/// Reports a bare `#[proc_macro]` with no adjacent comment saying why the pattern needs
/// true Rust-syntax parsing.
#[must_use]
pub fn Check_Prefer_Macro_Rules_Over_Procedural_Macros(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources.iter().filter(|source| return source.Is_Written_In(RUST_LANGUAGE))
    {
        findings.extend(Procedural_Macro_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Procedural_Macro_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if Declares_A_Function_Like_Procedural_Macro(&Code_Prefix(line)) && !Has_Adjacent_Explanation(&lines, index)
        {
            findings.push(Procedural_Macro_Finding(source, index.saturating_add(1)));
        }
    }

    return findings;
}

/// Whether `code` carries the bare attribute: an opening `#[`, the name, and the closing
/// `]` with nothing in between but space.
fn Declares_A_Function_Like_Procedural_Macro(code: &str) -> bool
{
    let mut searched_from = 0usize;

    while let Some(offset) = code.get(searched_from..).and_then(|rest| return rest.find("#["))
    {
        let start = searched_from.saturating_add(offset);
        let inside = code.get(start.saturating_add(2)..).unwrap_or("");

        if Closes_Immediately_After_The_Name(inside)
        {
            return true;
        }

        searched_from = start.saturating_add(2);
    }

    return false;
}

/// Whether the text inside an attribute is exactly the name, so `proc_macro_derive` and
/// `proc_macro_attribute` — which continue past it — do not match.
fn Closes_Immediately_After_The_Name(inside: &str) -> bool
{
    return inside
        .trim_start()
        .strip_prefix(PROCEDURAL_MACRO_ATTRIBUTE)
        .is_some_and(|rest| return rest.trim_start().starts_with(']'));
}

/// Whether any of the few lines above `index` carries a comment with words in it.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    let first = index.saturating_sub(JUSTIFICATION_WINDOW_LINES);

    return lines
        .get(first..index)
        .unwrap_or(&[])
        .iter()
        .any(|line| return Is_An_Explanatory_Comment(line));
}

fn Is_An_Explanatory_Comment(line: &str) -> bool
{
    let trimmed = line.trim_start();
    let Some(after_slashes) = trimmed.strip_prefix("//")
    else
    {
        return false;
    };

    return after_slashes
        .trim_start_matches(['/', '!'])
        .trim()
        .chars()
        .any(|character| return character.is_ascii_alphanumeric());
}

fn Procedural_Macro_Finding(source: &SourceFile, line_number: usize) -> Finding
{
    let location = format!("{}:{line_number}", source.path);

    return Finding {
        rule: RuleId::New(PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS),
        subject: source.subject,
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "{location} declares a function-like procedural macro with no adjacent comment saying why; it costs a compiled crate and the syn, quote and proc-macro2 surface on every dependent build, so express it with macro_rules or say what needs real syntax parsing"
        ),
        locations: vec![location],
    };
}

/// The code before any line comment. This crate's established per-file convention, which
/// `P45-CODE-PREFIX-KNOWS-STRINGS` will replace with one shared helper.
#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Report_A_Bare_Attribute()
    {
        let sources = vec![Source("demo/src/a.rs", &Attribute(""))];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS));
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    /// The two forms the rule sanctions, because each has to read the item it decorates —
    /// the true-syntax-parsing case procedural macros are reserved for.
    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Accept_The_Derive_And_Attribute_Forms()
    {
        let sources = vec![
            Source("demo/src/a.rs", &Attribute("_derive(Thing)")),
            Source("demo/src/b.rs", &Attribute("_attribute")),
        ];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Accept_An_Adjacent_Explanation()
    {
        let text = format!(
            "// The pattern reads the caller's own token stream and rewrites it, which no\n// declarative macro can express.\n{}",
            Attribute("")
        );
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Not_Accept_A_Wordless_Comment()
    {
        let text = format!("// ----\n{}", Attribute(""));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Ignore_A_Commented_Attribute()
    {
        let text = format!("// {}", Attribute(""));
        let sources = vec![Source("demo/src/a.rs", &text)];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Prefer_Macro_Rules_Over_Procedural_Macros_Should_Ignore_A_Language_It_Does_Not_Judge()
    {
        let sources = vec![Source("demo/src/a.go", &Attribute(""))];

        let findings = Check_Prefer_Macro_Rules_Over_Procedural_Macros(&sources);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// Built rather than spelled, so this file's own text never carries the attribute it
    /// judges — the self-match nine modules in this crate answer with a path-suffix
    /// exemption `P45-CODE-PREFIX-KNOWS-STRINGS` is retiring.
    fn Attribute(suffix: &str) -> String
    {
        return format!("#[{PROCEDURAL_MACRO_ATTRIBUTE}{suffix}]");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
