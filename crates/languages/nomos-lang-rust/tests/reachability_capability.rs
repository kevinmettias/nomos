//! The real seam `nomos_lang_rust::reachability` reaches across into `nomos_cap_controlflow`
//! — moved here from an inline test so it is checked through the public API a real consumer
//! has, rather than through this crate's own private items.

use nomos_lang_rust::reachability::{Declared_Guarantee, Provider_Offer};

/// The happy path: this offer is accepted under `nomos_cap_controlflow`'s own contract
/// through `nomos_capability::Registry` — the composition-time check every real caller
/// relies on.
#[test]
fn Test_The_Reachability_Offer_Should_Be_Accepted_By_Nomos_Cap_Controlflows_Own_Contract()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_controlflow::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// The error that crosses the boundary: an offer claiming more than
/// `nomos_cap_controlflow`'s own ceiling permits is refused by the registry, not silently
/// accepted.
#[test]
fn Test_An_Offer_Claiming_More_Than_The_Ceiling_Should_Be_Refused()
{
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_controlflow::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    let over_ceiling = ProviderOffer {
        guarantee: Guarantee::New(
            FactVariant::RuntimeObserved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        ),
        ..Provider_Offer()
    };

    assert!(
        registry.Offer(over_ceiling).is_err(),
        "an offer above the capability's own ceiling must be refused by the registry"
    );
}

/// The lifecycle: `nomos_cap_controlflow`'s own ceiling is a real value this offer's
/// guarantee is checked against via `nomos_contracts::Guarantee::Satisfies` — the exact
/// comparison the registry runs internally.
#[test]
fn Test_The_Declared_Guarantee_Should_Satisfy_Nomos_Cap_Controlflows_Own_Ceiling()
{
    assert!(
        nomos_cap_controlflow::Ceiling().Satisfies(&Declared_Guarantee()),
        "this crate's own claim must satisfy the capability's ceiling"
    );
}
