//! Shared shape for a capability contract's own tests.
//!
//! Every capability crate at band 23 declares its contract in its own `contract.rs`, and
//! every one of those files asks its freshly declared contract the same four questions:
//! does the ceiling admit an offer weaker than it, does it refuse one stronger, does an
//! unoffered contract read as coverage debt rather than a missing capability, and does one
//! capability admit only one declared contract. Five crates each hand-writing "declare a
//! registry, then check" around those four questions is exactly what
//! `check-interfile-duplication` reported as structural duplication spanning all five —
//! the mechanics never varied, only the contract, the capability id, the version and the
//! guarantees under test did. Those four vary by call site; the mechanics live here once,
//! beside the `Registry`, `ProviderOffer`, `Requirement` and `Resolution` types they are
//! already written in terms of.
//!
//! # Why this is a plain public module, not `#[cfg(test)]`
//!
//! `#[cfg(test)]` does not cross a crate boundary: it says "compiled only when *this* crate
//! is under test," and every one of this module's callers is a different crate compiling
//! *its own* tests, which link this crate as an ordinary dependency, not as a
//! `cfg(test)`-selected one. Reaching that same restriction honestly would need a Cargo
//! feature threaded through every one of the five callers' `[dev-dependencies]` — real
//! machinery for what is, today, five known callers and no production code that could
//! reach these functions by accident. `#![forbid(unsafe_code)]` and the functions' own
//! signatures (`Registry`, `CapabilityContract` and friends, never anything ambient) are
//! what keeps this module harmless to leave reachable outside a test.

use crate::{CapabilityContract, ProviderOffer, Registry, Requirement, Resolution, Unmet};
use nomos_contracts::{CapabilityId, ContractVersion, Guarantee, ProviderId};

/// A fresh registry with `contract` declared, and nothing else — the setup every one of a
/// contract's own tests needs before it can offer or resolve against it.
#[must_use]
pub fn Declared_Registry(contract: CapabilityContract) -> Registry
{
    let mut registry = Registry::New();
    registry.Declare(contract).expect("declared once");

    return registry;
}

/// Asserts that `guarantee` is weak enough for `contract`'s own ceiling to admit — the
/// property a ceiling exists to provide in the first place: an offer at or below it is
/// accepted.
pub fn Assert_Ceiling_Admits(
    contract: CapabilityContract,
    capability: CapabilityId,
    version: ContractVersion,
    guarantee: Guarantee,
)
{
    let mut registry = Declared_Registry(contract);
    let offer = ProviderOffer {
        provider: ProviderId::New("nomos.test.weak"),
        capability,
        version,
        guarantee,
    };

    assert_eq!(registry.Offer(offer), Ok(()));
}

/// Asserts that `guarantee` is too strong for `contract`'s own ceiling — a claim above it
/// is refused rather than silently accepted. `why` is the reason accepting it would be
/// wrong for this capability, and becomes the assertion's own failure message.
pub fn Assert_Ceiling_Refuses(
    contract: CapabilityContract,
    capability: CapabilityId,
    version: ContractVersion,
    guarantee: Guarantee,
    why: &str,
)
{
    let mut registry = Declared_Registry(contract);
    let offer = ProviderOffer {
        provider: ProviderId::New("nomos.test.optimistic"),
        capability,
        version,
        guarantee,
    };

    assert!(registry.Offer(offer).is_err(), "{why}");
}

/// Asserts that a contract with no offer against it resolves to coverage debt
/// (`Unmet::NoProvider`), never to a missing capability.
pub fn Assert_Stands_With_No_Provider(
    contract: CapabilityContract,
    capability: CapabilityId,
    version: ContractVersion,
    ceiling: Guarantee,
)
{
    let registry = Declared_Registry(contract);
    let need = Requirement::New(capability, version, ceiling);
    let resolution = registry.Resolve(&need);

    assert!(
        matches!(
            resolution,
            Resolution::Unsatisfied {
                reason: Unmet::NoProvider,
                ..
            }
        ),
        "the contract is declared and unoffered, which is coverage debt rather than an \
         undeclared capability: {resolution:?}"
    );
}

/// Asserts that one capability admits exactly one declared contract — declaring the same
/// contract a second time is refused. `why` names what that refusal protects, and becomes
/// the assertion's own failure message.
pub fn Assert_One_Contract_Per_Capability(contract: CapabilityContract, why: &str)
{
    let mut registry = Declared_Registry(contract.clone());

    assert!(registry.Declare(contract).is_err(), "{why}");
}
