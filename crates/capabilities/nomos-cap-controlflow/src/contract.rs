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
