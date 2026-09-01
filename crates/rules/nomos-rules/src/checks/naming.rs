//! A function's own name must match this workspace's `Pascal_Snake_Case` convention.
//!
//! `README.md`'s Conventions section states the rule directly: "Function names are
//! `Pascal_Snake_Case`... Types stay `UpperCamelCase`." `Cargo.toml`'s
//! `[workspace.lints.rust]` names the resulting gap in its own comment — the one rustc
//! lint that would have caught a deviation from Rust's *own* convention, `non_snake_case`,
//! is turned off "because... this is the workspace convention, not an oversight" — and
//! nothing was turned on to check the convention that replaced it. This rule is that check.
//!
//! # A second rule, deliberately different in shape from the first
//!
//! [`crate::Check_Completeness_Mirrors`] reconciles a claim embedded in one place (a doc
//! comment) against a fact derived from another (whether a name it claims exists). This
//! rule is a one-sided lexical predicate over each function's own name, stated once in
//! prose rather than declared per-subject — no claimed mirror, no cross-file resolution,
//! no [`crate::Reading::Unobserved`] handling, because nothing here depends on
//! documentation being observed at all. `OD-PACKAGE-008` asked whether a `RulePackage`
//! manifest's field boundaries generalize past a population of one rule; this is the
//! second data point that question needs.
//!
//! # Why this has no `CONTRACT_RECORD`
//!
//! Unlike [`crate::Check_Completeness_Mirrors`], whose contract is `D-134` — a versioned
//! governing record `tests/contract/tests/rule_contract_citation.rs` checks against — this
//! rule's contract is `README.md`'s Conventions section: prose, not a record with a
//! `version:` front-matter field that same mechanism could read. `PKG-014`'s traceability
//! requirement is accordingly not mechanically checked for this rule, the same way it was
//! not checked for the first one before that citation existed. Left named rather than
//! manufactured — a record authored only to give this rule something to cite would be the
//! record standing in for the check, not the other way around.
//!
//! # Split by responsibility
//!
//! [`reading`] requires and decodes one file's own syntax fact. [`violations`] judges an
//! already-decoded payload against the naming convention. This file keeps only what
//! composes the two: the rule's own identifier and [`Check_Naming_Convention`] itself, plus
//! the end-to-end tests that exercise both together through a real reader.

mod data_names;
mod reading;
mod test_names;
mod violations;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

pub use data_names::{Check_Data_Names_Stay_Lower_Snake, DATA_NAMES_STAY_LOWER_SNAKE};
pub use test_names::{Check_Test_Names_Describe_Behavior, TEST_NAME_DESCRIBES_BEHAVIOR};

/// This rule's own identifier.
pub const NAMING_CONVENTION: &str = "function-naming-convention";
/// The code-standards identifier for this workspace's function naming convention.
pub const PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE: &str = "project-owned-function-names-use-upper-snake-case";

/// Judges every function `sources` declares against the workspace's naming convention.
///
/// One fact per file, the same shape [`crate::Check_Completeness_Mirrors`] reads — this
/// rule states the same [`crate::Syntax_Requirement`] floor for the same reason: a name
/// spelled inside a comment or a string literal must not resolve as a real declaration,
/// and nothing weaker than a sound parse can promise that.
#[must_use]
pub fn Check_Naming_Convention(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    use reading::Payload_Of;
    use violations::Violations_In;

    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Judges the same function-name convention under the code-standards rule id.
#[must_use]
pub fn Check_Project_Owned_Function_Names_Use_Upper_Snake_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Check_Naming_Convention(sources, facts);

    for finding in &mut findings
    {
        finding.rule = nomos_contracts::RuleId::New(PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE);
        if finding.applicability == nomos_contracts::Applicability::Supported
        {
            finding.gate = nomos_contracts::GateCategory::Blocking;
        }
    }

    return findings;
}

/// [`Check_Naming_Convention`] itself, through a real registry, store and reader — the
/// half [`violations::tests`] does not reach, because a fact has to be required and
/// decoded before there is a payload to judge at all.
#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
    use nomos_model::Content_Digest;

    const PARSER: &str = "nomos.test.naming.parses";

    #[test]
    fn Test_Payload_Of_Should_Read_And_Judge_A_Real_Fact()
    {
        let source = Source_File(Path("src/lib.rs"), Text("fn bad_name() {}"));
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Naming_Convention(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "bad_name");
    }

    /// `path` and `text` are both `&str`; without a distinct type per position a call site
    /// like `Source_File("src/lib.rs", "fn bad_name() {}")` reads as two interchangeable
    /// strings and a swap compiles silently. These wrappers give each position a type the
    /// other cannot satisfy.
    #[derive(Clone, Copy)]
    struct Path<'a>(&'a str);

    #[derive(Clone, Copy)]
    struct Text<'a>(&'a str);

    fn Materialize_Syntax_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &str)
    {
        let inputs = InputDigest::Of(&[source.text.as_bytes()]);
        test_support::Materialize(store, source.subject, offer, inputs, nomos_cap_syntax::Payload_Schema(), payload.as_bytes().to_vec());
    }

    #[test]
    fn Test_Check_Naming_Convention_Should_Report_A_Subject_With_No_Fact_Rather_Than_Silently_Clean()
    {
        let source = Source_File(Path("src/lib.rs"), Text("fn bad_name() {}"));
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Naming_Convention(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").subject_name,
            "src/lib.rs"
        );
    }

    #[test]
    fn Test_Check_Project_Owned_Function_Names_Use_Upper_Snake_Case_Should_Report_Under_The_Code_Standards_Id()
    {
        let source = Source_File(Path("src/lib.rs"), Text("fn bad_name() {}"));
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Project_Owned_Function_Names_Use_Upper_Snake_Case(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, nomos_contracts::RuleId::New(PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE));
        assert_eq!(found.gate, nomos_contracts::GateCategory::Blocking);
    }

    #[test]
    fn Test_Check_Test_Names_Describe_Behavior_Should_Read_And_Judge_A_Real_Fact()
    {
        let source = Source_File(Path("src/lib.rs"), Text("#[test]\nfn Test_Insert_Works() {}"));
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tFunction\tPrivate\tTest_Insert_Works\t.\t+fn/0\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Test_Names_Describe_Behavior(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Test_Insert_Works");
    }

    #[test]
    fn Test_Check_Data_Names_Stay_Lower_Snake_Should_Read_And_Judge_A_Real_Fact()
    {
        let source = Source_File(Path("src/lib.rs"), Text("mod BadModule {}"));
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Syntax_Fact(
            &mut store,
            &source,
            &offer,
            "unexpanded\t0\nitem\t0\tModule\tPrivate\tBadModule\t.\t.\n",
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Data_Names_Stay_Lower_Snake(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadModule");
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Source_File(path: Path<'_>, text: Text<'_>) -> SourceFile
    {
        return SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_syntax::Capability_Contract(),
            nomos_cap_syntax::Capability(),
            nomos_cap_syntax::CONTRACT_VERSION,
            PARSER,
            Guarantee_At_Floor(),
        );
    }
}
