//! The sixty-six rules this crate implements, one module each — `naming` holds fourteen,
//! `rust_text` holds eight, `go_text` holds five, `structure` holds five, `formatting` holds
//! five, `security_text` holds three, `concurrency_text` holds three, `error_text` holds
//! three, `facade` holds three, `placement` holds two, `dependency` holds two, and `goals`
//! holds the one rule here whose subject is not source at all, while
//! `script_discipline` holds four,
//! [`Check_Dependency_Direction`] and [`Check_Every_Member_Declares_A_Band`],
//! since both judge the same declared architecture and observed
//! `nomos.cap.dependency.edges` fact, while naming holds the general function convention
//! and the test-name behavior convention.
//! `lib.rs`'s own module doc walks why each one exists and in what order it was built;
//! this file only gathers them so the crate root is not itself the ninth thing that
//! grows one module per rule forever.
//!
//! [`Relay_Findings`] and [`test_support`] are the two pieces of shared plumbing more than
//! one rule needed by hand before this file existed: [`lint`] and [`policy`] both relay a
//! `ToolProvider`'s own verdict 1:1 rather than judging it a second time, and every rule's
//! test module was separately rebuilding the registry/store/fact scaffolding
//! [`test_support`] now states once.

mod concurrency_text;
mod constant_scope;
mod crosslang;
mod dependency;
mod domain_type_alias;
mod enum_shape;
mod error_text;
mod facade;
mod flakiness_text;
mod formatting;
mod goals;
mod function_shape;
mod go_text;
mod lint;
mod mirror;
mod naming;
mod orphan_modules;
mod placement;
mod policy;
mod reachability;
mod role_surface_pair;
mod rust_text;
mod scalar_range;
mod script_discipline;
mod security_text;
mod structure;
#[cfg(test)]
mod test_support;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

