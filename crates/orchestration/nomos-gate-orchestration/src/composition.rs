//! The rule registry this crate composes.
//!
//! `nomos_rules::RuleRegistry` has existed since `OD-RULES-004` and its own module doc says
//! plainly that nothing consults it: "Nothing here is consulted by `Run()`, and nothing here
//! changes what runs on any given `nomos check`." This is that registry's first real
//! consumer -- not a change to what `nomos check` runs, which stays exactly what
//! `nomos-check-orchestration::run::Run` already does, but a second, honest answer to "what
//! rules exist" built from the same registration seam rather than by re-deriving the list.
//!
//! Honest requires whole. This registry composed two of three rules between
//! `P13-DEPENDENCY-WIRE-1` and `P13-GATE-REGISTRY-THIRD-RULE`, and a plan smaller than the run
//! it describes is worse than no plan: a caller reading it concludes dependency direction is
//! unenforced when every `nomos check` enforces it. `crate::tests` asserts the offers rather
//! than counting them, because a count agrees with itself.

use nomos_contracts::RuleId;
use nomos_rules::{RuleOffer, RuleRegistry, RuleRegistryError, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION, NAMING_CONVENTION};

/// Registers this workspace's three shipped rules and hands back the registry.
///
/// The three are exactly what `nomos-check-orchestration::run::Run` calls, and that is the
/// property this function exists to keep true rather than a coincidence to note.
///
/// # Errors
///
/// [`RuleRegistryError::AlreadyOffered`] if the same [`RuleId`] were offered twice. Not
/// reachable today -- `COMPLETENESS_MIRROR`, `DEPENDENCY_DIRECTION` and `NAMING_CONVENTION`
/// are distinct constants -- but returned rather than unwound for the same reason
/// `nomos_check_orchestration::composition::Registered` returns its own `RegistryError`:
/// a composition root's own defect must be representable, not panicked past.
pub fn Registered() -> Result<RuleRegistry, RuleRegistryError>
{
    let mut registry = RuleRegistry::New();

    registry.Offer(RuleOffer {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        contract_record: CONTRACT_RECORD.to_owned(),
        contract_record_version: CONTRACT_RECORD_VERSION,
    })?;

    // `Check_Dependency_Direction` cites `OD-RULES-003` through constants beside the rule,
    // the shape `mirror.rs` already uses, rather than a literal here. A version literal in a
    // composition root drifts silently against the record it names; a version beside the
    // implementation is where whoever amends the record is already reading, and
    // `tests/contract/tests/rule_contract_citation.rs` checks both citations against the
    // records' own front matter.
    registry.Offer(RuleOffer {
        rule: RuleId::New(DEPENDENCY_DIRECTION),
        contract_record: DEPENDENCY_CONTRACT_RECORD.to_owned(),
        contract_record_version: DEPENDENCY_CONTRACT_RECORD_VERSION,
    })?;

    // `Check_Naming_Convention` has no `CONTRACT_RECORD` the way `Check_Completeness_
    // Mirrors` cites `D-134` -- `naming.rs`'s own "# Why this has no `CONTRACT_RECORD`"
    // section says its contract is `README.md`'s Conventions section, prose rather than a
    // versioned record `RuleOffer::contract_record_version` could cite meaningfully.
    // `contract_record_version: 0` marks that absence -- "no versioned record", not "version
    // zero of one" -- local to this one construction site. It does not change what
    // `RuleOffer`'s fields mean generally: `OD-RULES-005` and `OD-RULES-006` already declined
    // to extend `RuleOffer`'s shape without a second real case forcing it, and a record-less
    // rule needing its own representation is that second case, left for whoever next needs
    // more than a sentinel here to say so.
    registry.Offer(RuleOffer {
        rule: RuleId::New(NAMING_CONVENTION),
        contract_record: "README.md".to_owned(),
        contract_record_version: 0,
    })?;

    return Ok(registry);
}
