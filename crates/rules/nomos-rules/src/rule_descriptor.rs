//! Every rule a run composes: its identifier, what it reads, the authority it answers to, and
//! the judgment itself -- declared as one table rather than as a branch in the run.
//!
//! The order of this table is the order a run executes in, because the run reads this table.
//!
//! # Which rules, and why the identifier constants outnumber the rows
//!
//! Not every rule this crate implements. Each rule declares its identifier as a string
//! constant under `checks`, and more rules do that than have a row here: the difference is
//! rules written and exported but composed into no run, each held out for a reason
//! `tests/contract/tests/rule_composition.rs` states and fails without. A count of "the rules"
//! is therefore a count of one of two populations, and this table's length answers only for the
//! composed one, which `tests/contract/tests/rule_descriptors.rs` holds equal to the run's own.
//!
//! A declared row counts here exactly as a linked one does, but it is written through
//! [`Descriptor_For_Declaration`] rather than [`Descriptor_For`], so counting either spelling
//! alone undercounts the table.
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
//! [`Descriptor_For`] takes that pointer and [`RuleJudgment::New`] names it, so the pointer
//! appears once here as the boundary it is and nowhere else.
//!
//! # What is still declared above this crate, and why
//!
//! Which rules are judged over a capability family's own materialized slice instead of the
//! walked sources is a mapping from rule to slice in `nomos-check-orchestration`, a fixed
//! hand-written declaration of the kind `OD-GATE-017` accepted rather than the demand planner
//! `OD-RULES-009` has declined. It cannot live here. A capability slice is an orchestration
//! concept, and a descriptor table naming one would be a lower band describing an upper band's
//! shape.
//!
//! Which families a selection obliges a run to materialize was declared there too, until
//! `P102` derived it from this table's own `requires` instead; [`DESCRIPTORS`] says where that
//! derivation is read and what holds it.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::{Finding, RuleId};

pub use population::Population;
pub use required_fact::RequiredFact;
pub use rule_judgment::RuleJudgment;
pub use subject_kind::SubjectKind;

pub(crate) use declared_detector::DeclaredDetector;
pub(crate) use declared_justification::DeclaredJustification;
pub(crate) use declared_parameter::DeclaredParameter;
pub(crate) use declared_text_rule::DeclaredTextRule;
pub(crate) use test_material_sensitivity::TestMaterialSensitivity;

pub(crate) mod policy_axis;

mod declared_detector;
mod declared_justification;
mod declared_parameter;
mod declared_text_rule;
mod population;
mod required_fact;
mod rule_judgment;
mod subject_kind;
mod test_material_sensitivity;

