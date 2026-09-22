#![doc = include_str!("../docs/api/lib.md")]
#![forbid(unsafe_code)]

mod checks;
mod declared_universe;
mod facts;
mod reading;
mod registry;
mod rule_descriptor;
mod universe_kind;

use nomos_cap_syntax::Language;
use nomos_capability::Requirement;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId};

pub use checks::{
    Check_Atomic_Ordering_Choices_Are_Justified, Check_Relaxed_Not_Used_When_Ordering_Matters,
    Check_Seqcst_Justified_Explicitly, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS,
    SEQCST_JUSTIFIED_EXPLICITLY,
    Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION,
    Check_Dependency_Direction, Check_Every_Member_Declares_A_Band, Check_Write_Authority,
    DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
    WRITE_AUTHORITY, WRITE_AUTHORITY_CONTRACT_RECORD, WRITE_AUTHORITY_CONTRACT_RECORD_VERSION,
    Check_Eager_Vs_Lazy_Context, Check_Error_Message_Has_No_Trailing_Punctuation, Check_Error_Message_Starts_Lowercase,
    EAGER_VS_LAZY_CONTEXT, LOWERCASE_FIRST_LETTER, NO_TRAILING_PUNCTUATION,
    Check_A_Consumer_Imports_Through_The_Facade, Check_A_Facade_Publishes_A_Child_One_Way,
    Check_A_Renamed_Facade_Re_Export_Names_The_Contract, FACADE_ALIASES_NAME_THE_CONTRACT,
    FACADE_CHOOSES_FLATTENING_OR_NAMESPACE, FACADE_CONSUMERS_USE_THE_FACADE_PATH,
    Check_Goals_And_Parts_Line_Up, GOALS_AND_PARTS_LINE_UP,
    Check_A_Test_Does_Not_Retry_Until_Green, Check_Sleep_Is_Not_Synchronization, SLEEP_BASED_SYNCHRONIZATION, ZERO_FLAKE_POLICY,
    Check_Deprecation_Carries_A_Reason, Check_No_Decorative_Section_Dividers, Check_No_Single_Line_Function_Bodies,
    Check_No_Trailing_Whitespace, Check_Todo_Format, DEPRECATION, NO_DECORATIVE_SECTION_DIVIDERS,
    NO_SINGLE_LINE_FUNCTION_BODIES, NO_TRAILING_WHITESPACE, TODO_FORMAT,
    Check_Function_Arity_Policy, Check_Go_Parameter_Count, Check_Parameter_Count, FunctionArityPolicy,
    FunctionAritySource, ReceiverAllowance, GO_PARAMETER_COUNT, PARAMETER_COUNT,
    Check_Lint_Diagnostics, LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION, LINT_DIAGNOSTICS,
    Check_Dependency_Policy, DEPENDENCY_POLICY, DEPENDENCY_POLICY_CONTRACT_RECORD,
    DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION,
    Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CONTRACT_RECORD, CROSS_LANGUAGE_CONTRACT_RECORD_VERSION,
    CROSS_LANGUAGE_CORRESPONDENCE,
    Check_Named_Fields_Over_Positional_Variant_Payloads, NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS,
    Check_Domain_Values_Are_Distinct_Types, DOMAIN_VALUES_ARE_DISTINCT_TYPES,
    Check_Constants_Are_The_Exception_To_Function_Scope_Use, CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE,
    Check_A_Known_Range_Picks_Its_Type, Check_Nonnegative_Storage_Is_Unsigned, A_KNOWN_RANGE_PICKS_ITS_TYPE, NONNEGATIVE_STORAGE_IS_UNSIGNED,
    Check_Abbreviations, Check_Boolean_Predicates, Check_File_Name_Matches_Declared_Type,
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_Go_Constants_Split_By_Export, Check_Go_Type_Names_Use_Camel_Case,
    Check_Go_Variables_Use_Lower_Snake_Case, Check_Module_And_Field_Names_Stay_Lower_Snake,
    Check_Naming_Clarity, Check_Naming_Convention, Check_One_Public_Type_Per_File, Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
    Check_Single_Letter_Names, Check_Test_Names_Describe_Behavior, Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    ABBREVIATIONS, BOOLEAN_PREDICATES, CONSTANTS_SPLIT_BY_EXPORT,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE, GO_VARIABLES_USE_LOWER_SNAKE_CASE,
    MODULE_AND_FIELD_NAMES_STAY_LOWER_SNAKE, NAMING_CLARITY, NAMING_CONVENTION,
    ONE_PUBLIC_TYPE_PER_FILE, PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE, SINGLE_LETTER_NAMES,
    TEST_NAME_DESCRIBES_BEHAVIOR, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
    Check_Guarantee_Declares_Its_Exerciser, GUARANTEE_DECLARES_ITS_EXERCISER,
    GUARANTEE_EXERCISER_CONTRACT_RECORD, GUARANTEE_EXERCISER_CONTRACT_RECORD_VERSION,
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
    Check_Requirement_Trace_Staleness, REQUIREMENT_TRACE_STALENESS, REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD,
    REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION,
    Check_Review_Findings, REVIEW_CONTRACT_RECORD, REVIEW_CONTRACT_RECORD_VERSION, REVIEW_FINDING,
    Check_Declared_Role_Matches_Surface, RoleSurfacePair, DECLARED_ROLE_MATCHES_SURFACE,
    Check_A_Disabled_Test_States_Why, Check_A_Rust_Path_Stays_Within_Its_Own_Subtree,
    Check_Every_Allow_Carries_A_Justification, Check_Inline_Always_Justification,
    Check_Panics_Are_Justified_Documented_And_Validated, Check_Shared_Interior_Mutability_Says_Why,
    Check_Unsafe_Justification, Check_Unwrap_Expect_Discipline,
    A_DISABLED_TEST_STATES_WHY, A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, EVERY_ALLOW_CARRIES_A_JUSTIFICATION,
    INLINE_ALWAYS_JUSTIFICATION, PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED, SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
    UNSAFE_JUSTIFICATION, UNWRAP_EXPECT_DISCIPLINE,
    Check_A_Discarded_Error_Is_Explained, Check_A_Skipped_Test_States_Why, Check_An_Excluded_File_Says_Why,
    Check_Suppression_Directives_Carry_A_Reason, Check_Workspace_Markers_Carry_A_Reason,
    A_DISCARDED_ERROR_IS_EXPLAINED, A_SKIPPED_TEST_STATES_WHY, AN_EXCLUDED_FILE_SAYS_WHY,
    SUPPRESSION_DIRECTIVES_CARRY_A_REASON, WORKSPACE_MARKERS_CARRY_A_REASON,
    Check_A_Script_Declares_Its_Purpose, Check_Declared_Tooling_Language_For_Scripts, Check_Executed_Scripts_Set_Nounset,
    Check_Scripts_Use_A_Portable_Shebang,
    A_SCRIPT_DECLARES_ITS_PURPOSE, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, EXECUTED_SCRIPTS_SET_NOUNSET,
    SCRIPTS_USE_A_PORTABLE_SHEBANG,
    Check_A_Credential_Is_Not_Hardcoded_In_Source, Check_A_Secret_Does_Not_Travel_In_A_Url,
    Check_Certificate_Verification_Is_Not_Disabled, A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE,
    A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, CERTIFICATE_VERIFICATION_IS_NOT_DISABLED,
    Check_No_Orphan_Modules, NO_ORPHAN_MODULES,
    Check_Nesting_Depth, NESTING_DEPTH,
    Check_Parameters_Borrow_Unless_Ownership_Is_Taken, PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN,
    Check_Lifetimes_Follow_The_Descriptive_Naming_Rule, Check_Static_Bounds_Are_Justified,
    LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, STATIC_BOUNDS_ARE_JUSTIFIED,
    Check_Prefer_Macro_Rules_Over_Procedural_Macros, PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS,
    Check_Boxed_Closures_Are_Justified_And_Off_Hot_Paths, Check_Closure_Bounds_Are_Minimal,
    BOXED_CLOSURES_ARE_JUSTIFIED_AND_OFF_HOT_PATHS, CLOSURE_BOUNDS_ARE_MINIMAL,
    Check_A_Package_Is_Named_After_Its_Directory, Check_No_Wildcard_Imports, A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
    NO_WILDCARD_IMPORTS,
    Check_File_Size_Justification_Trigger, Check_File_Size_Review_Trigger, Check_Go_File_Size_Hard_Trigger,
    Check_Go_File_Size_Review_Trigger, Check_No_Mod_Rs_Files,
    FILE_SIZE_JUSTIFICATION_TRIGGER, FILE_SIZE_REVIEW_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
    NO_MOD_RS_FILES, ONE_THOUSAND_LINE_HARD_TRIGGER,
};
pub use declared_universe::DeclaredUniverse;
pub use reading::Reading;
pub use registry::{RuleOffer, RuleRegistry, RuleRegistryError};
pub use rule_descriptor::{RequiredFact, RuleDescriptor, SubjectKind, DESCRIPTORS};
pub use universe_kind::{UniverseKind, Universes_In};

