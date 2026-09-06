//! What each composed rule reads, declared as data rather than as a branch in the run.
//!
//! A rule is otherwise three things written in three places: a function here, a rule
//! identifier here, and membership in a hand-written mapping in `nomos-check-orchestration`
//! that decides which capability families a selection has to materialize. The third is the
//! one that grows centrally, once per rule, which is what makes adding a rule a change to
//! the composition root rather than an addition beside its siblings.
//!
//! A descriptor is the data a planner would plan from. It deliberately does **not** carry
//! the judgment itself: the composed table holds closures over different captures, and
//! moving those is `P41-RUN-STOPS-NAMING-EVERY-RULE`, where the composition root and this
//! table sit in one territory. What is here is the part that can be declared without moving
//! any code, and proven complete against what the run actually composes.

use nomos_contracts::{CapabilityId, RuleId};

/// What a rule reads to reach a judgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubjectKind
{
    /// The text of each walked source, judged without consulting the fact store.
    SourceText,
    /// Facts filed under each walked source's own subject.
    SourceFacts,
    /// One whole-workspace fact, with no per-source subject of its own.
    Workspace,
}

/// A capability family a rule reads facts from.
///
/// A closed set naming the families this workspace has, rather than a string: each variant
/// resolves through the capability crate that owns the contract, so the identifier is
/// spelled once, where it is declared, and not again here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredFact
{
    SyntaxItems,
    DependencyEdges,
    LintDiagnostics,
    DependencyPolicy,
    Reachability,
    NamingPolicy,
    LimitsPolicy,
    ScriptingPolicy,
    GoalsPolicy,
    WordsPolicy,
    ReviewFindings,
    RequirementTrace,
}

impl RequiredFact
{
    /// The canonical identifier of the capability this family names.
    #[must_use]
    pub fn Capability(self) -> CapabilityId
    {
        return match self
        {
            Self::SyntaxItems => nomos_cap_syntax::Capability(),
            Self::DependencyEdges => nomos_cap_dependency::Capability(),
            Self::LintDiagnostics => nomos_cap_lint::Capability(),
            Self::DependencyPolicy => nomos_cap_dependency_policy::Capability(),
            Self::Reachability => nomos_cap_controlflow::Capability(),
            Self::NamingPolicy => nomos_cap_naming_policy::Capability(),
            Self::LimitsPolicy => nomos_cap_limits_policy::Capability(),
            Self::ScriptingPolicy => nomos_cap_scripting_policy::Capability(),
            Self::GoalsPolicy => nomos_cap_goals_policy::Capability(),
            Self::WordsPolicy => nomos_cap_words_policy::Capability(),
            Self::ReviewFindings => nomos_connector_coderabbit::Capability(),
            Self::RequirementTrace => nomos_cap_requirement_trace::Capability(),
        };
    }
}

/// One rule, and what a run would have to have materialized before it can be judged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleDescriptor
{
    /// The rule identifier, the same string the composed table names.
    pub id: &'static str,
    /// What this rule reads.
    pub subject: SubjectKind,
    /// The capability families this rule needs materialized, in declaration order.
    pub requires: &'static [RequiredFact],
    /// The authority this rule's implementation cites.
    ///
    /// A versioned governing record identifier for the eight rules whose contract a record
    /// decides, [`PORTED_STANDARD`] for a rule ported from code-standards, or
    /// [`WORKSPACE_CONVENTIONS`] for the one rule whose contract is this repository's own
    /// prose. `OD-GATE-020` measured that population and confirmed the two-field shape
    /// carries it.
    ///
    /// Here rather than in `nomos-gate-orchestration`'s own table because that crate's
    /// module doc named the absence directly: "a rule's contract citation is knowledge no
    /// export carries". `OD-RULES-022` decided a declaration carries it, and this is the
    /// export it named.
    pub contract_record: &'static str,
    /// The version of [`Self::contract_record`], or [`NO_VERSIONED_RECORD`] when the
    /// authority is prose and has no version a citation could be right or wrong about.
    pub contract_record_version: u32,
}

impl RuleDescriptor
{
    /// This descriptor's identifier as the kernel's own type.
    #[must_use]
    pub fn Rule(&self) -> RuleId
    {
        return RuleId::New(self.id);
    }

