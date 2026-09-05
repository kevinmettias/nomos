//! The rule registry this crate composes.
//!
//! `nomos_rules::RuleRegistry` has existed since `OD-RULES-004` and its own module doc says
//! plainly that nothing consults it: "Nothing here is consulted by `Run()`, and nothing here
//! changes what runs on any given `nomos check`." This is that registry's first real
//! consumer -- not a change to what `nomos check` runs, which stays exactly what
//! `nomos-check-orchestration::run_context::Run` already does, but a second, honest answer to
//! "what rules exist" built from the same registration seam rather than by re-deriving the
//! list.
//!
//! # Honest requires whole, and this is now checked rather than remembered
//!
//! The parity this registry claims with `Run` went false silently three times. It composed
//! two of three rules between `P13-DEPENDENCY-WIRE-1` and `P13-GATE-REGISTRY-THIRD-RULE`,
//! three of four until `P13-CONTROLFLOW-REACHABILITY-WIRE`, four of five until
//! `OD-GATE-019-REGISTRY-COHERENCE-A`, five of eight until
//! `OD-GATE-019-REGISTRY-COHERENCE-B`, and then eight of fifty-six -- the divergence
//! `OD-GATE-020` measured. Each of those corrections restored the count by hand and left the
//! same mechanism that had let it drift.
//!
//! `OD-GATE-020` named why: `OD-GATE-011`'s legitimate-exception test asks that both sides of
//! a duplicated answer derive from one authority named outside either artifact, and
//! `nomos-check-orchestration` exported none, so this list and `Run`'s were two hand-typed
//! enumerations of one question, compared by eye at correction time and by nothing in
//! between. `P35-GATE-020-COMPOSED-RULES-EXPORT` built that authority --
//! [`nomos_check_orchestration::Composed_Rules`], read off the same array literal `Run`
//! executes -- and `crate::tests` now compares this table against it on every `cargo test`.
//! A rule composed into `Run` without a row here is a red test, not a silent lie.
//!
//! This is not yet a shared derivation: the rows below are still authored, because a rule's
//! contract citation is knowledge no export carries. It is the shape `OD-GATE-020` described
//! as the fix -- one hand-typed list and one comparison against it.
//!
//! # What each rule cites, audited rule by rule
//!
//! `OD-GATE-020` left this classification to a correction item and expected roughly thirty
//! rules to want a versioned `*_CONTRACT_RECORD` constant of their own, on the strength of a
//! grep for record-shaped identifiers in the rule modules. Read module by module, that is not
//! what those citations are. Seven rules cite a governing record that decides *what the rule
//! requires*: `D-134`, `OD-RULES-003` (twice), `OD-RULES-008`, `OD-RULES-010` (twice) and
//! `OD-CAPABILITY-010`. Every other rule in this table is a port of a code-standards rule,
//! and its own module doc opens by saying so -- "code-standards' `<rule-id>` rule". The
//! records those modules name are mechanism, not contract: `OD-RULES-011` and
//! `OD-CAPABILITY-004` decide how a repository's declared parameters reach a rule, and
//! `OD-RULES-001` decides that a rule takes its subject as an argument. Several say outright
//! that `OD-RULES-011` does *not* apply to them -- `flakiness_text.rs`'s "not a fifth
//! `OD-RULES-011` capability", `go_text.rs`'s "the generalization is shared code, not shared
//! configuration". `OD-RULES-014`, `OD-RULES-015` and `OD-RULES-016` are the nearest miss:
//! they decide how far the port reaches on specific ambiguities -- a module of operations, a
//! companion type -- not what `file-name-matches-declared-type` requires, which is
//! code-standards' to say.
//!
//! So the ported rules cite [`PORTED_STANDARD`] and `Check_Naming_Convention` keeps
//! [`WORKSPACE_CONVENTIONS`], which `naming.rs`'s own "# Why this has no `CONTRACT_RECORD`"
//! section states is its contract: `README.md`'s Conventions section, this workspace's own
//! house convention rather than a ported one. Both take [`NO_VERSIONED_RECORD`], the sentinel
//! `Offer_Naming_Convention` introduced provisionally and `OD-GATE-020` accepted as the
//! general pattern rather than a one-off awaiting its own type. No field is added to
//! `RuleOffer` by any of this, which is what that record decided.
//!
//! A version literal never appears in a row: a citation written in a composition root drifts
//! silently against the record it names, so each of the seven names constants that live
//! beside the rule, where whoever amends the record is already reading, and
//! `tests/contract/tests/rule_contract_citation.rs` checks both halves against the records'
//! own front matter.
use nomos_contracts::RuleId;
use nomos_rules::{
    ABBREVIATIONS, AN_EXCLUDED_FILE_SAYS_WHY, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED,
    A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, A_DISABLED_TEST_STATES_WHY, A_DISCARDED_ERROR_IS_EXPLAINED,
    A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE,
    A_SCRIPT_DECLARES_ITS_PURPOSE, A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, A_SKIPPED_TEST_STATES_WHY,
    CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, COMPLETENESS_MIRROR, CONSTANTS_SPLIT_BY_EXPORT,
    CONTRACT_RECORD, CONTRACT_RECORD_VERSION, CROSS_LANGUAGE_CONTRACT_RECORD,
    CROSS_LANGUAGE_CONTRACT_RECORD_VERSION, CROSS_LANGUAGE_CORRESPONDENCE, DATA_NAMES_STAY_LOWER_SNAKE,
    DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION, DEPENDENCY_POLICY,
    DEPENDENCY_POLICY_CONTRACT_RECORD, DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION, DEPRECATION,
    EAGER_VS_LAZY_CONTEXT, EVERY_ALLOW_CARRIES_A_JUSTIFICATION, EXECUTED_SCRIPTS_SET_NOUNSET,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE,
    FILE_SIZE_JUSTIFICATION_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER, GOALS_AND_PARTS_LINE_UP,
    GO_HELPERS_PACKAGE_FIVE_INPUTS, GO_VARIABLES_USE_LOWER_SNAKE_CASE, INLINE_ALWAYS_JUSTIFICATION,
    LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION, LINT_DIAGNOSTICS, LOWERCASE_FIRST_LETTER,
    LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, NESTING_DEPTH,
    NAMING_CONVENTION, NO_MOD_RS_FILES, NO_ORPHAN_MODULES, NO_SINGLE_LINE_FUNCTION_BODIES, NO_TRAILING_PUNCTUATION,
    PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN, PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS,
    STATIC_BOUNDS_ARE_JUSTIFIED,
    NO_TRAILING_WHITESPACE, NO_WILDCARD_IMPORTS, ONE_THOUSAND_LINE_HARD_TRIGGER, PARAMETER_COUNT,
    RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, RuleOffer, RuleRegistry, RuleRegistryError,
    SCRIPTS_USE_A_PORTABLE_SHEBANG, SEQCST_JUSTIFIED_EXPLICITLY, SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
    SINGLE_LETTER_NAMES, SLEEP_BASED_SYNCHRONIZATION, SUPPRESSION_DIRECTIVES_CARRY_A_REASON, TODO_FORMAT,
    TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
    UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION, UNSAFE_JUSTIFICATION, WORKSPACE_MARKERS_CARRY_A_REASON,
    ZERO_FLAKE_POLICY,
};