/// What this crate needs from a syntax provider before it will believe an answer.
///
/// A floor, not a preference, and stated by the rule rather than by whoever runs it.
///
/// # Why the floor is here and not at the call site
///
/// Because the rule is the party that knows what an approximation would cost it. A
/// composition root that could lower this would be able to feed the resolver a line
/// scanner, and `nomos-lang-rust-scan`'s own
/// `Test_The_Declared_Unsoundness_Should_Be_Demonstrable` establishes exactly what that
/// buys: it reports declarations written inside block comments. A `fn Test_Renamed_Away`
/// in a comment or a string literal would then resolve a mirror claim that nothing checks
/// — the defect this rule exists to find, arriving through the rule's own resolver.
/// [`nomos_contracts::Guarantee::Satisfies`] refuses that offer against this floor and
/// `Registry::Resolve` reports `Unmet::BelowRequirement`.
///
/// # Why each axis is what it is
///
/// [`FactVariant::Syntactic`] because a check name is what a file says on its face;
/// nothing here resolves a name or follows a `use`.
///
/// Soundness [`Assurance::Sound`] because every name that resolves a claim of coverage
/// must really be in the token stream. This is the axis that separates the two providers
/// and the only one this rule cannot compromise on.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound`. A parser cannot bound
/// what a macro hid, so no provider of this capability can honestly claim complete — and
/// a floor no provider can meet is not caution, it is a declared need with nothing behind
/// it. What follows from an unknown-complete index is handled where it matters: a check
/// name the index is missing produces a phantom finding, and `mirror.rs` refuses to raise
/// one while any subject went unread.
///
/// [`IncrementalGranularity::File`] because a check name belongs to the file that declares
/// it, and nothing coarser would let one edited file be re-read on its own.
///
/// There is deliberately no `Preferring`. Naming a provider would be the rule deciding
/// what the registry exists to decide.
#[must_use]
pub(crate) fn Syntax_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );

    return Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        guarantee,
    );
}

