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
//! This *is* a shared derivation now, and the reason this paragraph used to give for the debt
//! was the part that went stale: a rule's contract citation is knowledge an export does carry
//! -- `RuleDescriptor::contract_record` and `contract_record_version`, on the same public
//! `nomos_rules::DESCRIPTORS` table this crate registers from. [`Registered`] reads both
//! straight off each descriptor and types no row of its own, so there is no hand-authored list
//! here to keep in step, and the "one hand-typed list and one comparison against it" that
//! `OD-GATE-020` described as the fix was the shape of the step before this one. The comparison
//! survives as the test below, which checks what is offered against `Composed_Rules` rather
//! than against a list written out here.
//!
//! `OD-RULES-027` measured what remained after that, and decided it rather than leaving it to
//! be rediscovered: `Run`'s own seventy-entry array can be derived the same way, that derivation
//! is available and is not `OD-RULES-009`'s demand planner, and building it is a capability
//! item's territory rather than this file's. Until that lands, `Run` is the side of this parity
//! still hand-typed, and the test below is what keeps the two from diverging quietly.
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
use crate::RuleCompositionError;
use nomos_rules::{RuleOffer, RuleRegistry};


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
/// [`RuleCompositionError::Resolution`] when what this build declares and what it composes are
/// not the same set, and [`RuleCompositionError::Registration`] if one [`RuleId`] is described
/// twice. Neither is reachable today -- `Declared_Rules` is derived from the same
/// `nomos_rules::DESCRIPTORS` this function registers from, and that table's own uniqueness
/// test would fail first -- but both are returned rather than unwound for the same reason
/// `nomos_check_orchestration::Registered` returns its own `RegistryError`: a composition
/// root's own defect must be representable, not panicked past.
///
/// The resolution runs before the registry is built, so a disagreement refuses instead of
/// producing a plan the run would not honour.
pub fn Registered() -> Result<RuleRegistry, RuleCompositionError>
{
    nomos_check_orchestration::Resolve_Rules(
        &nomos_check_orchestration::Declared_Rules(),
        &nomos_check_orchestration::Composed_Rules(),
    )?;

    let mut registry = RuleRegistry::New();

    for descriptor in nomos_rules::DESCRIPTORS
    {
        registry.Offer(RuleOffer {
            rule: descriptor.Rule(),
            contract_record: descriptor.contract_record.to_owned(),
            contract_record_version: descriptor.contract_record_version,
        })?;
    }

    return Ok(registry);
}

#[cfg(test)]
mod tests
{
    use super::Registered;
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
            "every rule nomos-check-orchestration::Run composes needs a descriptor, and \
             a row here that Run does not compose is a rule this registry claims and nothing \
             enforces"
        );
    }
}