/// The authority a rule ported from code-standards cites: the standard itself.
///
/// Every rule carrying this is one whose own module doc opens "code-standards' `<rule-id>`
/// rule", and the rule id in the same offer is the document's identity within that standard,
/// so the pair names the contract exactly. Distinct from [`WORKSPACE_CONVENTIONS`] because
/// the two are different documents: nothing in this repository's `README.md` states what
/// `atomic-ordering-choices-are-justified` requires.
const PORTED_STANDARD: &str = "code-standards";

/// The authority `Check_Naming_Convention` cites: `README.md`'s Conventions section.
///
/// `naming.rs`'s own "# Why this has no `CONTRACT_RECORD`" section says this in full, and
/// says why no record was manufactured to replace it -- "a record authored only to give this
/// rule something to cite would be the record standing in for the check, not the other way
/// around."
const WORKSPACE_CONVENTIONS: &str = "README.md";

/// The version a prose contract is at: none.
///
/// Zero marks the absence, not version zero of something. `OD-GATE-020` accepted this
/// sentinel as the general pattern for a record-less rule after `Offer_Naming_Convention`
/// introduced it for one, on the ground that forty-eight more instances of the same two
/// shapes are the second-and-onward case `OD-RULES-005` and `OD-RULES-006` asked for before
/// extending `RuleOffer` -- and that they take a shape the ninth rule had already proved out
/// rather than a new one.
const NO_VERSIONED_RECORD: u32 = 0;

