//! The agreement itself.
//!
//! Below every party to it: the provider that answers `nomos.cap.rust.copy_clones`
//! (`nomos-lang-rust-compiler`) and the rule that reads it (`nomos-rules`) both import this
//! module rather than either one naming the other.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate carries the contract for.
///
/// Named for what a caller gets -- which `.clone()` calls duplicate a value that was
/// already cheap to copy -- rather than for the mechanism (`ra_ap_hir`) that answers it,
/// the same reason `nomos.cap.dependency.policy` is named for its answer rather than for
/// `cargo deny`: a second provider answering the same question through a different
/// compiler frontend must be able to name this capability honestly.
pub const CAPABILITY: &str = "nomos.cap.rust.copy_clones";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.rust.copy_clones.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`]: a `.clone()` call's receiver only has a `Copy`
/// answer once its type is resolved and that type's own trait implementations are
/// looked up -- a syntax tree alone (`FactVariant::Syntactic`) cannot see a `#[derive]`
/// on a type declared in a different module, let alone one declared in another crate.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every finding this capability's real
/// provider reports names a receiver a real compiler frontend actually resolved to a
/// type that actually implements `Copy`, never one inferred from a naming convention or
/// a heuristic.
///
/// Completeness [`Assurance::Unknown`]: this ceiling leaves room for a stronger future
/// provider than today's one real answer honestly claims -- see
/// `nomos_lang_rust_compiler::Declared_Guarantee` for why the one provider that exists
/// today does not claim it either.
///
/// [`IncrementalGranularity::Project`]: resolving one `.clone()` call's receiver type can
/// depend on any item reachable from it through the crate's own module tree, so the unit
/// that must recompute together is the whole crate being analyzed, not the one file the
/// call happens to be written in.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
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
        summary: "Every `.clone()` call in an analyzed crate whose receiver a real \
                  compiler frontend resolved to a type that already implements Copy, \
                  found by asking rust-analyzer's own semantic-analysis engine to \
                  resolve the call and check the resolved type's trait implementations \
                  -- never inferred from the call's own syntax."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
