//! The rule registry this crate composes.
//!
//! `nomos_rules::RuleRegistry` has existed since `OD-RULES-004` and its own module doc says
//! plainly that nothing consults it: "Nothing here is consulted by `Run()`, and nothing here
//! changes what runs on any given `nomos check`." This is that registry's first real
//! consumer -- not a change to what `nomos check` runs, which stays exactly what
//! `nomos-check-orchestration::run::Run` already does, but a second, honest answer to "what
//! rules exist" built from the same registration seam rather than by re-deriving the list.

use nomos_contracts::RuleId;
use nomos_rules::{RuleOffer, RuleRegistry, RuleRegistryError, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION, NAMING_CONVENTION};

/// Registers this workspace's two shipped rules and hands back the registry.
///
/// # Errors
///
/// [`RuleRegistryError::AlreadyOffered`] if the same [`RuleId`] were offered twice. Not
/// reachable today -- `COMPLETENESS_MIRROR` and `NAMING_CONVENTION` are distinct constants
/// -- but returned rather than unwound for the same reason
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