    /// Whether this descriptor's authority carries a version a citation could be wrong about.
    ///
    /// A predicate rather than a comparison against [`NO_VERSIONED_RECORD`] at every call
    /// site: that constant is a sentinel meaning absence, and a caller writing `== 0` is a
    /// caller who has to know that. `OD-GATE-020` accepted the sentinel and said what it
    /// means; this is where it is read.
    #[must_use]
    pub const fn Cites_A_Versioned_Record(&self) -> bool
    {
        return self.contract_record_version != NO_VERSIONED_RECORD;
    }

    /// The same descriptor, citing `record` at `version` instead of the ported standard.
    ///
    /// A builder rather than a fifth parameter on [`Described`]: this workspace's own
    /// `parameter-count` rule caps a function at four value parameters, and a rule that
    /// declares its own limits should not be the first thing to exceed them.
    #[must_use]
    pub const fn Citing(self, record: &'static str, version: u32) -> Self
    {
        return Self { contract_record: record, contract_record_version: version, ..self };
    }
}

/// The authority a rule ported from code-standards cites: the standard itself.
///
/// Every rule carrying this is one whose own module doc opens "code-standards' `<rule-id>`
/// rule", and the rule id in the same descriptor is the document's identity within that
/// standard, so the pair names the contract exactly. Distinct from [`WORKSPACE_CONVENTIONS`]
/// because the two are different documents: nothing in this repository's `README.md` states
/// what `atomic-ordering-choices-are-justified` requires.
pub const PORTED_STANDARD: &str = "code-standards";

/// The authority `Check_Naming_Convention` cites: `README.md`'s Conventions section.
///
/// `naming.rs`'s own "# Why this has no `CONTRACT_RECORD`" section says this in full, and says
/// why no record was manufactured to replace it -- "a record authored only to give this rule
/// something to cite would be the record standing in for the check, not the other way around."
pub const WORKSPACE_CONVENTIONS: &str = "README.md";

/// The version a prose contract is at: none.
///
/// Zero marks the absence, not version zero of something. `OD-GATE-020` accepted this
/// sentinel as the general pattern for a record-less rule rather than a one-off awaiting its
/// own type.
pub const NO_VERSIONED_RECORD: u32 = 0;

/// One descriptor, named so the table below reads as one line per rule.
///
/// Cites [`PORTED_STANDARD`] at [`NO_VERSIONED_RECORD`], which is what all but eight rules in
/// the table cite. The eight that cite something else say so with [`RuleDescriptor::Citing`],
/// so the common case stays one line and the exceptions are visible as exceptions.
const fn Described(id: &'static str, subject: SubjectKind, requires: &'static [RequiredFact]) -> RuleDescriptor
{
    return RuleDescriptor {
        id,
        subject,
        requires,
        contract_record: PORTED_STANDARD,
        contract_record_version: NO_VERSIONED_RECORD,
    };
}

/// Every rule this crate offers, with what it reads.
///
/// The order matches the composed table so the two read side by side. The families beyond
/// syntax are exactly the ones the run's own selection predicates gate: dependency edges,
/// lint diagnostics, dependency policy, reachability, the naming, limits, scripting, goals
/// and words policies, review findings, and requirement trace. Syntax is materialized
/// unconditionally today and so appears in no predicate, which is why it is declared here
/// per rule rather than inferred from one.
///
/// Mirrored by `Test_Every_Composed_Rule_Should_Have_A_Descriptor`, which compares this
/// list against `nomos_check_orchestration::Composed_Rules` in both directions from
/// `tests/contract/tests/rule_descriptors.rs` -- the only crate above both.
pub const DESCRIPTORS: &[RuleDescriptor] = &[
    Described(crate::COMPLETENESS_MIRROR, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]).Citing(crate::CONTRACT_RECORD, crate::CONTRACT_RECORD_VERSION),
    Described(crate::NAMING_CONVENTION, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy]).Citing(WORKSPACE_CONVENTIONS, NO_VERSIONED_RECORD),
    Described(crate::DEPENDENCY_DIRECTION, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges]).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Described(crate::DEPENDENCY_COMPLETENESS, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges]).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Described(crate::WRITE_AUTHORITY, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges]).Citing(crate::WRITE_AUTHORITY_CONTRACT_RECORD, crate::WRITE_AUTHORITY_CONTRACT_RECORD_VERSION),
    Described(crate::LINT_DIAGNOSTICS, SubjectKind::SourceFacts, &[RequiredFact::LintDiagnostics]).Citing(crate::LINT_CONTRACT_RECORD, crate::LINT_CONTRACT_RECORD_VERSION),
    Described(crate::DEPENDENCY_POLICY, SubjectKind::SourceFacts, &[RequiredFact::DependencyPolicy]).Citing(crate::DEPENDENCY_POLICY_CONTRACT_RECORD, crate::DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION),
    Described(crate::UNREAD_REACHES_FINDING, SubjectKind::SourceFacts, &[RequiredFact::Reachability]).Citing(crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD, crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION),
    Described(crate::REVIEW_FINDING, SubjectKind::SourceFacts, &[RequiredFact::ReviewFindings]).Citing(crate::REVIEW_CONTRACT_RECORD, crate::REVIEW_CONTRACT_RECORD_VERSION),
    Described(crate::CROSS_LANGUAGE_CORRESPONDENCE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]).Citing(crate::CROSS_LANGUAGE_CONTRACT_RECORD, crate::CROSS_LANGUAGE_CONTRACT_RECORD_VERSION),
    Described(crate::NO_TRAILING_WHITESPACE, SubjectKind::SourceText, &[]),
    Described(crate::TODO_FORMAT, SubjectKind::SourceText, &[]),
    Described(crate::DEPRECATION, SubjectKind::SourceText, &[]),
    Described(crate::A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, SubjectKind::SourceText, &[]),
    Described(crate::SHARED_INTERIOR_MUTABILITY_SAYS_WHY, SubjectKind::SourceText, &[]),
    Described(crate::EVERY_ALLOW_CARRIES_A_JUSTIFICATION, SubjectKind::SourceText, &[]),
    Described(crate::UNSAFE_JUSTIFICATION, SubjectKind::SourceText, &[]),
    Described(crate::SCRIPTS_USE_A_PORTABLE_SHEBANG, SubjectKind::SourceText, &[]),
    Described(crate::A_SCRIPT_DECLARES_ITS_PURPOSE, SubjectKind::SourceText, &[]),
    Described(crate::EXECUTED_SCRIPTS_SET_NOUNSET, SubjectKind::SourceText, &[]),
    Described(crate::SLEEP_BASED_SYNCHRONIZATION, SubjectKind::SourceText, &[]),
    Described(crate::ZERO_FLAKE_POLICY, SubjectKind::SourceText, &[]),
    Described(crate::NO_MOD_RS_FILES, SubjectKind::SourceText, &[]),
    Described(crate::A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, SubjectKind::SourceText, &[]),
    Described(crate::A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, SubjectKind::SourceText, &[]),
    Described(crate::CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, SubjectKind::SourceText, &[]),
    Described(crate::A_DISCARDED_ERROR_IS_EXPLAINED, SubjectKind::SourceText, &[]),
    Described(crate::A_SKIPPED_TEST_STATES_WHY, SubjectKind::SourceText, &[]),
    Described(crate::AN_EXCLUDED_FILE_SAYS_WHY, SubjectKind::SourceText, &[]),
    Described(crate::SUPPRESSION_DIRECTIVES_CARRY_A_REASON, SubjectKind::SourceText, &[]),
    Described(crate::WORKSPACE_MARKERS_CARRY_A_REASON, SubjectKind::SourceText, &[]),
    Described(crate::A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, SubjectKind::SourceText, &[]),
    Described(crate::ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, SubjectKind::SourceText, &[]),
    Described(crate::SEQCST_JUSTIFIED_EXPLICITLY, SubjectKind::SourceText, &[]),
    Described(crate::RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, SubjectKind::SourceText, &[]),
    Described(crate::DATA_NAMES_STAY_LOWER_SNAKE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy]),
    Described(crate::FILE_NAME_MATCHES_DECLARED_TYPE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]),
    Described(crate::CONSTANTS_SPLIT_BY_EXPORT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]),
    Described(crate::GO_VARIABLES_USE_LOWER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]),
    Described(crate::EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy]),
    Described(crate::UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy]),
    Described(crate::TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy]),
    Described(crate::PARAMETER_COUNT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy]),
    Described(crate::GO_HELPERS_PACKAGE_FIVE_INPUTS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy]),
    Described(crate::DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, SubjectKind::SourceFacts, &[RequiredFact::ScriptingPolicy]),
    Described(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy]),
    Described(crate::ONE_THOUSAND_LINE_HARD_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy]),
    Described(crate::FIVE_HUNDRED_LINE_REVIEW_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy]),
    Described(crate::LOWERCASE_FIRST_LETTER, SubjectKind::SourceText, &[]),
    Described(crate::NO_TRAILING_PUNCTUATION, SubjectKind::SourceText, &[]),
    Described(crate::EAGER_VS_LAZY_CONTEXT, SubjectKind::SourceText, &[]),
    Described(crate::GOALS_AND_PARTS_LINE_UP, SubjectKind::Workspace, &[RequiredFact::GoalsPolicy]),
    Described(crate::ABBREVIATIONS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::WordsPolicy]),
    Described(crate::SINGLE_LETTER_NAMES, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems]),
    Described(crate::A_DISABLED_TEST_STATES_WHY, SubjectKind::SourceText, &[]),
    Described(crate::INLINE_ALWAYS_JUSTIFICATION, SubjectKind::SourceText, &[]),
    Described(crate::NO_WILDCARD_IMPORTS, SubjectKind::SourceText, &[]),
    Described(crate::NO_SINGLE_LINE_FUNCTION_BODIES, SubjectKind::SourceText, &[]),
    Described(crate::NO_ORPHAN_MODULES, SubjectKind::SourceText, &[]),
    Described(crate::PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN, SubjectKind::SourceText, &[]),
    Described(crate::LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, SubjectKind::SourceText, &[]),
    Described(crate::STATIC_BOUNDS_ARE_JUSTIFIED, SubjectKind::SourceText, &[]),
    Described(crate::PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS, SubjectKind::SourceText, &[]),
    Described(crate::NESTING_DEPTH, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy]),
    Described(crate::CLOSURE_BOUNDS_ARE_MINIMAL, SubjectKind::SourceText, &[]),
    Described(crate::BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS, SubjectKind::SourceText, &[]),
    // The second composed rule that takes no sources: its whole subject is a corpus of
    // committed declaration files compared against the workspace, which arrives through
    // the reader as one already-judged fact, the same way GOALS_AND_PARTS_LINE_UP's
    // declaration does.
    Described(crate::REQUIREMENT_TRACE_STALENESS, SubjectKind::Workspace, &[RequiredFact::RequirementTrace])
        .Citing(crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD, crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION),
];

