//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a repository's declared standards corpus, read where it is
/// declared — the same "name the answer, not the source" reason the `OD-RULES-011` policy
/// siblings are each named for their own resolved answer rather than for `standards.json`.
pub const CAPABILITY: &str = "nomos.cap.standards.corpus";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.standards.corpus.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] — the fact this capability answers is what each document's own
/// front matter says, read as text with no name resolution and no inference over the prose
/// that follows it. A reader here resolves nothing: it must not consult a rule's own table of
/// contents, follow its `canonical:` pointer into another repository, or decide that a
/// document *looks* like a rule. Each of those would be a claim the corpus's own schema
/// deliberately refuses to make, and the ceiling says so at the contract rather than leaving
/// it to whichever reader happens to be written first.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a reader that reached every document under
/// every declared root has read everything there is to read, and reports nothing it did not
/// find there. The population is the whole of the answer.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one declared corpus set, not a
/// property of any one workspace member. It is deliberately not `PerMember` even where a
/// declared root happens to sit inside one member: a corpus is the repository's, and a
/// document moved between members is the same rule in the same corpus.
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
        summary: "A repository's declared standards corpus -- every document under the roots \
                  its own declaration names, each with the kind it declares for itself, and for \
                  the documents declaring kind: rule the severity, gate and mechanical owners \
                  they declare -- read from a file of the corpus's own rather than from \
                  standards.json, which another tool decodes with unknown fields disallowed. A \
                  document is a rule because it declares kind: rule and never because a pattern \
                  recognised its shape; a document whose declaration will not parse is an issue \
                  rather than a silent omission from the population. A repository declaring no \
                  corpus judges exactly as it did before this capability existed."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
