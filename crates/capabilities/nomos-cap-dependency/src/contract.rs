//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a package's own first-party dependency edges — rather
/// than for how it is obtained, the same reason `nomos.cap.syntax.items` is named for its
/// answer rather than for `syn`.
pub const CAPABILITY: &str = "nomos.cap.dependency.edges";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract for the same reason `nomos-cap-syntax` gives:
/// the shape of the bytes and the meaning of the question change for different reasons.
pub const SCHEMA: &str = "nomos.dependency.edges.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`] because a first-party dependency edge is not what
/// a `dependencies = {...}` block says on its face — a manifest can name a package by a
/// string that a text scan cannot tell apart from an unrelated crate of the same name on
/// a registry, the same ambiguity `OD-RULES-003` names for why this capability's answer
/// must come from Cargo's own resolution rather than from pattern-matching TOML. A
/// producer that only scanned manifest text belongs at `Syntactic`, not here.
///
/// [`IncrementalGranularity::Project`] because a package's dependency edges are declared
/// once for the whole package, in its manifest, and there is no file-level slice of that
/// declaration to refresh independently — the same reasoning `nomos.cap.module.index`
/// already gives for its own ceiling.
///
/// Completeness [`Assurance::Sound`], not pinned to what any current provider achieves:
/// this ceiling leaves room for a provider that also resolves generated or workspace-
/// inherited dependency declarations, the same way `nomos-cap-syntax`'s ceiling leaves
/// room for a provider that expands macros.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
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
        summary: "One package's own first-party dependency edges — every other workspace \
                  member it names as a dependency, each attributed the kind (normal, dev, \
                  build) and whether it is optional, as Cargo itself resolves the \
                  declaration rather than as a manifest's text alone can say."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