#[cfg(test)]
mod descriptor_tests
{
    use super::*;
    use std::collections::BTreeSet;

    /// Every descriptor cites either a versioned record or a named prose authority, never a
    /// bare literal and never a prose authority wearing a version.
    ///
    /// Relocated here by `P52` from `nomos-gate-orchestration`'s own `OFFERINGS` table, which
    /// this field replaced. The invariant is about the citation, and the citation now lives
    /// here, which is also the only module where the two prose authorities are in scope.
    #[test]
    fn Test_Every_Descriptor_Should_Cite_A_Real_Authority()
    {
        for descriptor in DESCRIPTORS
        {
            assert!(!descriptor.contract_record.is_empty(), "{} cites an empty contract record", descriptor.id);

            let is_prose = descriptor.contract_record == PORTED_STANDARD || descriptor.contract_record == WORKSPACE_CONVENTIONS;
            assert_eq!(
                is_prose,
                !descriptor.Cites_A_Versioned_Record(),
                "{} cites {} at version {}: a prose authority carries no version for a citation                  to be right or wrong about, and a record always has one",
                descriptor.id,
                descriptor.contract_record,
                descriptor.contract_record_version
            );
        }
    }

    #[test]
    fn Test_No_Rule_Should_Be_Described_Twice()
    {
        let unique: BTreeSet<&str> = DESCRIPTORS.iter().map(|descriptor| return descriptor.id).collect();

        assert_eq!(unique.len(), DESCRIPTORS.len(), "a rule identifier is described more than once");
    }

    #[test]
    fn Test_A_Source_Text_Rule_Should_Require_No_Fact()
    {
        for descriptor in DESCRIPTORS.iter().filter(|described| return described.subject == SubjectKind::SourceText)
        {
            assert!(
                descriptor.requires.is_empty(),
                "{} judges source text and cannot also need a materialized fact",
                descriptor.id
            );
        }
    }

    #[test]
    fn Test_A_Fact_Reading_Rule_Should_Require_At_Least_One_Fact()
    {
        for descriptor in DESCRIPTORS.iter().filter(|described| return described.subject != SubjectKind::SourceText)
        {
            assert!(!descriptor.requires.is_empty(), "{} reads facts and must say which", descriptor.id);
        }
    }

    #[test]
    fn Test_Every_Required_Family_Should_Resolve_To_A_Capability()
    {
        for descriptor in DESCRIPTORS
        {
            for required in descriptor.requires
            {
                let _resolved = required.Capability();
            }
        }
    }
}
