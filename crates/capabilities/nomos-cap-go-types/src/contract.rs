//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The capability this crate answers.
pub const CAPABILITY: &str = "nomos.cap.go.discarded_values";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.go.discarded_values.v1";

/// The contract version -- not the crate version; callers read against this.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability: every discarded value in a file, each with
/// the type the Go type checker resolves for it, per file.
///
/// `SemanticallyResolved` because the answer is a resolved type, which no reading of the text
/// reaches: whether `_ = f()` discards an error depends on what `f` returns, and `f` may be declared
/// in another package.
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
        summary: "Every value one Go file assigns to the blank identifier -- in an assignment, a short \
                  variable declaration or a var declaration -- with its position, the type the Go type \
                  checker resolved for it, and whether that type is error."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
