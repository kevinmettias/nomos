//! Answering a requirement: which offer satisfies it, or why none does.
//!
//! The bodies of [`super::Registry::Resolve`] and [`super::Registry::Resolve_Requiring`],
//! which keep their documentation and signature on the type in `registry.rs` — the one file
//! the crate's public-surface reader resolves `Registry` against.

use crate::ProviderOffer;
use crate::Requirement;
use crate::Resolution;
use crate::Selection;
use crate::Unmet;
use nomos_contracts::Applicability;
use nomos_contracts::ProviderId;

use super::Registry;
use super::required_resolution::RequiredResolution;
use super::required_unmet::RequiredUnmet;

/// The body of [`Registry::Resolve`].
///
/// Never returns a bare no. Every path either names a provider or names what is
/// missing, and no path produces [`Applicability::NotApplicable`].
///
/// # Which of several usable offers answers
///
/// The floor decides which offers are *usable*; [`Selection`] decides which usable
/// offer is *chosen*, and the rule is that no usable offer is strictly stronger than
/// the one that answers. A named preference outranks that rule, and every offer the
/// rule passed over comes back in the selection. Nothing here consults a provider's
/// name except to break a tie the guarantee itself does not break — see
/// [`Selection::Unranked`], which is how a caller tells that apart from a decision.
///
/// This used to be `offers.find(usable)` over a name-sorted list, which resolved to
/// whichever provider sorted first. `docs/records/OD-CAPABILITY-001` records what
/// decided it.
pub(super) fn Resolve_Requirement(registry: &Registry, requirement: &Requirement) -> Resolution
{
    let selection = match Selected_Offer(registry, requirement)
    {
        Ok(selection) => selection,
        Err(reason) => return Resolution::Unsatisfied {
            capability: requirement.capability.clone(),
            reason,
        },
    };
    let applicability = Honoured_Preference(requirement, &selection);

    return Resolution::Satisfied {
        selection,
        applicability,
    };
}

/// Whether the answer came from the provider the caller asked for.
///
/// A named preference that was not honoured is what makes this a fallback. The judgment
/// still stands; its provenance is not what was asked for, and a caller that cannot tell
/// cannot record why.
///
/// Compared against who actually answered rather than against a second search for the
/// preference, because those are the same question and asking it twice is how the two
/// answers come to disagree.
fn Honoured_Preference(requirement: &Requirement, selection: &Selection) -> Applicability
{
    let Some(preferred) = &requirement.preferred
    else
    {
        return Applicability::Supported;
    };

    if *preferred == selection.chosen.provider
    {
        return Applicability::Supported;
    }

    return Applicability::SupportedWithFallback;
}

/// The body of [`Registry::Resolve_Requiring`].
///
/// `Resolve` treats a name as a preference: an answer from anybody else still satisfies
/// the requirement, reported through `Applicability::SupportedWithFallback` so a caller
/// that reads that far can tell. This treats the same name as the whole point. An offer
/// from anybody but `required` is refused rather than substituted, and the refusal names
/// both `required` and, when somebody else was usable, who that was.
///
/// This does not let the registry decide anything `OD-CAPABILITY-001` reserved to the
/// caller — the guarantee still ranks and a caller still spends. What changed is not who
/// ranks; it is that `required` forces the ranking's head the way a preference already
/// could, and a caller who cannot be answered by anyone else is told so instead of being
/// handed a fallback it did not ask for. `docs/records/OD-CAPABILITY-005` records why
/// refusing is right here though `OD-CAPABILITY-001` declined to let the registry refuse
/// on the caller's behalf: there, no caller had said which provider it would accept:
/// here, one has.
pub(super) fn Resolve_Requiring(
    registry: &Registry,
    requirement: &Requirement,
    required: &ProviderId,
) -> RequiredResolution
{
    let scoped = Scoped_To(requirement, required);

    let selection = match Selected_Offer(registry, &scoped)
    {
        Ok(selection) => selection,
        Err(reason) => return RequiredResolution::Unsatisfied {
            capability: requirement.capability.clone(),
            reason: RequiredUnmet::Unavailable(reason),
        },
    };

    return Settle_Required(requirement, required, selection);
}

/// `requirement`, with its preference pinned to `required` — `Resolve_Requiring`'s way of
/// reusing `Selected`'s own ranking rather than searching a second time.
fn Scoped_To(requirement: &Requirement, required: &ProviderId) -> Requirement
{
    return Requirement {
        preferred: Some(required.clone()),
        ..requirement.clone()
    };
}

/// Whether `selection` answered from `required` itself, or from somebody else — the
/// distinction `Resolve_Requiring` exists to draw once `Selected` has already ranked the
/// offers.
fn Settle_Required(requirement: &Requirement, required: &ProviderId, selection: Selection) -> RequiredResolution
{
    if &selection.chosen.provider == required
    {
        return RequiredResolution::Satisfied { selection };
    }

    return RequiredResolution::Unsatisfied {
        capability: requirement.capability.clone(),
        reason: RequiredUnmet::AnsweredByOther {
            required: required.clone(),
            answered: selection.chosen.provider.clone(),
        },
    };
}

