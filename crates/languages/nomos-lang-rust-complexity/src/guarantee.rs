//! What this offer promises, stated as a value and checked in both directions.
//!
//! Downward, `nomos_capability::Registry` refuses [`Provider_Offer`] if it claims more than
//! `nomos_cap_complexity::Ceiling` permits. Upward, `tests/guarantee.rs` exercises every axis
//! against what the provider actually emits, which is what `OD-CAPABILITY-016` requires of a
//! declaring crate.

use nomos_cap_complexity::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This implementation, named for what it computes the way `nomos.lang.rust.rollup` is: a
/// caller comparing two complexity answers needs to see which method produced each.
pub const PROVIDER: &str = "nomos.lang.rust.complexity";

/// What this offer claims, on every axis.
///
/// # Syntactic
///
/// `OD-ROADMAP-004` names the level in terms: a count read from text is `Syntactic`. Nothing here
/// resolves a name, so a call into a helper full of branches adds nothing to its caller, and a
/// macro is not expanded.
///
/// # Sound
///
/// Every function reported was in the parse tree, and every decision point counted is one `syn`
/// parsed. A value can be low -- see below -- but a macro cannot make one high, so a function this
/// provider reports over a limit really is over it.
///
/// # Completeness Unsound, and known rather than merely undetermined
///
/// A branch inside a macro invocation is not counted and a function a macro defines is not
/// reported: both are real cases this reading never sees, by construction. That is a
/// characterized omission, which is what `Unsound` states; `Unknown` would claim nobody knows.
///
/// # File
///
/// [`crate::Read_Complexity`] takes one file's text and nothing else.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File);
}

/// This crate's offer against `nomos_cap_complexity::Capability_Contract`.
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_complexity::Capability_Contract;
    use nomos_capability::{OfferRefusal, Registry, RegistryError, RegistryErrorKind, Requirement};

    fn Declared_Registry() -> Registry
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("the contract is the first declaration in a fresh registry");
        return registry;
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        let mut registry = Declared_Registry();

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    #[test]
    fn Test_Claiming_Runtime_Observation_Should_Be_Refused()
    {
        let mut registry = Declared_Registry();
        let overclaim = ProviderOffer {
            guarantee: Guarantee::New(FactVariant::RuntimeObserved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File),
            ..Provider_Offer()
        };

        assert_eq!(
            registry.Offer(overclaim),
            Err(RegistryError {
                capability: Capability(),
                kind: RegistryErrorKind::Offer { provider: ProviderId::New(PROVIDER), refusal: OfferRefusal::ExceedsCeiling },
            })
        );
    }

    /// A caller needing completeness assured -- every branch counted -- is not served by an offer
    /// that states a macro hides some.
    #[test]
    fn Test_A_Caller_Needing_Completeness_Should_Not_Resolve_To_This_Offer()
    {
        let mut registry = Declared_Registry();
        registry.Offer(Provider_Offer()).expect("within the ceiling");
        let needs_completeness = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);

        let resolved = registry.Resolve(&Requirement::New(Capability(), CONTRACT_VERSION, needs_completeness));

        assert!(resolved.Offer().is_none(), "{resolved:?}");
    }
}
