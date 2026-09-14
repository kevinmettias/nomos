//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The capability this crate answers.
///
/// Named for what a caller gets — a repository's own declared architecture — the same "name
/// the answer, not the source" reason `nomos.cap.naming.policy` is named for its resolved
/// convention rather than for `standards.json`. A second provider reading a different
/// declaration surface for the same answer must be able to name this capability honestly.
pub const CAPABILITY: &str = "nomos.cap.architecture.declaration";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.architecture.declaration.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — the fact this capability answers is the repository's own
/// declaration, read from its text with no name resolution and no inference over it. A
/// provider that *derived* an architecture by inspecting code would be answering a different
/// question, and `OD-RULES-024` is explicit that what is missing here is a declared
/// architecture and not an inference engine.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every component, edge and authority a
/// provider reports is one the repository's own declaration actually states, never one this
/// capability inferred. Completeness [`Assurance::Sound`] as well — a provider that read the
/// whole declaration has read everything there is to read.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one architecture. A component
/// is not a property of any one workspace member, and the order over components is a property
/// of none of them, so there is no per-member split this ceiling could honestly claim instead.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
    );
}

#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

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
        summary: "A repository's own declared architecture -- the components it divides \
                  itself into, which member belongs to which, which component may depend on \
                  which, the named pairs the order alone cannot express, and its declared \
                  write authorities -- read from its own configuration rather than compiled \
                  into whichever rule judges against it."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
