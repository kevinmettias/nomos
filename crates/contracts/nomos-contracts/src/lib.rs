//! Band 0 — the only authoritative statement of Nomos protocol semantics.
//!
//! Every vocabulary a peer must share in order to speak to Nomos lives here, and
//! nothing else does. No HTTP route, URL, verb, protobuf service, IPC frame, MCP tool
//! name, CLI spelling, generated SDK method or platform implementation type is
//! authoritative for these meanings; they are all projections of what this crate says.
//!
//! # What earns a place here
//!
//! A type is admitted when it **crosses a subsystem, process or plugin boundary and the
//! parties on both sides need one stable shared representation of it**. Everything else
//! stays in the crate that owns it, and crosses a boundary as a projection.
//!
//! The second half is the half that refuses, and it is the half a wider phrasing loses. A
//! finding shape, a rule package, a correction plan and an agent capability are all
//! statements of Nomos semantics, so "a statement of Nomos semantics" admits everything and
//! decides nothing. The question that decides is narrower: *would a peer that never compiles
//! this crate be unable to agree with us without this type?*
//!
//! `nomos-cap-syntax`'s `SyntaxPayload` is the worked counter-example — a real shared
//! representation, read by more than one provider, living at band 23 rather than here,
//! because the parties that must agree about it are the providers of one capability rather
//! than every peer that speaks to Nomos.
//!
//! `OD-CONTRACTS-001` decides this and is the only place it is stated. `README.md` and
//! `Cargo.toml` name it; neither restates it.
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
//!
//! [`Finding`] is where several of those answers arrive together about one subject. It
//! is a type rather than a formatted line because every one of them is destroyed by
//! printing: prose cannot be asked whether the rule reached its subject, how the claim
//! was come by, or whether anything would actually have failed a build over it.
//!
//! # Why `check-crate-split` reports this crate, and why it stays one
//!
//! That check clusters a crate's files by which of them reference which, and reports a
//! crate whose files fall into groups that never speak to each other. This crate is five
//! such groups: enforcement together with the finding vocabulary it is reported through,
//! then determinism, guarantee, authority and peer standing alone. They never mention one
//! another and they never will, because a shared vocabulary's terms are independent by
//! construction. The check's own standard names that case -- a boundary may exist to seal
//! a subsystem, and no reference graph can see intent.
//!
//! The intent here is the band. `OD-CONTRACTS-001` admits a type when peers on both sides
//! of a boundary need one stable representation of it, and `contracts_names_nothing`
//! asserts this crate depends on serde and nothing else. Five crates would make every
//! consumer name five, and would give five places for the answer to "is this vocabulary
//! authoritative" to drift apart.

#![forbid(unsafe_code)]

mod authority;
mod contract_version;
mod determinism;
mod enforcement;
mod finding;
mod guarantee;
mod identity;
mod package;
mod peer;
mod workflow_step;

pub use authority::{AuthorityClass, MutationClass};
pub use contract_version::ContractVersion;
pub use determinism::{Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
pub use enforcement::{EnforcementBreach, EnforcementReach, EnforcerRef, GateCategory};
pub use finding::{Applicability, DisplayLabel, EvidenceClass, Finding};
pub use guarantee::{Assurance, FactVariant, Guarantee, IncrementalGranularity};
pub use identity::{BuildVariantId, CapabilityId, ConfigurationId, Digest128, GenerationId, KnowledgeReferenceId, OperationName, PackageId, ProviderId, RuleId, RunId, SchemaId, SnapshotEntityId, SnapshotId, SubjectId};
pub use package::PackageKind;
pub use peer::{PeerAvailability, SynchronizationState};
pub use workflow_step::{Cacheability, CancellationBehavior, Compensation, RetryPolicy, Timeout, WorkflowStep};
