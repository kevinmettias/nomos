//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
#[doc = include_str!("../docs/api/contract.md")]
pub const CAPABILITY: &str = "nomos.cap.controlflow.reachability";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.controlflow.reachability.v1";

/// The contract version — not the crate version; callers read against this.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
#[doc = include_str!("../docs/api/contract.md")]
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

/// Wraps [`CAPABILITY`] as the [`CapabilityId`] this crate's contract is declared and
/// providers are offered under.
#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

/// Wraps [`SCHEMA`] as the [`SchemaId`] every answer to this capability is stamped with.
#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// The contract, to be declared once by whichever composition root builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "Whether a control-flow path forward from a fact-read failure — a match \
                  arm binding an `Err` from a capability read — reaches a `Finding` \
                  construction before the enclosing function returns, the property \
                  `Applicability`'s own module doc names as this product's first \
                  principle: unknown is not pass."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::contract_testing;

    /// The contract admits an answer weaker than its ceiling, which is what a ceiling is
    /// for — a `Syntactic`, pattern-matching-only provider is exactly the first offer this
    /// capability expects.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        contract_testing::Assert_Ceiling_Admits(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unsound,
                IncrementalGranularity::File,
            ),
        );
    }

    /// And refuses one above it — a claim of `RuntimeObserved` would be a provider
    /// asserting it watched every path actually execute, which no static reader does.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Above_Semantically_Resolved()
    {
        contract_testing::Assert_Ceiling_Refuses(
            Capability_Contract(),
            Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            "a provider claiming runtime observation for a capability about static \
             reachability would satisfy every rule that needs it",
        );
    }

    /// The contract stands without a provider.
    #[test]
    fn Test_The_Contract_Should_Stand_With_No_Provider_At_All()
    {
        contract_testing::Assert_Stands_With_No_Provider(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Ceiling(),
        );
    }

    /// One declaration per capability.
    #[test]
    fn Test_One_Capability_Should_Admit_One_Contract()
    {
        contract_testing::Assert_One_Contract_Per_Capability(Capability_Contract(), "one name, one meaning");
    }
}
