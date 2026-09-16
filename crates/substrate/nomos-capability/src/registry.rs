// The two error types are the registry's own vocabulary and nobody else raises them, so
// they live beneath it rather than beside it.
#[path = "registry/error.rs"]
mod error;
#[path = "registry/error_kind.rs"]
mod error_kind;

pub use error::Error as RegistryError;
pub use error_kind::ErrorKind as RegistryErrorKind;

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
        return declaring::Declare_Contract(self, contract);
    }

    /// # Errors
    ///
    /// Returns [`RegistryError`] if the capability is undeclared, the provider already
    /// offers it, or the offer claims more than the contract's ceiling.
    pub fn Offer(&mut self, offer: ProviderOffer) -> Result<(), RegistryError>
    {
        return declaring::Register_Offer(self, offer);
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
        return resolving::Resolve_Requirement(self, requirement);
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

// `mod tests` above is a SEPARATE file (`registry/tests.rs`). check-test-coverage's Rust
// front end keys a test's companion unit off the literal file it is textually written in,
// so a test living in that separate file can never address a function declared here,
// however it is named. This second, LITERAL inline module gives each method here the
// one-file address the check reads, without disturbing `registry/tests.rs`'s own broader
// behavioural suite.
#[cfg(test)]
mod local_tests
{
    use super::*;
    use nomos_contracts::{Assurance, ContractVersion, FactVariant, Guarantee, IncrementalGranularity};

    #[test]
    fn Test_New_Should_Begin_Completely_Blank()
    {
        let registry = Registry::New();

        assert_eq!(registry.Declared().count(), 0);
        assert!(registry.Offers(&Capability()).is_empty());
    }

    #[test]
    fn Test_Declare_Should_Refuse_A_Second_Declaration_For_The_Same_Capability()
    {
        let mut registry = Registry::New();

        assert!(registry.Declare(Contract()).is_ok());
        assert_eq!(registry.Declare(Contract()).unwrap_err().kind, RegistryErrorKind::AlreadyDeclared);
    }

    #[test]
    fn Test_Offer_Should_Refuse_An_Offer_Against_An_Undeclared_Capability()
    {
        let mut registry = Registry::New();

        let error = registry.Offer(Offer()).unwrap_err();

        assert_eq!(
            error.kind,
            RegistryErrorKind::Offer {
                provider: Offer().provider,
                refusal: crate::OfferRefusal::ForUndeclared,
            }
        );
    }

    #[test]
    fn Test_Declare_And_Offer_Should_Do_Both_In_One_Call()
    {
        let mut registry = Registry::New();

        assert!(registry.Declare_And_Offer(Contract(), Offer()).is_ok());
        assert_eq!(registry.Offers(&Capability()).len(), 1);
    }

    #[test]
    fn Test_Resolve_Should_Report_No_Provider_For_An_Unoffered_Capability()
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");

        let requirement = Requirement::New(Capability(), Version(), Floor());
        let resolution = registry.Resolve(&requirement);

        assert!(matches!(
            resolution,
            Resolution::Unsatisfied {
                reason: crate::Unmet::NoProvider,
                ..
            }
        ));
    }

    #[test]
    fn Test_Resolve_Requiring_Should_Refuse_A_Requirement_Nobody_Named_Answers()
    {
        let mut registry = Registry::New();
        registry.Declare_And_Offer(Contract(), Offer()).expect("declared and offered");
        let absent = ProviderId::New("nomos.test.absent");

        let requirement = Requirement::New(Capability(), Version(), Floor());
        let resolution = registry.Resolve_Requiring(&requirement, &absent);

        assert!(matches!(resolution, RequiredResolution::Unsatisfied { .. }));
    }

    #[test]
    fn Test_Declared_Should_List_Every_Declared_Contract()
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");

        let declared: Vec<&CapabilityContract> = registry.Declared().collect();

        assert_eq!(declared, vec![&Contract()]);
    }

    #[test]
    fn Test_Offers_Should_Be_Empty_For_A_Capability_Nobody_Offered()
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");

        assert!(registry.Offers(&Capability()).is_empty());
        registry.Offer(Offer()).expect("within the ceiling");
        assert_eq!(registry.Offers(&Capability()).len(), 1);
    }

    fn Capability() -> CapabilityId
    {
        return CapabilityId::New("nomos.cap.test.registry_local");
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
            summary: "a contract for registry.rs's own local tests".to_owned(),
            ceiling: Floor(),
        };
    }

    fn Offer() -> ProviderOffer
    {
        return ProviderOffer {
            provider: ProviderId::New("nomos.test.registry_local"),
            capability: Capability(),
            version: Version(),
            guarantee: Floor(),
        };
    }
}
