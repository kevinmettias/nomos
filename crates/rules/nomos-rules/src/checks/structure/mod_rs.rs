//! The one `structure.rs` rule that is text-independent: the path alone decides it.

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

use super::NO_MOD_RS_FILES;

/// Reports `src/**/mod.rs` files, exempting shared integration-test modules under `tests/`.
#[must_use]
pub fn Check_No_Mod_Rs_Files(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Disallowed_Mod_Rs(&source.path)
        {
            let finding = Finding_For_Source(
                source,
                Rule(NO_MOD_RS_FILES),
                Because("uses the old Rust module layout; use a sibling module file instead"),
            );
            findings.push(finding);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Is_Disallowed_Mod_Rs(path: &str) -> bool
{
    let normalized = path.replace('\\', "/");
    return normalized.ends_with("/mod.rs") && normalized.starts_with("src/");
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

/// Narrow, file-local proof for this file's own public function, addressed by name.
///
/// `structure/tests.rs` is a separate physical file whose behavioural suite this does not repeat
/// or replace. `check-test-coverage`'s Rust front end keys a test's companion unit off the literal
/// file it is textually written in, so a test living in that separate file can never address a
/// function declared here, however it is named — this module gives [`Check_No_Mod_Rs_Files`] the
/// one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_No_Mod_Rs_Files_Should_Report_A_Mod_File_Under_Source()
    {
        let findings = Check_No_Mod_Rs_Files(&[Source_At("src/pipeline/mod.rs")]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NO_MOD_RS_FILES));
    }

    /// A valueless source at `path` — this rule reads the path and nothing else.
    fn Source_At(path: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }
}
