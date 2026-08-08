//! Band 0 — the only authoritative statement of Nomos protocol semantics.
//!
//! Every vocabulary a peer must share in order to speak to Nomos lives here, and
//! nothing else does. No HTTP route, URL, verb, protobuf service, IPC frame, MCP tool
//! name, CLI spelling, generated SDK method or platform implementation type is
//! authoritative for these meanings; they are all projections of what this crate says.
//!
//! # Why this crate names almost nothing
//!
//! These types are reimplemented by systems that will never compile this crate — a
//! knowledge service in OCaml, a client in TypeScript, a platform in another Rust
//! workspace entirely. A dependency added here makes the protocol Nomos-shaped and
//! forces those peers to vendor a Rust crate in order to agree with us. `serde` is the
//! single exception, because the artifact a peer really reads is the JSON Schema
//! generated from these declarations. `tests/contract/contracts_names_nothing.rs`
//! holds the line.
//!
//! # The honesty vocabularies
//!
//! Five enums in this crate look similar and are deliberately not merged, because each
//! answers a different question and collapsing any pair reintroduces the failure this
//! whole design exists to prevent — an absence of knowledge reading as a statement that
//! all is well:
//!
//! | Type | Answers |
//! |---|---|
//! | [`Applicability`] | Did this rule get evaluated against this subject, and if not, why? |
//! | [`EvidenceClass`] | How was this claim come by? |
//! | [`PeerAvailability`] | Did the peer answer, or is this silence? |
//! | [`GateCategory`] | Can the thing that claims to enforce this rule actually fail a build? |
//! | [`Assurance`] | Does the producer of this fact claim soundness or completeness? |
//!
//! None of them has a `Default`, and none of them has an `is_ok`.

#![forbid(unsafe_code)]

mod applicability;
mod authority;
mod determinism;
mod enforcement;
mod evidence;
mod guarantee;
mod identity;
mod package;
mod peer;

pub use applicability::{Applicability, DisplayLabel};
pub use authority::{AuthorityClass, MutationClass};
pub use determinism::{
    Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence,
};
pub use enforcement::{EnforcementBreach, EnforcementReach, EnforcerRef, GateCategory};
pub use evidence::EvidenceClass;
pub use guarantee::{Assurance, FactVariant, Guarantee, IncrementalGranularity};
pub use identity::{
    BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128, GenerationId,
    OperationName, PackageId, ProviderId, RuleId, RunId, SchemaId, SnapshotEntityId, SnapshotId,
    SubjectId,
};
pub use package::PackageKind;
pub use peer::{PeerAvailability, SynchronizationState};