/// [`Syntax_Requirement`], narrowed to `preferred` when the caller names one —
/// `OD-CAPABILITY-009`'s decided fix for a capability whose real offers partition by
/// subject rather than compete over one.
///
/// Takes the preference as data rather than computing it from a path: this crate never
/// depends on a language-provider crate, and recognizing which language a path belongs to
/// is exactly that kind of dependency. The composition root already depends on every
/// registered syntax provider by name — it is the one place allowed to compute
/// `Recognition::Of_Path`, once, and carry the result here as
/// [`SourceFile::preferred_syntax_provider`], the same carried-rather-than-derived
/// convention that field's own sibling `subject` already documents. `None` carries no
/// preference and falls through to the floor above, unpreferenced — the case of a path
/// neither registered provider recognizes, which `OD-CAPABILITY-009` names explicitly
/// rather than leaves implicit. That fallthrough is still safe: nothing materializes a
/// fact under either provider's identity for a path neither recognizes, so an unrecognized
/// path still surfaces as an honestly unread subject rather than a wrongly-addressed one.
#[must_use]
pub(crate) fn Syntax_Requirement_For(preferred: Option<ProviderId>) -> Requirement
{
    let need = Syntax_Requirement();

    return match preferred
    {
        Some(provider) => need.Preferring(provider),
        None => need,
    };
}

