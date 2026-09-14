//! Every rule this crate offers: its identifier, what it reads, the authority it answers
//! to, and the judgment itself -- declared as one table rather than as a branch in the run.
//!
//! The order of this table is the order a run executes in, because the run reads this table.
//!
//! # Why the judgment is here
//!
//! It was not, until `OD-RULES-027`. A rule used to be written in two places -- a descriptor
//! here and an entry in a seventy-line array in `nomos-check-orchestration` pairing the same
//! identifier with a closure -- and a declaration split across two artifacts is a declaration
//! that can go out of step. `OD-GATE-020` measured exactly that happening twice, silently,
//! against the gate registry's own copy of the same list.
//!
//! What kept the judgment out was that the composed array held seventy closures over
//! different captures, and an array needs one element type. `OD-RULES-027` censused the
//! captures instead of assuming them: sixty-two of seventy closed over the same walked
//! sources, and three of the four closure shapes differ only in which of two arguments they
//! ignore. So one signature serves them all, it is a plain `fn` pointer, and a `fn` pointer
//! is `const`-compatible -- which is what lets this stay a table while carrying the code.
//!
//! # What is still declared above this crate, and why
//!
//! Two mappings from rule to something, both in `nomos-check-orchestration`, both fixed
//! hand-written declarations of the kind `OD-GATE-017` accepted rather than the demand
//! planner `OD-RULES-009` has declined: which six rules are judged over a capability
//! family's own materialized slice instead of the walked sources, and which rules feed a
//! capability and so oblige a selection to materialize it. Neither can live here. A
//! capability slice and a materialization are orchestration concepts, and a descriptor table
//! naming one would be a lower band describing an upper band's shape.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::{CapabilityId, Finding, RuleId};

/// A rule's own judgment, in the one shape every rule can be called through.
///
/// A plain `fn` pointer rather than a trait object or a closure, because [`DESCRIPTORS`] is
/// a `const` and a `fn` pointer is the only callable a `const` can hold. That is what makes
/// the table below the whole declaration of a rule rather than half of one, and it is what
/// lets `nomos_check_orchestration` derive its run from this list instead of writing a
/// second copy of it by hand -- `OD-RULES-027`.
///
/// Both parameters are taken by every rule and read by most. A rule that judges only source
/// text is widened here with a closure that ignores the reader, and the two rules whose
/// whole subject arrives through the reader are widened with one that ignores the sources;
/// neither wrapper decides anything, which is why they are spelled inline in the table
/// rather than given names of their own.
///
/// The sources a rule is handed are not always the walked ones. Six rules read a capability
/// family's own materialized slice instead, and which six is `nomos_check_orchestration`'s
/// to say, not this table's: a capability slice is an orchestration concept, and a
/// descriptor naming one would be a lower band describing an upper band's shape.
/// `OD-RULES-027` decided that split and why the mapping that remains there is a
/// declaration rather than the demand planner `OD-RULES-009` declines.
pub type RuleJudgment = fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>;

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
    ArchitectureDeclaration,
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
            Self::ArchitectureDeclaration => nomos_cap_architecture::Capability(),
        };
    }
}

