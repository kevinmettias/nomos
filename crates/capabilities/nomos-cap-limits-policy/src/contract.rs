//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a repository's own resolved numeric thresholds — the
/// same "name the answer, not the source" reason `nomos.cap.naming.policy` is named for
/// its resolved convention rather than for `standards.json`: a second provider reading a
/// different configuration surface for the same answer must be able to name this
/// capability honestly.
pub const CAPABILITY: &str = "nomos.cap.limits.policy";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.limits.policy.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — the fact this capability answers is the repository's own
/// declaration, read from `standards.json`'s text with no name resolution and no
/// inference over it, the same reasoning `nomos.cap.naming.policy::Ceiling` gives for the
/// sibling capability `OD-RULES-011` names this crate's own family after.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every row a provider reports is a
/// threshold the repository's own configuration actually declared, never one this
/// capability infers. Completeness [`Assurance::Sound`] as well — a provider that reads
/// the whole of `standards.json`'s declared threshold blocks has read everything there is
/// to read.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one declared set of
/// thresholds — a numeric ceiling is not a property of any one workspace member, so there
/// is no per-member split this ceiling could honestly claim instead.
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
        summary: "A repository's own declared numeric thresholds -- file-size triggers, \
                  parameter-count caps and their like -- read from its own configuration \
                  rather than compiled into whichever rule judges against them."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
