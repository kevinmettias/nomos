//! Script discipline rules from code-standards: [`Check_Scripts_Use_A_Portable_Shebang`],
//! [`Check_A_Script_Declares_Its_Purpose`], [`Check_Executed_Scripts_Set_Nounset`] and
//! [`Check_Declared_Tooling_Language_For_Scripts`], each carried in its own file by
//! responsibility, with only what they share -- reading a file's shebang line, and building a
//! finding from it -- left here.

mod nounset;
mod purpose;
mod shebang;
mod tooling_language;

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

pub use nounset::Check_Executed_Scripts_Set_Nounset;
pub use purpose::Check_A_Script_Declares_Its_Purpose;
pub use shebang::Check_Scripts_Use_A_Portable_Shebang;
pub use tooling_language::Check_Declared_Tooling_Language_For_Scripts;

/// The code-standards portable-shebang rule id.
pub const SCRIPTS_USE_A_PORTABLE_SHEBANG: &str = "scripts-use-a-portable-shebang";
/// The code-standards script-purpose rule id.
pub const A_SCRIPT_DECLARES_ITS_PURPOSE: &str = "a-script-declares-its-purpose";
/// The code-standards executed-scripts-nounset rule id.
pub const EXECUTED_SCRIPTS_SET_NOUNSET: &str = "executed-scripts-set-nounset";
/// The code-standards declared-tooling-language rule id.
pub const DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS: &str = "declared-tooling-language-for-scripts";

/// Whether a source's first line is a shebang at all -- the one question the script rules
/// that take only a path and a body ask before looking any further.
fn Is_Shebang_Script(source: &SourceFile) -> bool
{
    return Shebang_Interpreter(source).is_some();
}

/// The interpreter a source's first line names, if that line is a shebang at all.
///
/// A shebang names its interpreter by absolute path, so `#!` is followed -- after the
/// optional space some conventions write -- by `/`. The two bytes alone do not distinguish
/// a script from a file whose language spells something else the same way: Rust's inner
/// attributes begin `#![`, so every crate root opening with `#![forbid(unsafe_code)]` read
/// as a script here, and was then reported for a hardcoded interpreter it does not have and
/// a purpose comment a Rust file has no place to put. Requiring the path keeps the rules
/// language-agnostic, which is the reason they judge a first line rather than an extension.
fn Shebang_Interpreter(source: &SourceFile) -> Option<&str>
{
    let path = First_Line(source)?.strip_prefix("#!")?.trim_start_matches([' ', '\t']);

    if !path.starts_with('/')
    {
        return None;
    }

    return Some(path);
}

/// The first line of a source, or `None` for a file with no text at all.
fn First_Line(source: &SourceFile) -> Option<&str>
{
    return source.text.lines().next();
}

/// `rule` and `because` are both `&str`; without a distinct type per position, a call site
/// like `Finding_For_Source(source, rule, because)` reads as two interchangeable strings and
/// a swap compiles silently.
struct Rule<'a>(&'a str);
struct Because<'a>(&'a str);

fn Finding_For_Source(source: &SourceFile, rule: Rule<'_>, because: Because<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(rule.0),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {}", source.path, because.0),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
#[path = "script_discipline/tests.rs"]
mod tests;
