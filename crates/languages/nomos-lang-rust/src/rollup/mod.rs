//! A fact computed from other facts, and the dependency edges that come with it.
//!
//! # What was missing
//!
//! `nomos-analysis` ships a complete transitive invalidation mechanism — a reverse index of
//! dependents, a [`nomos_analysis::Reader`] that records every read, and an
//! [`nomos_analysis::InvalidationReport`] that separates what a change hit directly from
//! what it reached through an edge. Nothing that ships created an edge. Every fact this
//! workspace materializes outside a test is a leaf, so
//! [`nomos_analysis::InvalidationReport::dependent`] was a field no run could make
//! non-empty.
//!
//! The reason was structural rather than an omission. Every fact is filed under a
//! capability, and the one this crate answered — `nomos-cap-syntax`'s — is a per-file leaf
//! by construction: its semantic input is one file's text and nothing else. A capability
//! whose answer is a function of one file can never depend on another answer. So the
//! dependent half of invalidation needed a *second* capability before it could have a
//! producer, and this module is it. `docs/records/OD-ANALYSIS-002` records the reasoning.
//!
//! # The question this capability asks
//!
//! *Which items does a module declare, and which of its files declared each one.*
//!
//! That is deliberately not the syntax capability's question asked over more subjects. A
//! union of syntax payloads would lose the only thing that makes a module-level answer
//! worth having: [`nomos_cap_syntax::PayloadItem`] carries a name qualified by syntactic
//! nesting *within one file* and has no field for which file that was, so two files each
//! declaring `Read` produce two indistinguishable records. Every entry here names its
//! member, which is the field the rollup exists to add.
//!
//! # Why the contract lives beside its provider
//!
//! `OD-CAPABILITY-002` sets the criterion and it is contention, not principle: a capability
//! with a single provider is not wrongly filed for living beside that provider, and moving
//! it out would buy a crate and no property. `nomos-cap-syntax` exists because two crates
//! offer against it and neither may name the other. Nothing offers against this one but the
//! function below. The day something else does, this belongs under `crates/capabilities`
//! and `Test_A_Capability_Id_Should_Be_Written_In_One_Crate` will say so the moment the id
//! is spelled twice.
//!
//! # Why the granularity is Project
//!
//! A rollup over a module cannot refresh half a module. Declaring
//! [`IncrementalGranularity::File`] would be claiming a precision this has no way to
//! deliver, and the invalidation engine would refresh one member's contribution and treat
//! the rest as current. [`IncrementalGranularity::Project`] makes the engine broaden a
//! file-granular cause and record on
//! [`nomos_analysis::InvalidationReport::broadened`] that it did — the cost of the rollup,
//! stated rather than absorbed.
//!
//! # Why the edges are not a list written here
//!
//! [`Materialize_Index`] never assembles a dependency array. It reads through a
//! [`nomos_analysis::Reader`] and hands the store what the reader observed. A hand-written
//! edge list is a *claim* about what was read; this is a record of it, and the two diverge
//! the first time a read is added and the list is not. That includes the reads that found
//! nothing: a member with no fact is still an edge, because the day the parser does have
//! something for it, this rollup is stale and only the edge knows.
//!
//! # Determinism
//!
//! Same triple as [`crate::SyntaxFactProduction`], which is this crate's declaration and
//! covers it: the member order is canonical rather than the caller's, the payload is
//! hand-encoded in one place, and nothing in the path touches a clock, a path separator or
//! an unordered collection. A second `impl Strategy` naming the same row is deliberately
//! not added — `tests/contract`'s determinism guard requires every declaration to be held
//! to it by the integration harness, so a declaration added without one there is a promise
//! nothing can falsify, which is the defect that guard exists to catch.

mod contract;
mod materialize;
mod encode;
mod parse;
#[cfg(test)]
mod tests;

pub use contract::{CAPABILITY, CONTRACT_VERSION, Capability, Capability_Contract, Ceiling, Declared_Guarantee, PROVIDER, Payload_Schema, Provider_Offer, SCHEMA};
pub use materialize::{Index_Key, Materialize_Index};
pub use encode::Encode_Index;
pub use parse::Parse_Index;

mod against;
mod index_entry;
mod member_reading;
mod module;
mod module_index;
mod module_member;
mod outcome;
mod rolled;

pub use against::Against;
pub use index_entry::IndexEntry;
pub use member_reading::MemberReading;
pub use module::Module;
pub use module_index::ModuleIndex;
pub use module_member::ModuleMember;
pub use outcome::Outcome;
pub use rolled::Rolled;

use crate::provider::FactContext;
use nomos_analysis::{
    Context, Dependency, FactError, FactKey, FactPayload, FactReader, GuaranteeDigest, InputDigest,
    MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Requirement};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, ContractVersion, Digest128, EvidenceClass, FactVariant,
    Guarantee, IncrementalGranularity, ProviderId, SchemaId, SubjectId,
};

/// The `outcome` field for a member read from the provider the caller asked for.
pub const READ: &str = "read";

/// The `outcome` field for a member answered by a weaker provider than the caller asked
/// for.
pub const APPROXIMATE: &str = "approximate";

/// The `outcome` field for a member with no readable answer.
pub const UNREACHABLE: &str = "unreachable";
