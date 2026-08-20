// The two error types are the registry's own vocabulary and nobody else raises them, so
// they live beneath it rather than beside it.
mod error;
mod error_kind;

pub use error::RegistryError;
pub use error_kind::RegistryErrorKind;

// The required-naming vocabulary (`RequiredUnmet`, `RequiredResolution`) and the two
// responsibilities `Registry` itself carries (declaring/offering, and resolving) each keep
// their own file; only the type's own struct definition and its methods' documentation and
// signatures stay here, which is the one file the crate's public-surface reader resolves
// `Registry` against.
mod required_unmet;
mod required_resolution;
mod declaring;
mod resolving;

pub use required_unmet::RequiredUnmet;
pub use required_resolution::RequiredResolution;

use crate::CapabilityContract;
use crate::ProviderOffer;
use crate::Requirement;
use crate::Resolution;
use nomos_contracts::{CapabilityId, ProviderId};
use std::collections::BTreeMap;

/// Capability contracts and the offers against them.
///
/// Ordered maps throughout. Resolution picks a provider, and a provider picked by hash
/// order is a run whose results depend on the hasher.
#[derive(Debug, Default)]
pub struct Registry
{
    declared: BTreeMap<CapabilityId, CapabilityContract>,
    offers: BTreeMap<CapabilityId, Vec<ProviderOffer>>,
}

impl Registry
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    /// # Errors
    ///
    /// Returns [`RegistryErrorKind::AlreadyDeclared`] if the capability already has a
    /// contract. Two contracts for one capability is two meanings for one name.
    pub fn Declare(&mut self, contract: CapabilityContract) -> Result<(), RegistryError>
    {
        return declaring::Declare(self, contract);
    }

    /// # Errors
    ///
    /// Returns [`RegistryError`] if the capability is undeclared, the provider already
    /// offers it, or the offer claims more than the contract's ceiling.
    pub fn Offer(&mut self, offer: ProviderOffer) -> Result<(), RegistryError>
    {
        return declaring::Offer(self, offer);
    }

    /// Declares `contract` and registers `offer` against it, in that order.
    ///
    /// The pairing rather than a convenience. A contract nothing offers against resolves to
    /// nothing, so every composition in this workspace declares and then immediately offers;
    /// written out, that pair was the same two statements repeated at every composition
    /// root and in every test that builds one. The order is the whole reason it cannot be
    /// two calls a caller might transpose: [`Offer`](Self::Offer) refuses an undeclared
    /// capability.
    ///
    /// A second provider against the same contract is still [`Offer`](Self::Offer). This
    /// declares, so calling it twice for one capability is `AlreadyDeclared` and says so.
    ///
    /// # Errors
    ///
    /// Whatever [`Declare`](Self::Declare) or [`Offer`](Self::Offer) returns, unchanged: the
    /// contract is left declared if the offer is the half that is refused, because a partial
    /// registration a caller can inspect is more use than one this rolled back silently.
    pub fn Declare_And_Offer(
        &mut self,
        contract: CapabilityContract,
        offer: ProviderOffer,
    ) -> Result<(), RegistryError>
    {
        self.Declare(contract)?;

        return self.Offer(offer);
    }

    /// Answers a requirement.
    ///
    /// Never returns a bare no. Every path either names a provider or names what is
    /// missing, and no path produces [`nomos_contracts::Applicability::NotApplicable`].
    ///
    /// # Which of several usable offers answers
    ///
    /// The floor decides which offers are *usable*; [`Selection`](crate::Selection) decides
    /// which usable offer is *chosen*, and the rule is that no usable offer is strictly
    /// stronger than the one that answers. A named preference outranks that rule, and every
    /// offer the rule passed over comes back in the selection. Nothing here consults a
    /// provider's name except to break a tie the guarantee itself does not break — see
    /// [`crate::Selection::Unranked`], which is how a caller tells that apart from a decision.
    ///
    /// This used to be `offers.find(usable)` over a name-sorted list, which resolved to
    /// whichever provider sorted first. `docs/records/OD-CAPABILITY-001` records what
    /// decided it.
    #[must_use]
    pub fn Resolve(&self, requirement: &Requirement) -> Resolution
    {
        return resolving::Resolve(self, requirement);
    }

    /// Answers a requirement whose named provider is required rather than preferred.
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
    #[must_use]
    pub fn Resolve_Requiring(
        &self,
        requirement: &Requirement,
        required: &ProviderId,
    ) -> RequiredResolution
    {
        return resolving::Resolve_Requiring(self, requirement, required);
    }

    pub fn Declared(&self) -> impl Iterator<Item = &CapabilityContract>
    {
        return self.declared.values();
    }

    #[must_use]
    pub fn Offers(&self, capability: &CapabilityId) -> &[ProviderOffer]
    {
        return self
            .offers
            .get(capability)
            .map_or(&[] as &[ProviderOffer], Vec::as_slice);
    }
}

#[cfg(test)]
mod tests;
