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
//! # Why the four assertions are a submodule
//!
//! This module is two jobs: the setup a contract's own tests need ([`Declared_Registry`]),
//! and the four questions they then ask of it. The second job's names all begin `Assert` —
//! a module boundary drawn in spelling but not in the tree — so those four live in
//! `assertions`, and the re-export below keeps every caller's path exactly what it was.
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

mod assertions;

use crate::{CapabilityContract, Registry};

pub use assertions::{
    Assert_Ceiling_Admits, Assert_Ceiling_Refuses, Assert_One_Contract_Per_Capability,
    Assert_Stands_With_No_Provider,
};

/// A fresh registry with `contract` declared, and nothing else — the setup every one of a
/// contract's own tests needs before it can offer or resolve against it.
///
/// A freshly constructed [`Registry`] has nothing declared yet, so this first declaration
/// can never collide -- the failure `Declare` reports is a broken precondition here, not a
/// recoverable one, which is why it is asserted rather than propagated.
#[must_use]
pub fn Declared_Registry(contract: CapabilityContract) -> Registry
{
    let mut registry = Registry::New();
    let declared = registry.Declare(contract);
    assert!(declared.is_ok(), "a fresh registry's first declaration cannot conflict: {declared:?}");

    return registry;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{
        Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    };

    #[test]
    fn Test_Declared_Registry_Should_Declare_The_Contract_Exactly_Once()
    {
        let registry = Declared_Registry(Contract());

        assert_eq!(registry.Declared().count(), 1);
    }

    fn Contract() -> CapabilityContract
    {
        return CapabilityContract {
            id: Capability(),
            version: Version(),
            summary: "a contract for contract_testing.rs's own tests".to_owned(),
            ceiling: Ceiling(),
        };
    }

    fn Capability() -> CapabilityId
    {
        return CapabilityId::New("nomos.cap.test.contract_testing");
    }

    fn Version() -> ContractVersion
    {
        return ContractVersion::New(1, 0);
    }

    fn Ceiling() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        );
    }
}
