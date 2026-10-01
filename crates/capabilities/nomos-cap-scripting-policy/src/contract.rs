//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a repository's own resolved scripting policy — the same
/// "name the answer, not the source" reason `nomos.cap.naming.policy` and `nomos.cap.
/// limits.policy` are each named for their own resolved answer rather than for
/// `standards.json`.
pub const CAPABILITY: &str = "nomos.cap.scripting.policy";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.scripting.policy.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — the fact this capability answers is the repository's own
/// declaration, read from `standards.json`'s text with no name resolution and no
/// inference over it, matching `nomos_cap_naming_policy::Ceiling` and `nomos_cap_limits_
/// policy::Ceiling` exactly for the identical reason.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that read the whole of the
/// `scripting` block has read everything there is to read, and reports nothing it did not
/// find there.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one declared tooling
/// language, one declared forbidden-extensions list.
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
        summary: "A repository's own declared scripting policy -- the one language its \
                  tooling is written in, and the script extensions it has decided against \
                  -- read from its own configuration rather than compiled into whichever \
                  rule judges against it."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
