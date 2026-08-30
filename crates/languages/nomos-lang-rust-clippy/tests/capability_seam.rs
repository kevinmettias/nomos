//! The real seam between `nomos_lang_rust_clippy` and `nomos_cap_lint` / `nomos_capability`,
//! moved here from an inline test in `src/guarantee.rs` so it is checked through the public
//! API a real consumer has, rather than through this crate's own private items — the same
//! move `nomos_lang_rust::tests::reachability_capability` already made for the identical
//! reason.
//!
//! `src/guarantee.rs`'s own inline test proved the offer against `nomos_cap_lint::Ceiling`
//! directly; it never went through `nomos_capability::Registry`, which is the actual
//! composition-time check every real caller relies on
//! (`nomos_capability::registry::declaring::Offer`). This file proves both: the registry
//! accepts this crate's offer under the real contract, and refuses one that claims more than
//! the contract's own ceiling.

use nomos_lang_rust_clippy::{Declared_Guarantee, Provider_Offer};

/// The happy path: this offer is accepted under `nomos_cap_lint`'s own contract through
/// `nomos_capability::Registry` — the composition-time check every real caller relies on.
#[test]
fn Test_This_Crates_Offer_Should_Be_Accepted_By_Nomos_Cap_Lints_Own_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_lint::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// The error that crosses the boundary: an offer claiming more than `nomos_cap_lint`'s own
/// ceiling permits is refused by the registry, not silently accepted.
///
/// `FactVariant::RuntimeObserved` is the exact over-ceiling claim `nomos_cap_lint`'s own
/// `Test_The_Ceiling_Should_Refuse_A_Claim_Of_Runtime_Observation` already proves the
/// contract itself refuses; this proves the registry enforces that refusal for a real offer
/// rather than the contract merely being able to detect it in isolation.
#[test]
fn Test_An_Offer_Claiming_More_Than_The_Ceiling_Should_Be_Refused()
{
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_lint::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    let over_ceiling = ProviderOffer {
        guarantee: Guarantee::New(
            FactVariant::RuntimeObserved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Project,
        ),
        ..Provider_Offer()
    };

    let error = registry.Offer(over_ceiling).expect_err("a claim above the ceiling must be refused, not accepted");
    assert_eq!(
        error.kind,
        nomos_capability::RegistryErrorKind::Offer {
            provider: Provider_Offer().provider,
            refusal: nomos_capability::OfferRefusal::ExceedsCeiling,
        },
        "the refusal must name this offer's own provider and say why: {error:?}"
    );
}

/// The lifecycle constraint the ceiling imposes on every offer: this crate's own declared
/// guarantee must satisfy it, or no registry anywhere could ever accept this crate's offer —
/// moved unchanged from `src/guarantee.rs`'s own inline test, which proved the identical
/// property through `nomos_cap_lint::Ceiling` directly.
#[test]
fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
{
    use nomos_cap_lint::Ceiling;

    assert!(Ceiling().Satisfies(&Declared_Guarantee()));
}
