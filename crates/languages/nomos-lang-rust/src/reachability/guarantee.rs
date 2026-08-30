//! What this crate's second capability offer promises, stated as a value and checked in
//! two directions — the same split `crate::guarantee` draws for the first.

use nomos_cap_controlflow::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// What this offer claims, on every axis.
///
/// # Syntactic, below the capability's own ceiling
///
/// `nomos_cap_controlflow::Ceiling()` states `SemanticallyResolved` because a *sound*
/// answer needs every call an `Err` arm reaches resolved — `OD-RULES-008`'s own tier 2.
/// This offer does none of that; it pattern-matches one arm's own body, nothing past it.
///
/// # Sound, on what it flags
///
/// Every recorded site is a `match` arm whose pattern really is `Err(applicability)` and
/// whose body really is one of [`nomos_cap_controlflow::ArmShape`]'s four shapes — `syn`
/// parsed the tree, so there is no path by which this offer reports a shape the source
/// does not contain.
///
/// # Completeness Unsound, and known rather than merely undetermined
///
/// Not [`Assurance::Unknown`], which `crate::Declared_Guarantee` claims for its own
/// completeness because nobody has bounded how much a macro hides. This offer's
/// incompleteness is established by its own construction: an `if let Err(applicability)`
/// outside a `match`, a differently spelled binding, or any arm body this reading's four
/// patterns do not name are all real cases this offer will never see, by design rather
/// than by an unmeasured gap. `nomos_cap_controlflow::ReachabilitySite`'s own doc states
/// the same thing from the payload's side: absence is not a claim of cleanliness.
///
/// # File
///
/// The identical argument `crate::Declared_Guarantee` gives: `syn` parses one file's bytes
/// or fails, with no partial reparse a coarser or finer claim could ride on.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unsound,
        IncrementalGranularity::File,
    );
}

/// This crate's offer against `nomos_cap_controlflow::Capability_Contract`.
///
/// [`crate::PROVIDER`] is reused rather than a second constant declared here: the tool
/// behind both offers is the same `syn` parse of the same file, and a caller comparing two
/// answers from this crate needs to see that they came from one method, not two.
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(crate::PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_controlflow::Capability_Contract;
    use nomos_capability::{OfferRefusal, Registry, RegistryError, RegistryErrorKind, Requirement};

    #[test]
    fn Test_Declared_Guarantee_Should_State_A_Sound_Pattern_Unsound_Completeness_Claim()
    {
        assert_eq!(
            Declared_Guarantee(),
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File)
        );
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    /// The reason the ceiling exists, one tier past it rather than at it: the capability's
    /// own ceiling is already `SemanticallyResolved` — `OD-RULES-008`'s sound tier — so
    /// what this offer must not be able to claim is `RuntimeObserved`, watching every path
    /// actually execute, which no static reader does.
    #[test]
    fn Test_Claiming_Runtime_Observation_Should_Be_Refused()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        let refused = registry.Offer(Offer_Claiming_Runtime_Observation());

        Assert_Refused_For_Exceeding_Ceiling(&refused);
    }

    /// An offer identical to [`Provider_Offer`] except its fact variant claims
    /// `RuntimeObserved` — one tier past the capability's own ceiling.
    fn Offer_Claiming_Runtime_Observation() -> ProviderOffer
    {
        return ProviderOffer {
            guarantee: Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };
    }

    fn Assert_Refused_For_Exceeding_Ceiling(refused: &Result<(), RegistryError>)
    {
        assert_eq!(
            *refused,
            Err(RegistryError {
                capability: Capability(),
                kind: RegistryErrorKind::Offer {
                    provider: ProviderId::New(crate::PROVIDER),
                    refusal: OfferRefusal::ExceedsCeiling,
                },
            })
        );
    }

    /// A rule stating exactly what this offer delivers resolves to it — the floor
    /// `Check_Unread_Reaches_A_Finding` actually asks for, per this item's own `why`.
    #[test]
    fn Test_A_Caller_Needing_Only_What_This_Offer_Delivers_Should_Resolve_To_It()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry.Offer(Provider_Offer()).expect("within the ceiling");

        let need = Requirement::New(Capability(), CONTRACT_VERSION, Declared_Guarantee());
        let resolved = registry.Resolve(&need);

        assert_eq!(
            resolved.Offer().map(|offer| return offer.provider.clone()),
            Some(ProviderId::New(crate::PROVIDER))
        );
    }

    /// A rule that needs completeness assured cannot be served by an offer that says its
    /// own completeness is `Unsound` — the honest consequence of the axis above.
    #[test]
    fn Test_A_Caller_Needing_Completeness_Should_Not_Resolve_To_This_Offer()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry.Offer(Provider_Offer()).expect("within the ceiling");

        let needs_completeness = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );
        let need = Requirement::New(Capability(), CONTRACT_VERSION, needs_completeness);

        assert!(
            registry.Resolve(&need).Offer().is_none(),
            "an offer that states its own completeness as Unsound must not satisfy a \
             caller that requires completeness"
        );
    }
}
