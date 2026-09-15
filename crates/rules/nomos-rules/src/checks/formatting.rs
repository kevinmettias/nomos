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
//!
//! # Split by responsibility
//!
//! Five rules in one file passed the size at which this crate extracts a submodule, and the
//! split follows the one thing that genuinely separates them. [`single_line_body`] is the
//! only rule here that decides from more than one line, so its byte-walking machine and the
//! literal state it carries across lines live there; the four rules whose evidence is exactly
//! the line in front of them each have a child of their own — [`trailing_whitespace`],
//! [`todo_format`], [`decorative_dividers`] and [`deprecation`]. The per-line primitives all
//! five report through — [`For_Each_Line_Number`], [`Comment_Text_Of`] and
//! [`Finding_For_Line`] — and the rule ids stay here, with their end-to-end tests.

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

mod decorative_dividers;
mod deprecation;
mod single_line_body;
mod todo_format;
mod trailing_whitespace;

pub use decorative_dividers::Check_No_Decorative_Section_Dividers;
pub use deprecation::Check_Deprecation_Carries_A_Reason;
pub use single_line_body::Check_No_Single_Line_Function_Bodies;
pub use todo_format::Check_Todo_Format;
pub use trailing_whitespace::Check_No_Trailing_Whitespace;
/// Re-exported rather than reached through the child directly: `checks::nesting_depth` reads
/// both, and the split above must not move the path they are reached by.
pub(crate) use single_line_body::{Advance_Literal_State, RustLiteralState};

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

/// Walks `text` line by line (each line inclusive of its own trailing `\n`), calling `visit`
/// with each line's 1-based line number. A `usize::MAX` overflow simply stops the walk rather
/// than wrapping into a line number that lies about where the text actually is.
fn For_Each_Line_Number(text: &str, mut visit: impl FnMut(usize, &str))
{
    let mut line_number = 1usize;

    for line in text.split_inclusive('\n')
    {
        visit(line_number, line);

        match line_number.checked_add(1)
        {
            Some(next) => line_number = next,
            None => break,
        }
    }
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
mod tests;