/// One rule: what it reads, what a run must have materialized before it can be judged, the
/// authority it answers to, and the judgment itself.
///
/// # Why this carries no `PartialEq`
///
/// It used to, unused. [`Self::check`] is a `fn` pointer, and comparing two of those
/// compares addresses -- which the compiler is free to merge across distinct functions and
/// to duplicate across codegen units, so a derived `==` here would answer a question it
/// cannot actually answer, silently and in whichever direction the build happened to land.
/// Nothing in this workspace compared two descriptors, so the derive is gone rather than
/// hand-written to skip the field.
#[derive(Clone, Copy, Debug)]
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
    /// What this rule does when it is run.
    ///
    /// Here rather than in a composition root's own array because a rule declared in one
    /// place and judged in another is two declarations, and this workspace has already
    /// measured that pair going silently out of step twice (`OD-GATE-020`). A run derives
    /// its table from this field; see [`RuleJudgment`] for the one shape they share.
    pub check: RuleJudgment,
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
const fn Described(id: &'static str, subject: SubjectKind, requires: &'static [RequiredFact], check: RuleJudgment) -> RuleDescriptor
{
    return RuleDescriptor {
        id,
        subject,
        requires,
        check,
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
    Described(crate::COMPLETENESS_MIRROR, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Completeness_Mirrors).Citing(crate::CONTRACT_RECORD, crate::CONTRACT_RECORD_VERSION),
    Described(crate::NAMING_CONVENTION, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Naming_Convention).Citing(WORKSPACE_CONVENTIONS, NO_VERSIONED_RECORD),
    Described(crate::DEPENDENCY_DIRECTION, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Dependency_Direction).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Described(crate::DEPENDENCY_COMPLETENESS, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Every_Member_Declares_A_Band).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Described(crate::WRITE_AUTHORITY, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Write_Authority).Citing(crate::WRITE_AUTHORITY_CONTRACT_RECORD, crate::WRITE_AUTHORITY_CONTRACT_RECORD_VERSION),
    Described(crate::LINT_DIAGNOSTICS, SubjectKind::SourceFacts, &[RequiredFact::LintDiagnostics], crate::Check_Lint_Diagnostics).Citing(crate::LINT_CONTRACT_RECORD, crate::LINT_CONTRACT_RECORD_VERSION),
    Described(crate::DEPENDENCY_POLICY, SubjectKind::SourceFacts, &[RequiredFact::DependencyPolicy], crate::Check_Dependency_Policy).Citing(crate::DEPENDENCY_POLICY_CONTRACT_RECORD, crate::DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION),
    Described(crate::GUARANTEE_DECLARES_ITS_EXERCISER, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Guarantee_Declares_Its_Exerciser).Citing(crate::GUARANTEE_EXERCISER_CONTRACT_RECORD, crate::GUARANTEE_EXERCISER_CONTRACT_RECORD_VERSION),
    Described(crate::UNREAD_REACHES_FINDING, SubjectKind::SourceFacts, &[RequiredFact::Reachability], crate::Check_Unread_Reaches_A_Finding).Citing(crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD, crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION),
    Described(crate::REVIEW_FINDING, SubjectKind::SourceFacts, &[RequiredFact::ReviewFindings], crate::Check_Review_Findings).Citing(crate::REVIEW_CONTRACT_RECORD, crate::REVIEW_CONTRACT_RECORD_VERSION),
    Described(crate::CROSS_LANGUAGE_CORRESPONDENCE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Cross_Language_Correspondence).Citing(crate::CROSS_LANGUAGE_CONTRACT_RECORD, crate::CROSS_LANGUAGE_CONTRACT_RECORD_VERSION),
    Described(crate::NO_TRAILING_WHITESPACE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Trailing_Whitespace(sources)),
    Described(crate::TODO_FORMAT, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Todo_Format(sources)),
    Described(crate::DEPRECATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Deprecation_Carries_A_Reason(sources)),
    Described(crate::A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources)),
    Described(crate::SHARED_INTERIOR_MUTABILITY_SAYS_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Shared_Interior_Mutability_Says_Why(sources)),
    Described(crate::EVERY_ALLOW_CARRIES_A_JUSTIFICATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Every_Allow_Carries_A_Justification(sources)),
    Described(crate::UNSAFE_JUSTIFICATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Unsafe_Justification(sources)),
    Described(crate::SCRIPTS_USE_A_PORTABLE_SHEBANG, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Scripts_Use_A_Portable_Shebang(sources)),
    Described(crate::A_SCRIPT_DECLARES_ITS_PURPOSE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Script_Declares_Its_Purpose(sources)),
    Described(crate::EXECUTED_SCRIPTS_SET_NOUNSET, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Executed_Scripts_Set_Nounset(sources)),
    Described(crate::SLEEP_BASED_SYNCHRONIZATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Sleep_Is_Not_Synchronization(sources)),
    Described(crate::ZERO_FLAKE_POLICY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Test_Does_Not_Retry_Until_Green(sources)),
    Described(crate::NO_MOD_RS_FILES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Mod_Rs_Files(sources)),
    Described(crate::A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Credential_Is_Not_Hardcoded_In_Source(sources)),
    Described(crate::A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Secret_Does_Not_Travel_In_A_Url(sources)),
    Described(crate::CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Certificate_Verification_Is_Not_Disabled(sources)),
    Described(crate::A_DISCARDED_ERROR_IS_EXPLAINED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Discarded_Error_Is_Explained(sources)),
    Described(crate::A_SKIPPED_TEST_STATES_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Skipped_Test_States_Why(sources)),
    Described(crate::AN_EXCLUDED_FILE_SAYS_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_An_Excluded_File_Says_Why(sources)),
    Described(crate::SUPPRESSION_DIRECTIVES_CARRY_A_REASON, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Suppression_Directives_Carry_A_Reason(sources)),
    Described(crate::WORKSPACE_MARKERS_CARRY_A_REASON, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Workspace_Markers_Carry_A_Reason(sources)),
    Described(crate::A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Package_Is_Named_After_Its_Directory(sources)),
    Described(crate::ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Atomic_Ordering_Choices_Are_Justified(sources)),
    Described(crate::SEQCST_JUSTIFIED_EXPLICITLY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Seqcst_Justified_Explicitly(sources)),
    Described(crate::RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Relaxed_Not_Used_When_Ordering_Matters(sources)),
    Described(crate::DATA_NAMES_STAY_LOWER_SNAKE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Data_Names_Stay_Lower_Snake),
    Described(crate::FILE_NAME_MATCHES_DECLARED_TYPE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_File_Name_Matches_Declared_Type),
    Described(crate::CONSTANTS_SPLIT_BY_EXPORT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Go_Constants_Split_By_Export),
    Described(crate::GO_VARIABLES_USE_LOWER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Go_Variables_Use_Lower_Snake_Case),
    Described(crate::EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Exported_Go_Functions_Use_Upper_Snake_Case),
    Described(crate::UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter),
    Described(crate::TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Go_Type_Names_Use_Camel_Case),
    Described(crate::PARAMETER_COUNT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy], crate::Check_Parameter_Count),
    Described(crate::GO_HELPERS_PACKAGE_FIVE_INPUTS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy], crate::Check_Go_Helpers_Package_Five_Inputs),
    Described(crate::DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, SubjectKind::SourceFacts, &[RequiredFact::ScriptingPolicy], crate::Check_Declared_Tooling_Language_For_Scripts),
    Described(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_File_Size_Justification_Trigger),
    Described(crate::NONNEGATIVE_STORAGE_IS_UNSIGNED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Nonnegative_Storage_Is_Unsigned(sources)),
    Described(crate::A_KNOWN_RANGE_PICKS_ITS_TYPE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Known_Range_Picks_Its_Type(sources)),
    Described(crate::NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Named_Fields_Over_Positional_Variant_Payloads(sources)),
    Described(crate::ONE_THOUSAND_LINE_HARD_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Go_File_Size_Hard_Trigger),
    Described(crate::FIVE_HUNDRED_LINE_REVIEW_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Go_File_Size_Review_Trigger),
    Described(crate::LOWERCASE_FIRST_LETTER, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Error_Message_Starts_Lowercase(sources)),
    Described(crate::NO_TRAILING_PUNCTUATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Error_Message_Has_No_Trailing_Punctuation(sources)),
    Described(crate::EAGER_VS_LAZY_CONTEXT, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Eager_Vs_Lazy_Context(sources)),
    Described(crate::GOALS_AND_PARTS_LINE_UP, SubjectKind::Workspace, &[RequiredFact::GoalsPolicy], |_sources, reader| return crate::Check_Goals_And_Parts_Line_Up(reader)),
    Described(crate::REQUIREMENT_TRACE_STALENESS, SubjectKind::Workspace, &[RequiredFact::RequirementTrace], |_sources, reader| return crate::Check_Requirement_Trace_Staleness(reader)) .Citing(crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD, crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION),
    Described(crate::ABBREVIATIONS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::WordsPolicy], crate::Check_Abbreviations),
    Described(crate::SINGLE_LETTER_NAMES, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Single_Letter_Names),
    Described(crate::A_DISABLED_TEST_STATES_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Disabled_Test_States_Why(sources)),
    Described(crate::INLINE_ALWAYS_JUSTIFICATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Inline_Always_Justification(sources)),
    Described(crate::NO_WILDCARD_IMPORTS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Wildcard_Imports(sources)),
    Described(crate::NO_SINGLE_LINE_FUNCTION_BODIES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Single_Line_Function_Bodies(sources)),
    Described(crate::NO_ORPHAN_MODULES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Orphan_Modules(sources)),
    Described(crate::PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Parameters_Borrow_Unless_Ownership_Is_Taken(sources)),
    Described(crate::LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(sources)),
    Described(crate::STATIC_BOUNDS_ARE_JUSTIFIED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Static_Bounds_Are_Justified(sources)),
    Described(crate::PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Prefer_Macro_Rules_Over_Procedural_Macros(sources)),
    Described(crate::NESTING_DEPTH, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Nesting_Depth),
    Described(crate::CLOSURE_BOUNDS_ARE_MINIMAL, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Closure_Bounds_Are_Minimal(sources)),
    Described(crate::BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths(sources)),
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