pub use concurrency_text::{
    Check_Atomic_Ordering_Choices_Are_Justified, Check_Relaxed_Not_Used_When_Ordering_Matters,
    Check_Seqcst_Justified_Explicitly, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS,
    SEQCST_JUSTIFIED_EXPLICITLY,
};
pub use constant_scope::{Check_Constants_Are_The_Exception_To_Function_Scope_Use, CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE};
pub use crosslang::{
    Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CONTRACT_RECORD, CROSS_LANGUAGE_CONTRACT_RECORD_VERSION,
    CROSS_LANGUAGE_CORRESPONDENCE,
};
pub use dependency::{
    Check_Dependency_Direction, Check_Every_Member_Declares_A_Band, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
};
pub use domain_type_alias::{Check_Domain_Values_Are_Distinct_Types, DOMAIN_VALUES_ARE_DISTINCT_TYPES};
pub use enum_shape::{Check_Named_Fields_Over_Positional_Variant_Payloads, NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS};
pub use error_text::{
    Check_Eager_Vs_Lazy_Context, Check_Error_Message_Has_No_Trailing_Punctuation, Check_Error_Message_Starts_Lowercase,
    EAGER_VS_LAZY_CONTEXT, LOWERCASE_FIRST_LETTER, NO_TRAILING_PUNCTUATION,
};
pub use facade::{
    Check_A_Consumer_Imports_Through_The_Facade, Check_A_Facade_Publishes_A_Child_One_Way,
    Check_A_Renamed_Facade_Re_Export_Names_The_Contract, FACADE_ALIASES_NAME_THE_CONTRACT,
    FACADE_CHOOSES_FLATTENING_OR_NAMESPACE, FACADE_CONSUMERS_USE_THE_FACADE_PATH,
};
pub use flakiness_text::{
    Check_A_Test_Does_Not_Retry_Until_Green, Check_Sleep_Is_Not_Synchronization, SLEEP_BASED_SYNCHRONIZATION, ZERO_FLAKE_POLICY,
};
pub use formatting::{
    Check_Deprecation_Carries_A_Reason, Check_No_Decorative_Section_Dividers, Check_No_Single_Line_Function_Bodies,
    Check_No_Trailing_Whitespace, Check_Todo_Format, DEPRECATION, NO_DECORATIVE_SECTION_DIVIDERS,
    NO_SINGLE_LINE_FUNCTION_BODIES, NO_TRAILING_WHITESPACE, TODO_FORMAT,
};
pub use function_shape::{
    Check_Function_Arity_Policy, Check_Go_Helpers_Package_Five_Inputs, Check_Parameter_Count, FunctionArityPolicy,
    FunctionAritySource, ReceiverAllowance, GO_HELPERS_PACKAGE_FIVE_INPUTS, PARAMETER_COUNT,
};
pub use go_text::{
    Check_A_Discarded_Error_Is_Explained, Check_A_Skipped_Test_States_Why, Check_An_Excluded_File_Says_Why,
    Check_Suppression_Directives_Carry_A_Reason, Check_Workspace_Markers_Carry_A_Reason,
    A_DISCARDED_ERROR_IS_EXPLAINED, A_SKIPPED_TEST_STATES_WHY, AN_EXCLUDED_FILE_SAYS_WHY,
    SUPPRESSION_DIRECTIVES_CARRY_A_REASON, WORKSPACE_MARKERS_CARRY_A_REASON,
};
pub use goals::{Check_Goals_And_Parts_Line_Up, GOALS_AND_PARTS_LINE_UP};
pub use lint::{Check_Lint_Diagnostics, LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION, LINT_DIAGNOSTICS};
pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION};
pub use naming::{
    Check_Abbreviations, Check_Boolean_Predicates, Check_Data_Names_Stay_Lower_Snake, Check_File_Name_Matches_Declared_Type,
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_Go_Constants_Split_By_Export, Check_Go_Type_Names_Use_Camel_Case,
    Check_Go_Variables_Use_Lower_Snake_Case, Check_Naming_Clarity, Check_Naming_Convention, Check_One_Public_Type_Per_File,
    Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
    Check_Single_Letter_Names, Check_Test_Names_Describe_Behavior, Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    ABBREVIATIONS, BOOLEAN_PREDICATES, CONSTANTS_SPLIT_BY_EXPORT, DATA_NAMES_STAY_LOWER_SNAKE,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE, GO_VARIABLES_USE_LOWER_SNAKE_CASE,
    NAMING_CLARITY, NAMING_CONVENTION,
    ONE_PUBLIC_TYPE_PER_FILE, PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE, SINGLE_LETTER_NAMES,
    TEST_NAME_DESCRIBES_BEHAVIOR, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
};
pub use orphan_modules::{Check_No_Orphan_Modules, NO_ORPHAN_MODULES};
pub use placement::{
    Check_A_Package_Is_Named_After_Its_Directory, Check_No_Wildcard_Imports, A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
    NO_WILDCARD_IMPORTS,
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
    Check_A_Disabled_Test_States_Why, Check_A_Rust_Path_Stays_Within_Its_Own_Subtree,
    Check_Every_Allow_Carries_A_Justification, Check_Inline_Always_Justification,
    Check_Panics_Are_Justified_Documented_And_Validated, Check_Shared_Interior_Mutability_Says_Why,
    Check_Unsafe_Justification, Check_Unwrap_Expect_Discipline,
    A_DISABLED_TEST_STATES_WHY, A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, EVERY_ALLOW_CARRIES_A_JUSTIFICATION,
    INLINE_ALWAYS_JUSTIFICATION, PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED, SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
    UNSAFE_JUSTIFICATION, UNWRAP_EXPECT_DISCIPLINE,
};
pub use scalar_range::{
    Check_A_Known_Range_Picks_Its_Type, Check_Nonnegative_Storage_Is_Unsigned, A_KNOWN_RANGE_PICKS_ITS_TYPE, NONNEGATIVE_STORAGE_IS_UNSIGNED,
};
pub use script_discipline::{
    Check_A_Script_Declares_Its_Purpose, Check_Declared_Tooling_Language_For_Scripts, Check_Executed_Scripts_Set_Nounset,
    Check_Scripts_Use_A_Portable_Shebang,
    A_SCRIPT_DECLARES_ITS_PURPOSE, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, EXECUTED_SCRIPTS_SET_NOUNSET,
    SCRIPTS_USE_A_PORTABLE_SHEBANG,
};
pub use security_text::{
    Check_A_Credential_Is_Not_Hardcoded_In_Source, Check_A_Secret_Does_Not_Travel_In_A_Url,
    Check_Certificate_Verification_Is_Not_Disabled, A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE,
    A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, CERTIFICATE_VERIFICATION_IS_NOT_DISABLED,
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

/// Whether a source sits in the part of a repository that holds tests and examples rather
/// than the code they exercise.
///
/// A repository-layout convention, not a language or provider fact: it answers where a file
/// sits, never what it is written in, which is why `OD-RULES-014` measured the two verbatim
/// copies this replaces and refused to fold them into the carried-language question it was
/// deciding. Housed here for the reason [`Relay_Findings`] is — plumbing more than one rule
/// needed by hand — rather than in either module that used to hold a copy: a leaf reaching
/// into a sibling leaf for it would make the borrower structurally downstream of the lender
/// over a fact neither one owns.
///
/// The prefixes match a workspace-relative path, the infixes a crate-relative one, and the
/// suffixes the in-file convention; slashes are normalized first so a Windows path answers
/// the same as a POSIX one. A bare `tests.rs` and a bare `test_support.rs` are matched by
/// their whole names rather than by a wider rule: `_tests.rs` will not catch the first,
/// because this workspace names an inline test module's file plainly and a suffix wide enough
/// to reach it would also reach `contests.rs`; and a `test_` filename prefix would catch the
/// second at the cost of also catching `checks/naming/test_names.rs`, which implements the
/// test-naming rules and is production code that merely talks about tests. Both names are
/// declared `#[cfg(test)]` at every site in this workspace that declares them, which is the
/// fact a path predicate cannot read and these two clauses stand in for. `security_text`'s own `Is_Test_Or_Fixture_Source` is
/// deliberately not folded in: it admits `/testdata/`, `/fixtures/` and `_test.go` besides,
/// and reads `examples` as an infix rather than a prefix, so it is a different predicate
/// that resembles this one rather than a third copy of it.
pub(crate) fn Is_Test_Or_Example_Source(source: &SourceFile) -> bool
{
    let normalized = source.path.replace('\\', "/");
    return normalized.starts_with("tests/")
        || normalized.starts_with("examples/")
        || normalized.contains("/tests/")
        || normalized.contains("/test/")
        || normalized.ends_with("_test.rs")
        || normalized.ends_with("_tests.rs")
        || normalized.ends_with("/tests.rs")
        || normalized == "tests.rs"
        || normalized.ends_with("/test_support.rs")
        || normalized == "test_support.rs";
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

    /// One case per clause, plus the shapes that must stay out: a production path that merely
    /// contains the word, and `protests.rs`, which a plain `tests.rs` suffix would swallow and
    /// the separator-anchored clause does not. A Windows-separated path appears twice, which
    /// is the reason the predicate normalizes before it compares rather than after, and a bare
    /// `tests.rs` covers the spelling that has no separator to anchor on at all.
    /// `test_names.rs` is the case that keeps the `test_support.rs` clause an exact name: it
    /// is a rule implementation, and the obvious `test_` prefix would exempt it.
    #[test]
    fn Test_Is_Test_Or_Example_Source_Should_Judge_Prefixes_Infixes_And_Suffixes()
    {
        let inside = [
            "tests/contract/main.rs",
            "examples/one.rs",
            "crates/rules/nomos-rules/tests/fixtures.rs",
            "crates/rules/nomos-rules/test/fixtures.rs",
            "crates/rules/nomos-rules/src/thing_test.rs",
            "crates/rules/nomos-rules/src/thing_tests.rs",
            "crates/languages/nomos-lang-go-modules/src/discovery/tests.rs",
            "crates/rules/nomos-rules/src/checks/test_support.rs",
            "tests.rs",
            "test_support.rs",
            "crates\\rules\\nomos-rules\\src\\discovery\\tests.rs",
            "crates\\rules\\nomos-rules\\tests\\fixtures.rs",
        ];
        let outside = [
            "crates/rules/nomos-rules/src/checks.rs",
            "crates/latest/src/contests.rs",
            "crates/rules/src/testing.rs",
            "crates/rules/nomos-rules/src/protests.rs",
            "crates/rules/nomos-rules/src/checks/naming/test_names.rs",
        ];

        for path in inside
        {
            assert!(Is_Test_Or_Example_Source(&Source(path)), "should be exempt: {path}");
        }
        for path in outside
        {
            assert!(!Is_Test_Or_Example_Source(&Source(path)), "should be judged: {path}");
        }
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
