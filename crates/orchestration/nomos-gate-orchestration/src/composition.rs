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
//! than counting them, because a count agrees with itself. `P13-CONTROLFLOW-REACHABILITY-WIRE`
//! composed the fourth the same way, and `OD-GATE-019-REGISTRY-COHERENCE-A` the fifth:
//! `nomos-check-orchestration::run_context::RULE_COUNT` had already reached eight while this
//! registry stayed at four, so a caller reading it concluded dependency completeness was
//! unenforced when every `nomos check` enforces it too. `DEPENDENCY_COMPLETENESS` shares
//! `DEPENDENCY_DIRECTION`'s own `DEPENDENCY_CONTRACT_RECORD` -- both judge the same declared
//! architecture -- so this offer needed no new citable record, unlike the three
//! `OD-GATE-019-REGISTRY-COHERENCE-B` still has to compose.

use nomos_contracts::RuleId;
use nomos_rules::{
    RuleOffer, RuleRegistry, RuleRegistryError, COMPLETENESS_MIRROR, CONTRACT_RECORD,
    CONTRACT_RECORD_VERSION, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION, NAMING_CONVENTION,
    UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};

/// Registers this workspace's five shipped rules and hands back the registry.
///
/// The five are exactly what `nomos-check-orchestration::run::Run` calls today, and that is
/// the property this function exists to keep true rather than a coincidence to note --
/// `LINT_DIAGNOSTICS`, `DEPENDENCY_POLICY` and `CROSS_LANGUAGE_CORRESPONDENCE` still run
/// through `Run` uncomposed here, left for `OD-GATE-019-REGISTRY-COHERENCE-B`.
///
/// # Errors
///
/// [`RuleRegistryError::AlreadyOffered`] if the same [`RuleId`] were offered twice. Not
/// reachable today -- `COMPLETENESS_MIRROR`, `DEPENDENCY_DIRECTION`, `DEPENDENCY_COMPLETENESS`,
/// `NAMING_CONVENTION` and `UNREAD_REACHES_FINDING` are distinct constants -- but returned
/// rather than unwound for the same reason `nomos_check_orchestration::composition::Registered`
/// returns its own `RegistryError`: a composition root's own defect must be representable, not
/// panicked past.
pub fn Registered() -> Result<RuleRegistry, RuleRegistryError>
{
    let mut registry = RuleRegistry::New();

    Offer_Completeness_Mirror(&mut registry)?;
    Offer_Dependency_Direction(&mut registry)?;
    Offer_Dependency_Completeness(&mut registry)?;
    Offer_Naming_Convention(&mut registry)?;
    Offer_Unread_Reaches_A_Finding(&mut registry)?;

    return Ok(registry);
}

fn Offer_Completeness_Mirror(registry: &mut RuleRegistry) -> Result<(), RuleRegistryError>
{
    registry.Offer(RuleOffer {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        contract_record: CONTRACT_RECORD.to_owned(),
        contract_record_version: CONTRACT_RECORD_VERSION,
    })?;

    return Ok(());
}

/// `Check_Dependency_Direction` cites `OD-RULES-003` through constants beside the rule, the
/// shape `mirror.rs` already uses, rather than a literal here. A version literal in a
/// composition root drifts silently against the record it names; a version beside the
/// implementation is where whoever amends the record is already reading, and
/// `tests/contract/tests/rule_contract_citation.rs` checks both citations against the
/// records' own front matter.
fn Offer_Dependency_Direction(registry: &mut RuleRegistry) -> Result<(), RuleRegistryError>
{
    registry.Offer(RuleOffer {
        rule: RuleId::New(DEPENDENCY_DIRECTION),
        contract_record: DEPENDENCY_CONTRACT_RECORD.to_owned(),
        contract_record_version: DEPENDENCY_CONTRACT_RECORD_VERSION,
    })?;

    return Ok(());
}

/// `Check_Every_Member_Declares_A_Band` cites the same `OD-RULES-003` as
/// `Check_Dependency_Direction` above -- both judge the same declared architecture and the
/// same observed `nomos.cap.dependency.edges` fact, so this offer needs no citation of its
/// own beyond the one `DEPENDENCY_CONTRACT_RECORD` already carries.
fn Offer_Dependency_Completeness(registry: &mut RuleRegistry) -> Result<(), RuleRegistryError>
{
    registry.Offer(RuleOffer {
        rule: RuleId::New(DEPENDENCY_COMPLETENESS),
        contract_record: DEPENDENCY_CONTRACT_RECORD.to_owned(),
        contract_record_version: DEPENDENCY_CONTRACT_RECORD_VERSION,
    })?;

    return Ok(());
}

/// `Check_Naming_Convention` has no `CONTRACT_RECORD` the way `Check_Completeness_Mirrors`
/// cites `D-134` -- `naming.rs`'s own "# Why this has no `CONTRACT_RECORD`" section says its
/// contract is `README.md`'s Conventions section, prose rather than a versioned record
/// `RuleOffer::contract_record_version` could cite meaningfully. `contract_record_version:
/// 0` marks that absence -- "no versioned record", not "version zero of one" -- local to
/// this one construction site. It does not change what `RuleOffer`'s fields mean generally:
/// `OD-RULES-005` and `OD-RULES-006` already declined to extend `RuleOffer`'s shape without
/// a second real case forcing it, and a record-less rule needing its own representation is
/// that second case, left for whoever next needs more than a sentinel here to say so.
fn Offer_Naming_Convention(registry: &mut RuleRegistry) -> Result<(), RuleRegistryError>
{
    registry.Offer(RuleOffer {
        rule: RuleId::New(NAMING_CONVENTION),
        contract_record: "README.md".to_owned(),
        contract_record_version: 0,
    })?;

    return Ok(());
}

/// `Check_Unread_Reaches_A_Finding` cites `OD-RULES-008` through constants beside the rule,
/// the identical shape `DEPENDENCY_DIRECTION`'s citation above already uses.
fn Offer_Unread_Reaches_A_Finding(registry: &mut RuleRegistry) -> Result<(), RuleRegistryError>
{
    registry.Offer(RuleOffer {
        rule: RuleId::New(UNREAD_REACHES_FINDING),
        contract_record: UNREAD_REACHES_FINDING_CONTRACT_RECORD.to_owned(),
        contract_record_version: UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
    })?;

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::Registered;
    use nomos_contracts::RuleId;
    use nomos_rules::{
        COMPLETENESS_MIRROR, DEPENDENCY_COMPLETENESS, DEPENDENCY_DIRECTION, NAMING_CONVENTION,
        UNREAD_REACHES_FINDING,
    };

    /// The whole registry, by identity and in `RuleId` order -- the same discipline
    /// `crate::tests::Test_Registered_Should_Compose_All_Four_Shipped_Rules` (over `Run`'s own
    /// output) already keeps, asserted here directly against `Registered` itself.
    #[test]
    fn Test_Registered_Should_Offer_All_Five_Shipped_Rules()
    {
        let registry = Registered().expect("this crate's own registration must not be contradictory");

        let ids: Vec<RuleId> = registry.Offers().map(|offer| return offer.rule.clone()).collect();
        assert_eq!(
            ids,
            vec![
                RuleId::New(COMPLETENESS_MIRROR),
                RuleId::New(DEPENDENCY_COMPLETENESS),
                RuleId::New(DEPENDENCY_DIRECTION),
                RuleId::New(NAMING_CONVENTION),
                RuleId::New(UNREAD_REACHES_FINDING),
            ],
            "in RuleId order: {ids:?}"
        );
    }
}
