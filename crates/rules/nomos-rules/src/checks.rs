//! The thirty-eight rules this crate implements, one module each — `naming` holds thirteen,
//! `rust_text` holds four, `structure` holds five, `formatting` holds four, and `dependency` holds two,
//! while `script_discipline` holds two,
//! [`Check_Dependency_Direction`] and [`Check_Every_Member_Declares_A_Band`], since both
//! judge the same declared architecture and observed `nomos.cap.dependency.edges` fact,
//! while naming holds the general function convention and the test-name behavior
//! convention.
//! `lib.rs`'s own module doc walks why each one exists and in what order it was built;
//! this file only gathers them so the crate root is not itself the ninth thing that
//! grows one module per rule forever.
//!
//! [`Relay_Findings`] and [`test_support`] are the two pieces of shared plumbing more than
//! one rule needed by hand before this file existed: [`lint`] and [`policy`] both relay a
//! `ToolProvider`'s own verdict 1:1 rather than judging it a second time, and every rule's
//! test module was separately rebuilding the registry/store/fact scaffolding
//! [`test_support`] now states once.

mod crosslang;
mod dependency;
mod formatting;
mod function_shape;
mod lint;
mod mirror;
mod naming;
mod policy;
mod reachability;
mod role_surface_pair;
mod rust_text;
mod script_discipline;
mod structure;
#[cfg(test)]
mod test_support;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

