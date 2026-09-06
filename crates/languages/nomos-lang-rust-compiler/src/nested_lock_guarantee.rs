//! What this crate's second provider offers, and at what guarantee.
//!
//! The same engine as [`crate::guarantee`] -- [`crate::guarantee::PROVIDER`] is reused
//! rather than a second `ProviderId` minted, because `nomos.lang.rust.compiler` names the
//! `ra_ap_hir`-backed engine itself, not either single capability it happens to answer
//! today. [`nomos_capability::ProviderOffer`] already pairs a provider with a capability
//! per offer, so one provider offering two capabilities needs two offers, not two names.

use crate::guarantee::PROVIDER;
use crate::nested_lock_contract::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// What this provider claims, on every axis, for `nomos.cap.rust.nested_locks`.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: every finding this provider
/// reports comes from `ra_ap_hir::Semantics::resolve_type` and `Type::as_adt` /
/// `Type::type_arguments`, all of which resolve names -- including through a type alias
/// -- rather than read syntax.
///
/// Soundness [`Assurance::Sound`]: a finding this provider reports names an outer lock
/// `ra_ap_hir` actually resolved and an inner type argument it actually resolved to
/// another lock type; nothing here infers one from spelling.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` -- this provider only
/// walks `ast::Type` nodes syntax itself already contains; a nesting that only exists
/// after monomorphizing a generic parameter with a lock type at some call site this
/// analysis never visits (the struct field itself stays generic, e.g. `struct S<T> { l:
/// Mutex<T> }` instantiated elsewhere as `S<Mutex<i32>>`) is not a case this provider can
/// honestly claim it would have caught -- the same "no floor no provider can honestly
/// meet" reasoning [`crate::guarantee::Declared_Guarantee`] already gives for a different
/// underlying reason.
///
/// [`IncrementalGranularity::Project`]: [`crate::nested_lock_contract::Ceiling`] states
/// why one analyzed crate is the unit that must recompute together.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`crate::nested_lock_contract::Capability_Contract`].
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

    /// The offer must satisfy the contract's own ceiling -- the same check
    /// `Registry::Offer` runs at composition time, asserted here in the same direction
    /// `crate::guarantee`'s own test already asserts for the crate's first capability.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use crate::nested_lock_contract::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use crate::nested_lock_contract::Capability_Contract;
        use nomos_capability::Registry;

        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
