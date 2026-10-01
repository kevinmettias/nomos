//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The capability this crate answers.
#[doc = include_str!("../docs/api/contract.md")]
pub const CAPABILITY: &str = "nomos.cap.metric.complexity";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.metric.complexity.v1";

/// The contract version — not the crate version; callers read against this.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability: a count over a resolved program,
/// macros expanded, which would see every branch a function really has.
#[doc = include_str!("../docs/api/contract.md")]
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);
}

/// Wraps [`CAPABILITY`] as the [`CapabilityId`] this crate's contract is declared and providers
/// are offered under.
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

/// The contract, to be declared once by whichever composition builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "Each function's cyclomatic complexity -- one more than the number of decision \
                  points in its body -- with the MET-006 descriptor that states the measure's unit, \
                  subject kind, aggregation, weighting, normalization, missing-data behaviour, \
                  directionality, baseline, uncertainty and snapshot comparability."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