/// The language name Go-specific rules name, as `nomos-lang-go` declares it.
///
/// A rule naming one language is inherent to a language-specific norm; knowing the set of
/// languages is not, and this crate names two literals rather than a set. The value has to
/// agree with `nomos_lang_go::LANGUAGE`, which this crate cannot depend on to check --
/// `run_context.rs` carries the test that fails loudly if the two ever drift.
pub const GO_LANGUAGE: &str = "go";

/// The language name Rust-specific rules name, as both Rust providers declare it.
///
/// Both `nomos_lang_rust::LANGUAGE` and `nomos_lang_rust_scan::LANGUAGE` carry this one
/// value, which is why a rule asks about the language and never about a provider identity.
pub const RUST_LANGUAGE: &str = "rust";

/// One file of source, as the caller found it.
///
/// `path` is repo-relative with forward slashes, and it is reporting only. Nothing in
/// this crate keys anything on it: `identity.rs` states the rule for the whole system,
/// and a finding identified by its path is a finding that closes and reopens every time
/// somebody moves a file.
///
/// `check-file-name` names this file for not matching this type. `lib.rs` is the crate
/// root every crate needs as its entry point, so renaming it to follow a type it declares
/// is not a real option — a structural exception this file cannot resolve on its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Repo-relative, forward slashes. For reporting.
    pub path: String,
    /// The subject the composition root filed this file's facts under.
    ///
    /// Carried rather than derived, and that is the whole of why it is a field. A
    /// [`SubjectId`] is a digest of a *normalized* path, and normalization is an
    /// addressing convention the root owns — `tests/integration/src/corpus.rs` folds
    /// case and drops `.` segments and says why. A rule that computed its own would be a
    /// second answer to that convention, and the two would disagree silently: every
    /// `Require` would miss, every mirror would fail to resolve, and the run would report
    /// a workspace full of unavailable facts rather than a mistake in one function.
    /// Carrying it makes rule and root agree by construction.
    ///
    /// The *inputs* digest is not carried and is recomputed in the rule from `text`. That
    /// is the same deliberate choice `tests/integration/src/slice.rs` documents: if the two
    /// disagree the read misses loudly, and a shared helper would make that whole class of
    /// mismatch untestable.
    pub subject: SubjectId,
    /// The file's full text.
    pub text: String,
    /// Which `nomos.cap.syntax.items` provider identity to narrow toward when this file's
    /// syntax fact is required, if any — `OD-CAPABILITY-009`'s fix for a capability whose
    /// real offers partition by subject rather than compete over one.
    ///
    /// Carried rather than derived, for the identical reason [`SourceFile::subject`] is: a
    /// capability with more than one registered offer over disjoint subjects (today,
    /// `nomos-lang-rust` over `.rs` and `nomos-lang-go` over `.go`) needs its read side and
    /// its write side to agree on which provider answers for one file, and computing that
    /// twice independently is how the two sides drift. The composition root recognizes
    /// `path` against every provider it registers and sets this once, before any rule ever
    /// sees the file — this crate itself never depends on a language-provider crate to
    /// compute it. `None` means either no registered provider recognizes this path, or the
    /// caller named none; [`Syntax_Requirement_For`] treats both the same way, falling
    /// through to the subject-agnostic floor.
    pub preferred_syntax_provider: Option<ProviderId>,
    /// Which language this file is written in, if any registered provider recognizes it.
    ///
    /// Carried rather than derived, for the identical reason [`SourceFile::subject`] and
    /// [`SourceFile::preferred_syntax_provider`] are. A rule whose norm is about one
    /// language used to answer this privately by looking at the extension, and eleven
    /// copies of that three-line test had accumulated across nine modules before
    /// `OD-RULES-014` measured them; the composition root already recognizes `path`
    /// against every provider it registers, so it sets this once and every rule reads one
    /// answer.
    ///
    /// Distinct from `preferred_syntax_provider` and not derivable from it: that field
    /// names the tool that would read the file, and `nomos.cap.syntax.items` has two
    /// registered offers for Rust alone, so a provider identity answers a narrower
    /// question than "which language is this".
    ///
    /// `None` means no registered provider recognized the path, or the caller named none.
    /// A rule restricted to a language treats that as "not my subject", the same as a
    /// different language.
    pub language: Option<Language>,
}

