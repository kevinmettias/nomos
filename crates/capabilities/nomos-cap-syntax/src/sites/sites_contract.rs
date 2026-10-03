//! The agreement for the family: its identity, version, ceiling and schema.

use nomos_capability::CapabilityContract;
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The capability every provider offering a kind answers.
///
/// Named for what a caller gets -- the sites of constructs a file's syntax holds -- and not for
/// any one projection, because a projection is a kind this family carries rather than a
/// capability of its own (`OD-CAPABILITY-019`).
pub const SITES_CAPABILITY: &str = "nomos.cap.syntax.sites";

/// The payload schema every answer to [`SITES_CAPABILITY`] is stamped with.
///
/// A new kind does not move it: the grammar names a kind and its fields by declaration, so a
/// payload carrying a kind this build has never declared is refused by kind, not by schema.
pub const SITES_SCHEMA: &str = "nomos.syntax.sites.v1";

/// The contract version. The question -- which declared constructs a file holds, and where --
/// is the same for every kind, so adding one does not move this either.
pub const SITES_CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this family: what a file says on its face.
///
/// The same ceiling and the same reasons as [`crate::Ceiling`]. A site is a fact about the
/// parse tree, so a provider that resolved names would be answering a different question; and
/// completeness and granularity are left at what a better provider might reach rather than
/// pinned to what today's do.
#[must_use]
pub const fn Sites_Ceiling() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::Region);
}

#[must_use]
pub fn Sites_Capability() -> CapabilityId
{
    return CapabilityId::New(SITES_CAPABILITY);
}

#[must_use]
pub fn Sites_Payload_Schema() -> SchemaId
{
    return SchemaId::New(SITES_SCHEMA);
}

/// The contract, declared once by whichever composition root builds a registry -- by a root and
/// not by a provider, for the reason [`crate::Capability_Contract`] gives.
#[must_use]
pub fn Sites_Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Sites_Capability(),
        version: SITES_CONTRACT_VERSION,
        summary: "Every construct a source file's syntax holds of a kind this contract declares, one flat record per \
                  construct with its line and its kind's fields, and for each declared kind whether the file's \
                  provider offers it or declines it and why."
            .to_owned(),
        ceiling: Sites_Ceiling(),
    };
}
