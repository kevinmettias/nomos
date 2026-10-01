//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The capability this crate answers.
#[doc = include_str!("../docs/api/contract.md")]
pub const CAPABILITY: &str = "nomos.cap.csharp.conditional_compilation";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.csharp.conditional_compilation.v1";

/// The contract version — not the crate version; callers read against this.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability: every branch of every conditional in
/// the file, each judged against the definition set the named build hands its compiler.
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
        summary: "Which branches of a C# file's #if/#elif/#else chains a named build compiles -- judged \
                  against the preprocessor symbols that build defines, which no reading of the file \
                  alone can know."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
