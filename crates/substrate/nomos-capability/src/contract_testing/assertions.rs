//! The four questions a capability contract's own tests ask it, and nothing else.
//!
//! Split out of `contract_testing.rs` by responsibility: [`super::Declared_Registry`] is the
//! setup every one of these needs, and these are what the setup is for. They share the
//! `Assert` prefix because they are assertions, so the prefix is a home rather than a hidden
//! boundary — this module is named for it, and the parent re-exports the four so every
//! caller's path is unchanged.

use crate::{CapabilityContract, ProviderOffer, Requirement, Resolution, Unmet};
use nomos_contracts::{CapabilityId, ContractVersion, Guarantee, ProviderId};

use super::Declared_Registry;

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

/// Asserts that `guarantee` is too strong for `contract`'s own ceiling — a claim above it is
/// refused rather than silently accepted. `why` is the reason accepting it would be wrong
/// for this capability, and becomes the assertion's own failure message.
///
/// Unlike its siblings, this does not take `capability` and `version` as separate
/// parameters: `contract.id` and `contract.version` already carry them, every caller was
/// passing the same values back in twice, and doing so had pushed this function one
/// parameter past the standard's limit.
pub fn Assert_Ceiling_Refuses(contract: CapabilityContract, guarantee: Guarantee, why: &str)
{
    let capability = contract.id.clone();
    let version = contract.version;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

    #[test]
    fn Test_Assert_Ceiling_Admits_Should_Pass_For_A_Guarantee_At_Or_Below_The_Ceiling()
    {
        Assert_Ceiling_Admits(Contract(), Capability(), Version(), Weaker_Guarantee());
    }

    /// Weaker than [`Ceiling`] on every axis a ceiling can be strong on.
    fn Weaker_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unsound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    #[test]
    fn Test_Assert_Ceiling_Refuses_Should_Pass_For_A_Guarantee_Above_The_Ceiling()
    {
        Assert_Ceiling_Refuses(Contract(), Stronger_Guarantee(), "stronger than the declared ceiling");
    }

    /// Claims name resolution, which [`Ceiling`]'s merely-syntactic promise cannot back.
    fn Stronger_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        );
    }

    #[test]
    fn Test_Assert_Stands_With_No_Provider_Should_Pass_For_A_Declared_And_Unoffered_Contract()
    {
        Assert_Stands_With_No_Provider(Contract(), Capability(), Version(), Ceiling());
    }

    #[test]
    fn Test_Assert_One_Contract_Per_Capability_Should_Pass_When_A_Second_Declaration_Is_Refused()
    {
        Assert_One_Contract_Per_Capability(Contract(), "one name, one meaning");
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

    fn Contract() -> CapabilityContract
    {
        return CapabilityContract {
            id: Capability(),
            version: Version(),
            summary: "a contract for contract_testing.rs's own tests".to_owned(),
            ceiling: Ceiling(),
        };
    }
}
