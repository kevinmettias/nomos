//! Declaring capability contracts, and registering offers against them.
//!
//! The bodies of [`super::Registry::Declare`] and [`super::Registry::Offer`], which keep
//! their documentation and signature on the type in `registry.rs` — the one file the crate's
//! public-surface reader resolves `Registry` against.

use crate::CapabilityContract;
use crate::OfferRefusal;
use crate::ProviderOffer;
use crate::RegistryError;
use crate::RegistryErrorKind;
use nomos_contracts::CapabilityId;
use nomos_contracts::ProviderId;

use super::Registry;

/// The body of [`Registry::Declare`].
pub(super) fn Declare_Contract(registry: &mut Registry, contract: CapabilityContract) -> Result<(), RegistryError>
{
    if registry.declared.contains_key(&contract.id)
    {
        return Err(RegistryError {
            capability: contract.id,
            kind: RegistryErrorKind::AlreadyDeclared,
        });
    }

    registry.declared.insert(contract.id.clone(), contract);
    return Ok(());
}

/// The body of [`Registry::Offer`].
pub(super) fn Register_Offer(registry: &mut Registry, offer: ProviderOffer) -> Result<(), RegistryError>
{
    Refuse_Unofferable(registry, &offer)?;

    let against = registry.offers.entry(offer.capability.clone()).or_default();
    if against.iter().any(|existing| existing.provider == offer.provider)
    {
        return Err(Refused_Offer(&offer.capability, &offer.provider, OfferRefusal::Duplicate));
    }

    against.push(offer);
    // Name order, which is no longer what selects. It is here so that the input to
    // [`crate::Selection`] does not depend on registration order, and so the tiebreak it
    // falls back to when the guarantee ranks nothing is at least the same tiebreak
    // every time.
    against.sort_by(|left, right| left.provider.cmp(&right.provider));
    return Ok(());
}

/// An offer against a capability that will not take it.
///
/// The ceiling is checked here rather than at resolution because it is a statement about
/// the offer and not about any requirement: a provider claiming more than the contract
/// admits is wrong whether or not anybody ever asks for that much.
fn Refuse_Unofferable(registry: &Registry, offer: &ProviderOffer) -> Result<(), RegistryError>
{
    let Some(contract) = registry.declared.get(&offer.capability)
    else
    {
        return Err(Refused_Offer(&offer.capability, &offer.provider, OfferRefusal::ForUndeclared));
    };
    if !contract.ceiling.Satisfies(&offer.guarantee)
    {
        return Err(Refused_Offer(&offer.capability, &offer.provider, OfferRefusal::ExceedsCeiling));
    }

    return Ok(());
}

/// One provider's offer, refused for a named reason.
fn Refused_Offer(capability: &CapabilityId, provider: &ProviderId, refusal: OfferRefusal) -> RegistryError
{
    return RegistryError {
        capability: capability.clone(),
        kind: RegistryErrorKind::Offer {
            provider: provider.clone(),
            refusal,
        },
    };
}