pub use crosslang::{
    Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CONTRACT_RECORD, CROSS_LANGUAGE_CONTRACT_RECORD_VERSION,
    CROSS_LANGUAGE_CORRESPONDENCE,
};
pub use dependency::{
    Check_Dependency_Direction, Check_Every_Member_Declares_A_Band, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
};
pub use formatting::{
    Check_Deprecation_Carries_A_Reason, Check_No_Decorative_Section_Dividers, Check_No_Trailing_Whitespace,
    Check_Todo_Format, DEPRECATION, NO_DECORATIVE_SECTION_DIVIDERS, NO_TRAILING_WHITESPACE, TODO_FORMAT,
};
pub use function_shape::{
    Check_Function_Arity_Policy, Check_Go_Helpers_Package_Five_Inputs, Check_Parameter_Count, FunctionArityPolicy,
    FunctionAritySource, ReceiverAllowance, GO_HELPERS_PACKAGE_FIVE_INPUTS, PARAMETER_COUNT,
};
pub use lint::{Check_Lint_Diagnostics, LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION, LINT_DIAGNOSTICS};
pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION};
pub use naming::{
    Check_Boolean_Predicates, Check_Data_Names_Stay_Lower_Snake, Check_File_Name_Matches_Declared_Type,
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_Go_Constants_Split_By_Export, Check_Go_Type_Names_Use_Camel_Case,
    Check_Go_Variables_Use_Lower_Snake_Case, Check_Naming_Convention, Check_One_Public_Type_Per_File,
    Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
    Check_Single_Letter_Names, Check_Test_Names_Describe_Behavior, Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    BOOLEAN_PREDICATES, CONSTANTS_SPLIT_BY_EXPORT, DATA_NAMES_STAY_LOWER_SNAKE,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE, GO_VARIABLES_USE_LOWER_SNAKE_CASE,
    NAMING_CONVENTION,
    ONE_PUBLIC_TYPE_PER_FILE, PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE, SINGLE_LETTER_NAMES,
    TEST_NAME_DESCRIBES_BEHAVIOR, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
};
pub use policy::{
    Check_Dependency_Policy, DEPENDENCY_POLICY, DEPENDENCY_POLICY_CONTRACT_RECORD,
    DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION,
};
pub use reachability::{
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
pub use role_surface_pair::{Check_Declared_Role_Matches_Surface, RoleSurfacePair, DECLARED_ROLE_MATCHES_SURFACE};
pub use rust_text::{
    Check_A_Rust_Path_Stays_Within_Its_Own_Subtree, Check_Panics_Are_Justified_Documented_And_Validated,
    Check_Shared_Interior_Mutability_Says_Why, Check_Unwrap_Expect_Discipline,
    A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED,
    SHARED_INTERIOR_MUTABILITY_SAYS_WHY, UNWRAP_EXPECT_DISCIPLINE,
};
pub use script_discipline::{
    Check_A_Script_Declares_Its_Purpose, Check_Scripts_Use_A_Portable_Shebang, A_SCRIPT_DECLARES_ITS_PURPOSE,
    SCRIPTS_USE_A_PORTABLE_SHEBANG,
};
pub use structure::{
    Check_File_Size_Justification_Trigger, Check_File_Size_Review_Trigger, Check_Go_File_Size_Hard_Trigger,
    Check_Go_File_Size_Review_Trigger, Check_No_Mod_Rs_Files,
    FILE_SIZE_JUSTIFICATION_TRIGGER, FILE_SIZE_REVIEW_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
    NO_MOD_RS_FILES, ONE_THOUSAND_LINE_HARD_TRIGGER,
};

/// Requires and judges one payload per source, the identical "read the fact, judge the
/// payload, sort by subject then summary" shape [`lint`] and [`policy`] each rebuilt by
/// hand for their own payload type: since a `ToolProvider`'s own verdict is already a
/// judgment, both rules relay it 1:1 rather than reaching a second opinion, and the only
/// thing that differs between them is how a payload is read and how it becomes findings —
/// exactly the two functions this takes rather than reimplements.
pub(crate) fn Relay_Findings<Payload>(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    mut payload_of: impl FnMut(&SourceFile, &mut dyn FactReader) -> Result<Payload, Finding>,
    mut findings_of: impl FnMut(&SourceFile, &Payload) -> Vec<Finding>,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match payload_of(source, &mut *facts)
        {
            Ok(payload) =>
            {
                let source_findings = findings_of(source, &payload);
                findings.extend(source_findings);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
    return findings;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::Registry;

    fn Source(path: &str) -> SourceFile
    {
        use nomos_contracts::SubjectId;
        use nomos_model::Content_Digest;

        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }

    #[test]
    fn Test_Relay_Findings_Should_Push_The_Payload_Error_And_Extend_The_Findings_Of_Success()
    {
        let sources = vec![Source("a.rs"), Source("b.rs")];
        let IdleReader { registry, store } = Idle_Reader();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());

        let findings = Relay_Findings(
            &sources,
            &mut facts,
            |source, _facts| {
                if source.path == "a.rs"
                {
                    return Err(Finding {
                        rule: nomos_contracts::RuleId::New("example"),
                        subject: source.subject,
                        subject_name: source.path.clone(),
                        applicability: nomos_contracts::Applicability::DependencyUnavailable,
                        evidence: nomos_contracts::EvidenceClass::Derived,
                        gate: nomos_contracts::GateCategory::Advisory,
                        summary: "no fact for a.rs".to_owned(),
                        locations: vec![source.path.clone()],
                    });
                }
                return Ok(2u32);
            },
            |source, payload| {
                return (0..*payload)
                    .map(|index| {
                        return Finding {
                            rule: nomos_contracts::RuleId::New("example"),
                            subject: source.subject,
                            subject_name: format!("{}#{index}", source.path),
                            applicability: nomos_contracts::Applicability::Supported,
                            evidence: nomos_contracts::EvidenceClass::Derived,
                            gate: nomos_contracts::GateCategory::Advisory,
                            summary: "relayed".to_owned(),
                            locations: vec![source.path.clone()],
                        };
                    })
                    .collect();
            },
        );

        assert_eq!(findings.len(), 3, "one pushed error plus two relayed findings: {findings:?}");
        assert!(findings.iter().any(|finding| return finding.summary == "no fact for a.rs"));
        assert_eq!(findings.iter().filter(|finding| return finding.summary == "relayed").count(), 2);
    }

    /// An admitted registry and an empty store — named so a call site reads
    /// `reader.registry`, not a position it has to count.
    struct IdleReader
    {
        registry: Registry,
        store: MemoryFactStore,
    }

    /// An admitted registry and an empty store — `Relay_Findings` never actually calls
    /// `Require` itself (its closures do), so what this reader offers is beside the
    /// point; it only has to be a real `FactReader`, the same view every real caller has.
    fn Idle_Reader() -> IdleReader
    {
        return IdleReader { registry: Registry::New(), store: MemoryFactStore::New() };
    }
}