/// Every rule this workspace ships: its identifier, the authority its implementation cites,
/// and the version of that authority it was written against.
///
/// In the order `nomos_check_orchestration` composes them, so a reader comparing the two
/// lists reads them the same way round. The seven versioned citations lead, because they
/// were here first and because they are the rows whose middle column a reader has to check
/// against a record; the rest follow grouped by the module that implements them.
const OFFERINGS: &[(&str, &str, u32)] = &[
    (COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION),
    (NAMING_CONVENTION, WORKSPACE_CONVENTIONS, NO_VERSIONED_RECORD),
    (DEPENDENCY_DIRECTION, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION),
    (DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION),
    (LINT_DIAGNOSTICS, LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION),
    (DEPENDENCY_POLICY, DEPENDENCY_POLICY_CONTRACT_RECORD, DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION),
    (UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD, UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION),
    (CROSS_LANGUAGE_CORRESPONDENCE, CROSS_LANGUAGE_CONTRACT_RECORD, CROSS_LANGUAGE_CONTRACT_RECORD_VERSION),
    // formatting.rs
    (NO_TRAILING_WHITESPACE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (TODO_FORMAT, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (DEPRECATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // rust_text.rs
    (A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (SHARED_INTERIOR_MUTABILITY_SAYS_WHY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (EVERY_ALLOW_CARRIES_A_JUSTIFICATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (UNSAFE_JUSTIFICATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // script_discipline.rs
    (SCRIPTS_USE_A_PORTABLE_SHEBANG, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (A_SCRIPT_DECLARES_ITS_PURPOSE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (EXECUTED_SCRIPTS_SET_NOUNSET, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // flakiness_text.rs
    (SLEEP_BASED_SYNCHRONIZATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (ZERO_FLAKE_POLICY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // structure.rs
    (NO_MOD_RS_FILES, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // security_text.rs
    (A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // go_text.rs
    (A_DISCARDED_ERROR_IS_EXPLAINED, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (A_SKIPPED_TEST_STATES_WHY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (AN_EXCLUDED_FILE_SAYS_WHY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (SUPPRESSION_DIRECTIVES_CARRY_A_REASON, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (WORKSPACE_MARKERS_CARRY_A_REASON, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // placement.rs
    (A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // concurrency_text.rs
    (ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (SEQCST_JUSTIFIED_EXPLICITLY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // data_names.rs
    (DATA_NAMES_STAY_LOWER_SNAKE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // file_names.rs
    (FILE_NAME_MATCHES_DECLARED_TYPE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // go_data_names.rs
    (CONSTANTS_SPLIT_BY_EXPORT, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (GO_VARIABLES_USE_LOWER_SNAKE_CASE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // go_function_names.rs
    (EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // go_type_names.rs
    (TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // function_shape.rs
    (PARAMETER_COUNT, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (GO_HELPERS_PACKAGE_FIVE_INPUTS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // script_discipline.rs
    (DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // structure.rs
    (FILE_SIZE_JUSTIFICATION_TRIGGER, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (ONE_THOUSAND_LINE_HARD_TRIGGER, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (FIVE_HUNDRED_LINE_REVIEW_TRIGGER, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // error_text.rs
    (LOWERCASE_FIRST_LETTER, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (NO_TRAILING_PUNCTUATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (EAGER_VS_LAZY_CONTEXT, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // goals.rs
    (GOALS_AND_PARTS_LINE_UP, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // abbreviations.rs
    (ABBREVIATIONS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // single_letter_names.rs
    (SINGLE_LETTER_NAMES, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // rust_text.rs
    (A_DISABLED_TEST_STATES_WHY, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (INLINE_ALWAYS_JUSTIFICATION, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // placement.rs
    (NO_WILDCARD_IMPORTS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // formatting.rs
    (NO_SINGLE_LINE_FUNCTION_BODIES, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // orphan_modules.rs
    (NO_ORPHAN_MODULES, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // borrowed_container.rs
    (PARAMETERS_BORROW_UNLESS_OWNERSHIP_IS_TAKEN, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // lifetime_discipline.rs
    (LIFETIMES_FOLLOW_THE_DESCRIPTIVE_NAMING_RULE, PORTED_STANDARD, NO_VERSIONED_RECORD),
    (STATIC_BOUNDS_ARE_JUSTIFIED, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // procedural_macro.rs
    (PREFER_MACRO_RULES_OVER_PROCEDURAL_MACROS, PORTED_STANDARD, NO_VERSIONED_RECORD),
    // nesting_depth.rs
    (NESTING_DEPTH, PORTED_STANDARD, NO_VERSIONED_RECORD),
];

/// Registers every rule `nomos-check-orchestration::Run` composes and hands back the
/// registry.
///
/// That parity is the property this function exists to keep true rather than a coincidence
/// to note, and `crate::tests` is where it is kept: it compares what this returns against
/// [`nomos_check_orchestration::Composed_Rules`], so the two cannot diverge unnoticed the
/// way they did three times before that export existed.
///
/// # Errors
///
/// [`RuleRegistryError::AlreadyOffered`] if the same [`RuleId`] appeared twice in
/// [`OFFERINGS`]. Not reachable today -- `Test_Registered_Should_Offer_Every_Composed_Rule`
/// would fail first, and `Composed_Rules`' own uniqueness test one crate away would fail
/// before that -- but returned rather than unwound for the same reason
/// `nomos_check_orchestration::Registered` returns its own `RegistryError`: a composition
/// root's own defect must be representable, not panicked past.
pub fn Registered() -> Result<RuleRegistry, RuleRegistryError>
{
    let mut registry = RuleRegistry::New();

    for (rule, contract_record, contract_record_version) in OFFERINGS
    {
        registry.Offer(RuleOffer {
            rule: RuleId::New(*rule),
            contract_record: (*contract_record).to_owned(),
            contract_record_version: *contract_record_version,
        })?;
    }

    return Ok(registry);
}

#[cfg(test)]
mod tests
{
    use super::{Registered, NO_VERSIONED_RECORD, OFFERINGS, PORTED_STANDARD, WORKSPACE_CONVENTIONS};
    use nomos_contracts::RuleId;

    /// The assertion this whole file is arranged around: what is offered is what is run.
    ///
    /// Against `Composed_Rules` rather than a written-out list of identifiers, which is the
    /// whole of what changed. The expectation this replaced named eight rules by hand and was
    /// green while `Run` composed fifty-six -- a hand-written expectation checked against a
    /// hand-written registration is two hand-written artifacts agreeing with each other, not
    /// a claim about `Run`. Sorted on both sides because `Offers` yields in `RuleId` order
    /// and `Composed_Rules` yields in the order `Run` executes; this is a statement about
    /// membership, and the two orders are each meaningful where they are.
    #[test]
    fn Test_Registered_Should_Offer_Every_Composed_Rule()
    {
        let registry = Registered().expect("this crate's own registration must not be contradictory");

        let mut offered: Vec<RuleId> = registry.Offers().map(|offer| return offer.rule.clone()).collect();
        let mut composed = nomos_check_orchestration::Composed_Rules();
        offered.sort();
        composed.sort();

        assert_eq!(
            offered, composed,
            "every rule nomos-check-orchestration::Run composes needs a row in OFFERINGS, and \
             a row here that Run does not compose is a rule this registry claims and nothing \
             enforces"
        );
    }

    /// Each row cites either a versioned record or a named prose authority, never a bare
    /// literal.
    ///
    /// A row is authored, so a row can be authored wrong. This is the shape a wrong one takes
    /// that the comparison above cannot see: a correct identifier beside an invented contract
    /// string, or a prose authority handed a version number it cannot have. `nomos gate
    /// explain` prints this pair straight to a person asking what a finding is grounded in.
    #[test]
    fn Test_Every_Offering_Should_Cite_A_Real_Authority()
    {
        for (rule, record, version) in OFFERINGS
        {
            assert!(!record.is_empty(), "{rule} cites an empty contract record");

            let is_prose = *record == PORTED_STANDARD || *record == WORKSPACE_CONVENTIONS;
            if is_prose
            {
                assert_eq!(
                    *version, NO_VERSIONED_RECORD,
                    "{rule} cites the prose authority {record} at version {version}, but prose \
                     carries no version for a citation to be right or wrong about"
                );
            }
            else
            {
                assert!(
                    *version > NO_VERSIONED_RECORD,
                    "{rule} cites the record {record} at the no-version sentinel; a record has \
                     front matter, and tests/contract/tests/rule_contract_citation.rs checks it"
                );
            }
        }
    }
}