/// One rule: what it reads, what a run must have materialized before it can be judged, the
/// authority it answers to, and the judgment itself.
///
/// # Why this carries no `PartialEq`
///
/// It used to, unused. [`Self::check`] carries a `fn` pointer, and comparing two of those
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
    /// A versioned governing record identifier for a rule whose contract a record decides,
    /// [`PORTED_STANDARD`] for a rule ported from code-standards, or [`WORKSPACE_CONVENTIONS`]
    /// for a rule whose contract is this repository's own prose. `OD-GATE-020` measured that
    /// population and confirmed the two-field shape carries it.
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
    /// its table from this field and calls it through [`RuleJudgment::Judges`]; see
    /// [`RuleJudgment`] for the one shape they share.
    pub check: RuleJudgment,
    /// Which of the sources this rule is handed it judges, as `OD-ANALYSIS-012` version 2
    /// decided: [`Population::Every`] unless the row says otherwise with
    /// [`RuleDescriptor::Judging`].
    ///
    /// The very value the rule's body filters by, not a description of it beside the body. A row
    /// that narrows names the constant its body reads, so the two cannot disagree, and the
    /// composition root reads it only to count how many sources the rule was judged over -- it
    /// hands the body exactly the sources it always did.
    pub population: Population,
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
    pub const fn Is_Citing_A_Versioned_Record(&self) -> bool
    {
        return self.contract_record_version != NO_VERSIONED_RECORD;
    }

    /// The same descriptor, citing `record` at `version` instead of the ported standard.
    ///
    /// A builder rather than a fifth parameter on [`Descriptor_For`]: this workspace's own
    /// `parameter-count` rule caps a function at four value parameters, and a rule that
    /// declares its own limits should not be the first thing to exceed them.
    #[must_use]
    pub const fn Citing(self, record: &'static str, version: u32) -> Self
    {
        return Self { contract_record: record, contract_record_version: version, ..self };
    }

    /// The same descriptor, judging `population` rather than every source it is handed.
    ///
    /// `population` is the constant the rule's own body filters by, named here rather than
    /// restated, which is what keeps the declaration and the body one statement.
    #[must_use]
    pub const fn Judging(self, population: Population) -> Self
    {
        return Self { population, ..self };
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
/// Cites [`PORTED_STANDARD`] at [`NO_VERSIONED_RECORD`], which is what most rules in the table
/// cite. A linked row that cites something else says so with [`RuleDescriptor::Citing`], so
/// the common case stays one line and the exceptions are visible as exceptions.
///
/// `judge` is the plain `fn` pointer rather than the [`RuleJudgment`] that carries it, because
/// this is the boundary where the table's own vocabulary -- a bare rule function or a closure
/// widening one -- meets the named type. Every linked entry below passes its judgment in this
/// shape, so [`RuleJudgment::New`] is named here and at no call site; a declared entry goes
/// through [`Descriptor_For_Declaration`] instead, and the two are the only two ways a row
/// is written.
const fn Descriptor_For(id: &'static str, subject: SubjectKind, requires: &'static [RequiredFact], judge: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>) -> RuleDescriptor
{
    return RuleDescriptor {
        id,
        subject,
        requires,
        check: RuleJudgment::New(judge),
        contract_record: PORTED_STANDARD,
        contract_record_version: NO_VERSIONED_RECORD,
        population: Population::Every,
    };
}

/// One descriptor for a rule that is declared rather than written, named so the table below
/// still reads as one line per rule.
///
/// Every field is taken from the declaration, including the two [`Descriptor_For`] defaults:
/// a declaration states its own contract citation, so a declared row carries no `Citing` and
/// no second place to get the authority wrong. [`DeclaredTextRule::Subject`] and
/// [`DeclaredTextRule::Requires`] derive the other two from what the declaration says about
/// test material, which is what keeps `Test_A_Source_Text_Rule_Should_Require_No_Fact` and
/// its sibling judging a declared row exactly as they judge a linked one.
///
/// `OD-RULES-034` refused the obvious alternative by name: a declared rule belongs in this
/// table, beside the linked ones, because a second list of rules is the accretion
/// `OD-GATE-020` measured going silently out of step twice.
const fn Descriptor_For_Declaration(declaration: &'static DeclaredTextRule) -> RuleDescriptor
{
    // The one refusal that is not impossible by construction, and it is refused here rather
    // than reported at run time: this table is a `const`, so a declaration naming a parameter
    // no detector in the vocabulary reads does not compile. `DeclaredTextRule`'s own doc says
    // why the field is carried at all and what would make it live.
    assert!(declaration.Is_Well_Formed(), "a declared rule names a parameter its detector does not take");

    return RuleDescriptor {
        id: declaration.id,
        subject: declaration.Subject(),
        requires: declaration.Requires(),
        check: RuleJudgment::Declaring(declaration),
        contract_record: declaration.contract_record,
        contract_record_version: declaration.contract_record_version,
        population: declaration.Population(),
    };
}

/// Every rule a run composes, with what it reads.
///
/// A rule this crate implements with no row here is held out of every run on purpose; the
/// module doc says why the identifier constants outnumber these rows and where each rule in
/// the difference is accounted for.
///
/// The order matches the composed table so the two read side by side.
///
/// Which families a selection obliges a run to materialize is derived from this table, not
/// listed beside it. [`RequiredFact`] declares every family a row can name.
/// `nomos-check-orchestration`'s `Demanded_Families` takes a selection's demand as the union
/// of `requires` over the rules it selects, and each materialization section in its
/// `run_context/capabilities.rs` runs only when that demand names the section's family.
/// `Test_The_Demand_Should_Be_The_Union_Of_What_The_Selected_Rules_Declare`, in that
/// module's `demand_tests`, holds the union against this table row by row. No family is
/// listed here: a list would be one more statement of that relation, and nothing would hold
/// it to either.
///
/// Syntax is not gated by demand at all. The run materializes it before it reads the
/// selection, so no section consults the demand for it. Rows declare it all the same,
/// because `requires` states what a rule needs materialized, and the demand is derived from
/// that rather than the other way round.
///
/// Mirrored by `Test_Every_Composed_Rule_Should_Have_A_Descriptor`, which compares this
/// list against `nomos_check_orchestration::Composed_Rules` in both directions from
/// `tests/contract/tests/rule_descriptors.rs` -- the only crate above both.
pub const DESCRIPTORS: &[RuleDescriptor] = &[
    Descriptor_For(crate::COMPLETENESS_MIRROR, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Completeness_Mirrors).Citing(crate::CONTRACT_RECORD, crate::CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::NAMING_CONVENTION, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Naming_Convention).Citing(WORKSPACE_CONVENTIONS, NO_VERSIONED_RECORD),
    Descriptor_For(crate::DEPENDENCY_DIRECTION, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Dependency_Direction).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::DEPENDENCY_COMPLETENESS, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Every_Member_Declares_A_Band).Citing(crate::DEPENDENCY_CONTRACT_RECORD, crate::DEPENDENCY_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::WRITE_AUTHORITY, SubjectKind::SourceFacts, &[RequiredFact::DependencyEdges, RequiredFact::ArchitectureDeclaration], crate::Check_Write_Authority).Citing(crate::WRITE_AUTHORITY_CONTRACT_RECORD, crate::WRITE_AUTHORITY_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::LINT_DIAGNOSTICS, SubjectKind::SourceFacts, &[RequiredFact::LintDiagnostics], crate::Check_Lint_Diagnostics).Citing(crate::LINT_CONTRACT_RECORD, crate::LINT_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::DEPENDENCY_POLICY, SubjectKind::SourceFacts, &[RequiredFact::DependencyPolicy], crate::Check_Dependency_Policy).Citing(crate::DEPENDENCY_POLICY_CONTRACT_RECORD, crate::DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::GUARANTEE_DECLARES_ITS_EXERCISER, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Guarantee_Declares_Its_Exerciser).Citing(crate::GUARANTEE_EXERCISER_CONTRACT_RECORD, crate::GUARANTEE_EXERCISER_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::UNREAD_REACHES_FINDING, SubjectKind::SourceFacts, &[RequiredFact::Reachability], crate::Check_Unread_Reaches_A_Finding).Citing(crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD, crate::UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::REVIEW_FINDING, SubjectKind::SourceFacts, &[RequiredFact::ReviewFindings], crate::Check_Review_Findings).Citing(crate::REVIEW_CONTRACT_RECORD, crate::REVIEW_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::CROSS_LANGUAGE_CORRESPONDENCE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Cross_Language_Correspondence).Citing(crate::CROSS_LANGUAGE_CONTRACT_RECORD, crate::CROSS_LANGUAGE_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::NO_TRAILING_WHITESPACE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Trailing_Whitespace(sources)),
    Descriptor_For(crate::TODO_FORMAT, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Todo_Format(sources)),
    Descriptor_For(crate::DEPRECATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Deprecation_Carries_A_Reason(sources)),
    Descriptor_For(crate::A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources)).Judging(crate::checks::populations::PATH_ATTRIBUTE_POPULATION),
    Descriptor_For(crate::SHARED_INTERIOR_MUTABILITY_SAYS_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Shared_Interior_Mutability_Says_Why(sources)).Judging(crate::checks::populations::INTERIOR_MUTABILITY_POPULATION),
    Descriptor_For_Declaration(&crate::checks::EVERY_ALLOW_DECLARATION),
    Descriptor_For(crate::UNSAFE_JUSTIFICATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Unsafe_Justification(sources)).Judging(crate::checks::populations::UNSAFE_JUSTIFICATION_POPULATION),
    Descriptor_For(crate::SCRIPTS_USE_A_PORTABLE_SHEBANG, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Scripts_Use_A_Portable_Shebang(sources)).Judging(crate::checks::populations::SCRIPT_POPULATION),
    Descriptor_For(crate::A_SCRIPT_DECLARES_ITS_PURPOSE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Script_Declares_Its_Purpose(sources)).Judging(crate::checks::populations::SCRIPT_POPULATION),
    Descriptor_For(crate::EXECUTED_SCRIPTS_SET_NOUNSET, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Executed_Scripts_Set_Nounset(sources)).Judging(crate::checks::populations::SCRIPT_POPULATION),
    Descriptor_For(crate::SLEEP_BASED_SYNCHRONIZATION, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_Sleep_Is_Not_Synchronization).Judging(crate::checks::populations::SLEEP_POPULATION),
    Descriptor_For(crate::ZERO_FLAKE_POLICY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Test_Does_Not_Retry_Until_Green(sources)).Judging(crate::checks::populations::RETRY_UNTIL_GREEN_POPULATION),
    Descriptor_For(crate::NO_MOD_RS_FILES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Mod_Rs_Files(sources)),
    Descriptor_For(crate::A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_A_Credential_Is_Not_Hardcoded_In_Source),
    Descriptor_For(crate::A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_A_Secret_Does_Not_Travel_In_A_Url),
    Descriptor_For(crate::CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_Certificate_Verification_Is_Not_Disabled),
    Descriptor_For(crate::A_DISCARDED_ERROR_IS_EXPLAINED, SubjectKind::SourceFacts, &[RequiredFact::GoDiscardedValues], crate::Check_A_Discarded_Error_Is_Explained),
    Descriptor_For(crate::A_SKIPPED_TEST_STATES_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Skipped_Test_States_Why(sources)).Judging(crate::checks::populations::SKIPPED_TEST_POPULATION),
    Descriptor_For(crate::AN_EXCLUDED_FILE_SAYS_WHY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_An_Excluded_File_Says_Why(sources)).Judging(crate::checks::populations::EXCLUDED_FILE_POPULATION),
    Descriptor_For(crate::SUPPRESSION_DIRECTIVES_CARRY_A_REASON, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Suppression_Directives_Carry_A_Reason(sources)).Judging(crate::checks::populations::GO_MARKERS_POPULATION),
    Descriptor_For(crate::WORKSPACE_MARKERS_CARRY_A_REASON, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Workspace_Markers_Carry_A_Reason(sources)).Judging(crate::checks::populations::GO_MARKERS_POPULATION),
    Descriptor_For(crate::A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Package_Is_Named_After_Its_Directory(sources)).Judging(crate::checks::populations::PACKAGE_PLACEMENT_POPULATION),
    Descriptor_For(crate::ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_Atomic_Ordering_Choices_Are_Justified).Judging(crate::checks::populations::CONCURRENCY_TEXT_POPULATION),
    Descriptor_For(crate::SEQCST_JUSTIFIED_EXPLICITLY, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_Seqcst_Justified_Explicitly).Judging(crate::checks::populations::CONCURRENCY_TEXT_POPULATION),
    Descriptor_For(crate::RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_Relaxed_Not_Used_When_Ordering_Matters).Judging(crate::checks::populations::CONCURRENCY_TEXT_POPULATION),
    Descriptor_For(crate::MODULE_AND_FIELD_NAMES_STAY_LOWER_SNAKE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Module_And_Field_Names_Stay_Lower_Snake),
    Descriptor_For(crate::FILE_NAME_MATCHES_DECLARED_TYPE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::TestMaterialPolicy], crate::Check_File_Name_Matches_Declared_Type),
    Descriptor_For(crate::CONSTANTS_SPLIT_BY_EXPORT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Go_Constants_Split_By_Export).Judging(crate::checks::populations::GO_DATA_NAMES_POPULATION),
    Descriptor_For(crate::GO_VARIABLES_USE_LOWER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Go_Variables_Use_Lower_Snake_Case).Judging(crate::checks::populations::GO_DATA_NAMES_POPULATION),
    Descriptor_For(crate::EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Exported_Go_Functions_Use_Upper_Snake_Case).Judging(crate::checks::populations::GO_FUNCTION_NAMES_POPULATION),
    Descriptor_For(crate::UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter).Judging(crate::checks::populations::GO_FUNCTION_NAMES_POPULATION),
    Descriptor_For(crate::TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::NamingPolicy], crate::Check_Go_Type_Names_Use_Camel_Case).Judging(crate::checks::populations::GO_TYPE_NAMES_POPULATION),
    Descriptor_For(crate::PARAMETER_COUNT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy, RequiredFact::TestMaterialPolicy], crate::Check_Parameter_Count),
    Descriptor_For(crate::GO_PARAMETER_COUNT, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::LimitsPolicy, RequiredFact::TestMaterialPolicy], crate::Check_Go_Parameter_Count).Judging(crate::checks::populations::GO_PARAMETER_COUNT_POPULATION),
    Descriptor_For(crate::DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, SubjectKind::SourceFacts, &[RequiredFact::ScriptingPolicy], crate::Check_Declared_Tooling_Language_For_Scripts),
    Descriptor_For(crate::FILE_SIZE_JUSTIFICATION_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_File_Size_Justification_Trigger),
    Descriptor_For(crate::NONNEGATIVE_STORAGE_IS_UNSIGNED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Nonnegative_Storage_Is_Unsigned(sources)).Judging(crate::checks::populations::SCALAR_RANGE_POPULATION),
    Descriptor_For(crate::A_KNOWN_RANGE_PICKS_ITS_TYPE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_A_Known_Range_Picks_Its_Type(sources)).Judging(crate::checks::populations::SCALAR_RANGE_POPULATION),
    Descriptor_For(crate::NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Named_Fields_Over_Positional_Variant_Payloads(sources)).Judging(crate::checks::populations::ENUM_SHAPE_POPULATION),
    Descriptor_For(crate::ONE_THOUSAND_LINE_HARD_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Go_File_Size_Hard_Trigger).Judging(crate::checks::populations::GO_FILE_SIZE_POPULATION),
    Descriptor_For(crate::FIVE_HUNDRED_LINE_REVIEW_TRIGGER, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Go_File_Size_Review_Trigger).Judging(crate::checks::populations::GO_FILE_SIZE_POPULATION),
    Descriptor_For(crate::LOWERCASE_FIRST_LETTER, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Error_Message_Starts_Lowercase(sources)).Judging(crate::checks::populations::ERROR_TEXT_POPULATION),
    Descriptor_For(crate::NO_TRAILING_PUNCTUATION, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Error_Message_Has_No_Trailing_Punctuation(sources)).Judging(crate::checks::populations::ERROR_TEXT_POPULATION),
    Descriptor_For(crate::EAGER_VS_LAZY_CONTEXT, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Eager_Vs_Lazy_Context(sources)).Judging(crate::checks::populations::ERROR_TEXT_POPULATION),
    Descriptor_For(crate::GOALS_AND_PARTS_LINE_UP, SubjectKind::Workspace, &[RequiredFact::GoalsPolicy], |_sources, reader| return crate::Check_Goals_And_Parts_Line_Up(reader)),
    Descriptor_For(crate::REQUIREMENT_TRACE_STALENESS, SubjectKind::Workspace, &[RequiredFact::RequirementTrace], |_sources, reader| return crate::Check_Requirement_Trace_Staleness(reader)) .Citing(crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD, crate::REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::ABBREVIATIONS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::WordsPolicy], crate::Check_Abbreviations),
    Descriptor_For(crate::SINGLE_LETTER_NAMES, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems], crate::Check_Single_Letter_Names),
    Descriptor_For_Declaration(&crate::checks::A_DISABLED_TEST_DECLARATION),
    Descriptor_For_Declaration(&crate::checks::INLINE_ALWAYS_DECLARATION),
    Descriptor_For(crate::NO_WILDCARD_IMPORTS, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy], crate::Check_No_Wildcard_Imports).Judging(crate::checks::populations::WILDCARD_IMPORT_POPULATION),
    Descriptor_For(crate::NO_SINGLE_LINE_FUNCTION_BODIES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Single_Line_Function_Bodies(sources)).Judging(crate::checks::populations::SINGLE_LINE_BODY_POPULATION),
    Descriptor_For(crate::NO_ORPHAN_MODULES, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_No_Orphan_Modules(sources)).Judging(crate::checks::populations::ORPHAN_MODULES_POPULATION),
    Descriptor_For(crate::PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Parameters_Borrow_Unless_Ownership_Is_Taken(sources)).Judging(crate::checks::populations::BORROWED_CONTAINER_POPULATION),
    Descriptor_For(crate::LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Lifetimes_Follow_The_Descriptive_Naming_Rule(sources)).Judging(crate::checks::populations::LIFETIME_DISCIPLINE_POPULATION),
    Descriptor_For(crate::STATIC_BOUNDS_ARE_JUSTIFIED, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Static_Bounds_Are_Justified(sources)).Judging(crate::checks::populations::LIFETIME_DISCIPLINE_POPULATION),
    Descriptor_For(crate::PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS, SubjectKind::SourceText, &[], |sources, _reader| return crate::Check_Prefer_Macro_Rules_Over_Procedural_Macros(sources)).Judging(crate::checks::populations::PROCEDURAL_MACRO_POPULATION),
    Descriptor_For(crate::NESTING_DEPTH, SubjectKind::SourceFacts, &[RequiredFact::LimitsPolicy], crate::Check_Nesting_Depth).Judging(crate::checks::populations::NESTING_DEPTH_POPULATION),
    Descriptor_For(crate::CYCLOMATIC_COMPLEXITY, SubjectKind::SourceFacts, &[RequiredFact::Complexity, RequiredFact::LimitsPolicy], crate::Check_Cyclomatic_Complexity).Citing(crate::CYCLOMATIC_COMPLEXITY_CONTRACT_RECORD, crate::CYCLOMATIC_COMPLEXITY_CONTRACT_RECORD_VERSION).Judging(crate::checks::populations::CYCLOMATIC_COMPLEXITY_POPULATION),
    Descriptor_For(crate::CLOSURE_BOUNDS_ARE_MINIMAL, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::TestMaterialPolicy], crate::Check_Closure_Bounds_Are_Minimal).Judging(crate::checks::populations::CLOSURE_BOUNDS_POPULATION),
    Descriptor_For(crate::BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS, SubjectKind::SourceFacts, &[RequiredFact::SyntaxItems, RequiredFact::TestMaterialPolicy], crate::Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths).Judging(crate::checks::populations::CLOSURE_BOUNDS_POPULATION),
    Descriptor_For(crate::COPY_CLONES, SubjectKind::SourceFacts, &[RequiredFact::CopyClones], crate::Check_Copy_Clones).Citing(crate::COPY_CLONES_CONTRACT_RECORD, crate::COPY_CLONES_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::NESTED_LOCKS, SubjectKind::SourceFacts, &[RequiredFact::NestedLocks], crate::Check_Nested_Locks).Citing(crate::NESTED_LOCKS_CONTRACT_RECORD, crate::NESTED_LOCKS_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::STANDARDS_CORPUS, SubjectKind::Workspace, &[RequiredFact::StandardsCorpusPolicy], |_sources, reader| return crate::Check_Standards_Corpus(reader)).Citing(crate::STANDARDS_CORPUS_CONTRACT_RECORD, crate::STANDARDS_CORPUS_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::UNDECLARED_POLICY_KEY, SubjectKind::Workspace, &[RequiredFact::LimitsPolicy], |_sources, reader| return crate::Check_Undeclared_Policy_Key(reader)).Citing(crate::UNDECLARED_POLICY_KEY_CONTRACT_RECORD, crate::UNDECLARED_POLICY_KEY_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::UNCOMPILED_CONDITIONAL_BRANCH, SubjectKind::SourceFacts, &[RequiredFact::CsharpConditional], crate::Check_Uncompiled_Conditional_Branch).Citing(crate::UNCOMPILED_CONDITIONAL_BRANCH_CONTRACT_RECORD, crate::UNCOMPILED_CONDITIONAL_BRANCH_CONTRACT_RECORD_VERSION),
    Descriptor_For(crate::A_LABELED_JUMP_LEAVES_ONE_LOOP, SubjectKind::SourceFacts, &[RequiredFact::SyntaxSites], crate::Check_A_Labeled_Jump_Leaves_One_Loop).Citing(crate::A_LABELED_JUMP_LEAVES_ONE_LOOP_CONTRACT_RECORD, crate::A_LABELED_JUMP_LEAVES_ONE_LOOP_CONTRACT_RECORD_VERSION).Judging(crate::checks::populations::LABELED_JUMP_POPULATION),
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
                !descriptor.Is_Citing_A_Versioned_Record(),
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

    /// The three rules `OD-RULES-034`'s form was built for, with the descriptor each one
    /// carried while it was a linked function. Every value is what the table said before the
    /// conversion, transcribed from the rows this commit replaced.
    ///
    /// This is the descriptor half of the record's indistinguishability requirement: a
    /// declared rule is not a new rule wearing an old identifier, so every field a consumer
    /// of a [`RuleDescriptor`] can read must still answer what it answered. The findings half
    /// is `checks::rust_text::declarations`' own byte-identical comparison against this tree.
    const DECLARED_ROWS: &[(&str, SubjectKind, &[RequiredFact])] = &[
        (crate::EVERY_ALLOW_CARRIES_A_JUSTIFICATION, SubjectKind::SourceFacts, &[RequiredFact::TestMaterialPolicy]),
        (crate::A_DISABLED_TEST_STATES_WHY, SubjectKind::SourceText, &[]),
        (crate::INLINE_ALWAYS_JUSTIFICATION, SubjectKind::SourceText, &[]),
    ];

    /// A declared row answers every reading a linked row answers, with the values it always
    /// had. `Declared_Rules` in `nomos-check-orchestration` builds a `RulePackage` out of
    /// exactly these readings plus [`RuleDescriptor::Rule`] and
    /// [`RuleDescriptor::Is_Citing_A_Versioned_Record`], both checked below, so a declaration
    /// that passed this could not change what that derivation produces.
    #[test]
    fn Test_A_Declared_Row_Should_Read_Exactly_As_The_Linked_Row_It_Replaced()
    {
        for (id, subject, requires) in DECLARED_ROWS
        {
            let described = DESCRIPTORS
                .iter()
                .find(|candidate| return candidate.id == *id)
                .expect("every declared rule is in the one table");

            assert_eq!(described.subject, *subject, "{id}");
            assert_eq!(described.requires, *requires, "{id}");
            assert_eq!(described.contract_record, PORTED_STANDARD, "{id}");
            assert_eq!(described.contract_record_version, NO_VERSIONED_RECORD, "{id}");
            assert!(!described.Is_Citing_A_Versioned_Record(), "{id}");
            assert_eq!(described.Rule(), nomos_contracts::RuleId::New(*id), "{id}");
        }
    }

    /// The last channel through which a consumer could tell the two arms apart, closed on
    /// purpose. A derived `Debug` would print the arm; this asserts the hand-written one does
    /// not, by comparing a declared row's rendering against a linked row's.
    #[test]
    fn Test_A_Declared_Judgment_Should_Render_Exactly_As_A_Linked_One()
    {
        let declared = DESCRIPTORS
            .iter()
            .find(|candidate| return candidate.id == crate::EVERY_ALLOW_CARRIES_A_JUSTIFICATION)
            .expect("the declared rule is in the one table");
        let linked = DESCRIPTORS
            .iter()
            .find(|candidate| return candidate.id == crate::UNSAFE_JUSTIFICATION)
            .expect("the linked sibling is in the one table");

        assert_eq!(format!("{:?}", declared.check), format!("{:?}", linked.check));
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
