//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a repository's own resolved goal declaration — the same
/// "name the answer, not the source" reason its four `OD-RULES-011` siblings are each
/// named for their resolved answer rather than for `standards.json`.
pub const CAPABILITY: &str = "nomos.cap.goals.policy";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.goals.policy.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — the fact this capability answers is the repository's own
/// declaration, read from `standards.json`'s text with no name resolution and no inference
/// over it, matching every `OD-RULES-011` sibling's ceiling for the identical reason. A
/// goal in particular cannot be anything stronger: `check-goal-traceability`'s own header
/// says the intent it judges "is not a fact about which code calls which", so there is
/// nothing here a provider could resolve even if it wanted to.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that read the whole of the
/// goal and subsystem declaration has read everything there is to read, and reports
/// nothing it did not find there.
///
/// [`IncrementalGranularity::WholeWorkspace`]: a system has one set of purposes. Asking
/// which goals hold for one member would be asking the question this capability exists to
/// answer, since which part serves which purpose is the answer rather than the index.
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
        summary: "The purposes a repository declares it exists to serve, the parts it \
                  declares to serve them, and the ceiling on how thinly one purpose may be \
                  spread -- read from the repository's own configuration rather than \
                  compiled into whichever rule holds the two halves to each other."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