/// Which offer answers, or which of the four things was missing.
///
/// The four are distinct on purpose: nobody declared it, it was declared at a version
/// this caller cannot read, nobody offers it, and everyone who offers it is below the
/// floor. Collapsing any two of them would leave a caller unable to tell a composition
/// mistake from a missing dependency.
fn Selected_Offer(registry: &Registry, requirement: &Requirement) -> Result<Selection, Unmet>
{
    if let Some(reason) = Unreadable_Contract(registry, requirement)
    {
        return Err(reason);
    }

    let offers = Offers_For(registry, requirement);
    let Some(first) = offers.first()
    else
    {
        return Err(Unmet::NoProvider);
    };

    let usable = Usable_Offers(offers, requirement);

    return Selection::Over(usable, requirement.preferred.as_ref()).ok_or_else(|| {
        return Unmet::BelowRequirement {
            closest: first.provider.clone(),
        };
    });
}

/// Every offer standing against a capability, in name order.
///
/// An empty slice rather than an absence, because "nobody offers this" and "this
/// capability has no entry yet" are the same answer to the caller and giving them two
/// shapes would make the caller decide which.
fn Offers_For<'registry>(registry: &'registry Registry, requirement: &Requirement) -> &'registry [ProviderOffer]
{
    return registry
        .offers
        .get(&requirement.capability)
        .map_or(&[] as &[ProviderOffer], Vec::as_slice);
}

/// Why the contract itself cannot answer this requirement, if it cannot.
///
/// Both answers are about the capability rather than about any offer: it was never
/// declared, or it was declared at a version this caller cannot read. Neither depends
/// on who is offering, which is why they are asked before the offers are looked at.
#[cfg(test)]
mod tests
{
    use super::*;
    use crate::CapabilityContract;
    use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity};

    #[test]
    fn Test_Resolve_Requirement_Should_Report_No_Provider_For_A_Declared_But_Unoffered_Capability()
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");

        let resolution = Resolve_Requirement(&registry, &Requirement::New(Capability(), Version(), Floor()));

        assert!(matches!(resolution, Resolution::Unsatisfied { reason: Unmet::NoProvider, .. }));
    }

    #[test]
    fn Test_Resolve_Requiring_Should_Refuse_A_Requirement_Nobody_Named_Answers()
    {
        let registry = Registry_With_One_Offer();
        let required = ProviderId::New("nomos.test.absent");

        let resolution =
            Resolve_Requiring(&registry, &Requirement::New(Capability(), Version(), Floor()), &required);

        assert!(matches!(resolution, RequiredResolution::Unsatisfied { .. }));
    }

    fn Registry_With_One_Offer() -> Registry
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");
        registry.Offer(Offer_From("nomos.test.resolving")).expect("within the ceiling");

        return registry;
    }

    fn Offer_From(provider: &str) -> ProviderOffer
    {
        return ProviderOffer {
            provider: ProviderId::New(provider),
            capability: Capability(),
            version: Version(),
            guarantee: Floor(),
        };
    }

    fn Capability() -> CapabilityId
    {
        return CapabilityId::New("nomos.cap.test.resolving");
    }

    fn Version() -> ContractVersion
    {
        return ContractVersion::New(1, 0);
    }

    fn Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Contract() -> CapabilityContract
    {
        return CapabilityContract {
            id: Capability(),
            version: Version(),
            summary: "a contract for resolving.rs's own tests".to_owned(),
            ceiling: Floor(),
        };
    }
}

fn Unreadable_Contract(registry: &Registry, requirement: &Requirement) -> Option<Unmet>
{
    let Some(contract) = registry.declared.get(&requirement.capability)
    else
    {
        return Some(Unmet::Undeclared);
    };

    if !requirement.version.Can_Read(contract.version)
    {
        return Some(Unmet::VersionMismatch {
            offered: contract.version,
        });
    }

    return None;
}

/// What resolution says about a capability nobody installed, pinned.
///
/// These assert the half of `P11-KNOWLEDGE-ABSENT` that belongs here, and they are in `src`
/// rather than in `tests/` because they are about this type's own vocabulary — the same
/// reason `selection.rs` keeps its own.
///
/// The other half deliberately has no test here, because it has no mechanism here: whether
/// a provider that answered found anything *relevant* is a statement about content, and a
/// registry that could report it would be grading a provider's work. `OD-CAPABILITY-004`
/// carries that argument and says where the combined distinction lives instead.
/// Every offer this requirement could actually use: readable at its version, and at or
/// above its floor.
fn Usable_Offers(offers: &[ProviderOffer], requirement: &Requirement) -> Vec<ProviderOffer>
{
    return offers
        .iter()
        .filter(|offer| {
            return requirement.version.Can_Read(offer.version)
                && offer.guarantee.Satisfies(&requirement.minimum);
        })
        .cloned()
        .collect();
}