impl SourceFile
{
    /// Builds one, for callers that have a path, a subject and the text.
    ///
    /// `preferred_syntax_provider` starts `None` — a caller that has already resolved one
    /// sets the field directly, since every field here is public for exactly that reason.
    #[must_use]
    pub fn New(path: impl Into<String>, subject: SubjectId, text: impl Into<String>) -> Self
    {
        return Self {
            path: path.into(),
            subject,
            text: text.into(),
            preferred_syntax_provider: None,
            language: None,
        };
    }

    /// Whether this file is written in `language`.
    ///
    /// The one comparison a language-restricted rule makes. It takes the name rather than
    /// a [`Language`] so a rule states its own literal without allocating one per file,
    /// and it answers `false` for an unrecognized file rather than guessing.
    #[must_use]
    pub fn Is_Written_In(&self, language: &str) -> bool
    {
        return self
            .language
            .as_ref()
            .is_some_and(|carried| return carried.As_Str() == language);
    }
}

/// Recognizes a path's language the way the composition root does, for tests only.
///
/// Every `mod tests` in this crate that builds a [`SourceFile`] needs its `language` set,
/// because in production the root sets it and no rule derives it. This is the one stand-in
/// for that step: eleven private copies of an extension test is what `OD-RULES-014` removed
/// from the rules, and eight copies in their tests would be the same defect wearing a
/// `#[cfg(test)]`.
#[cfg(test)]
#[must_use]
pub(crate) fn Recognized_Language_In_Tests(path: &str) -> Option<Language>
{
    return match std::path::Path::new(path).extension()?.to_str()?
    {
        "rs" => Some(Language::New(RUST_LANGUAGE)),
        "go" => Some(Language::New(GO_LANGUAGE)),
        _ => None,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Syntax_Requirement_For_Should_Narrow_The_Floor_To_A_Named_Preference()
    {
        let preferred = ProviderId::New("nomos.test.provider");

        let need = Syntax_Requirement_For(Some(preferred.clone()));

        assert_eq!(need.preferred, Some(preferred));
        assert_eq!(need.minimum, Syntax_Requirement().minimum, "the floor itself is untouched");
    }

    /// The case `OD-CAPABILITY-009` names explicitly: no preference is named, and the
    /// floor falls through to the registry's own, subject-agnostic ranking — the same as
    /// before this fix existed.
    #[test]
    fn Test_Syntax_Requirement_Should_Be_The_Bare_Floor_Returned_When_No_Preference_Is_Named()
    {
        let need = Syntax_Requirement_For(None);

        assert_eq!(need, Syntax_Requirement());
    }

    #[test]
    fn Test_New_Should_Build_A_Source_File_Whose_Preferred_Provider_Starts_Unset()
    {
        let subject = SubjectId::From_Digest(nomos_model::Content_Digest(b"a.rs"));

        let source = SourceFile::New("a.rs", subject, "fn Test_Something() {}");

        assert_eq!(source.path, "a.rs");
        assert_eq!(source.text, "fn Test_Something() {}");
        assert_eq!(source.subject, subject);
        assert_eq!(source.preferred_syntax_provider, None);
    }
}
