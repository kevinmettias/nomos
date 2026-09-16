//! Zone: Protocol — the only authoritative statement of Nomos protocol semantics.
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
//! generated from these declarations.
//! `tests/contract/tests/boundaries/graph.rs` holds the line.
//!
//! # What machine can run this
//!
//! `core` plus an allocator, and nothing else. The crate declares `#![no_std]` unless its
//! default `std` feature is on, and both freestanding triples the gate compiles --
//! `x86_64-unknown-none` and `thumbv7em-none-eabihf` -- build it.
//!
//! This follows from what the crate already is rather than adding a constraint to it. A
//! peer that must agree with this vocabulary may be a service in another language, a
//! client in TypeScript, or a platform with no operating system under it; a protocol
//! vocabulary that cannot be stated without a host is one such a peer cannot hold. The
//! only capability it genuinely needs is allocation, because its names are owned strings
//! and its collections owned vectors.
//!
//! **The floor is declared in `Cargo.toml`, not inherited.** This is the one crate here
//! that does not take `serde` through `workspace = true`, and the reason is a trap worth
//! naming: inheriting a workspace dependency *unions* its feature list with the member's,
//! so `default-features = false` does not remove a feature the workspace entry names --
//! and that entry names `std`. Inherited, this crate compiles clean under
//! `cargo check --no-default-features` on a host and fails with nearly two thousand errors
//! on a target that has no `std`, because `serde` was built with it. Measured directly,
//! both ways. A host check is not a portability check.
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
//! of a boundary need one stable representation of it, and `boundaries/graph.rs`
//! asserts this crate depends on serde and nothing else. Five crates would make every
//! consumer name five, and would give five places for the answer to "is this vocabulary
//! authoritative" to drift apart.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod authority;
mod contract_version;
mod determinism;
mod guarantee;
mod digest128;
mod knowledge_source_role;
mod package_kind;
mod peer;
mod reporting;
mod workflow_step;

pub use authority::{AuthorityClass, MutationClass, SemanticChangeAuthorityResolution, SemanticChangeClass, UntrustedPromptContent, UntrustedPromptOrigin};
pub use contract_version::ContractVersion;
pub use determinism::{Declaration_Is_Coherent, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
pub use guarantee::{Assurance, FactVariant, Guarantee, IncrementalGranularity};
pub use digest128::{BuildVariantId, CapabilityId, ConfigurationId, Digest128, GenerationId, KnowledgeReferenceId, OperationName, PackageId, ProviderId, RuleId, RunId, SchemaId, SnapshotEntityId, SnapshotId, SubjectId};
pub use knowledge_source_role::{KnowledgeContextItem, KnowledgeSourceRole};
pub use package_kind::PackageKind;
pub use peer::{PeerAvailability, SynchronizationState};
pub use reporting::{Applicability, DisplayLabel, EnforcementBreach, EnforcementReach, EnforcerRef, EvidenceClass, Finding, GateCategory};
pub use workflow_step::{Cacheability, CancellationBehavior, Compensation, RetryPolicy, Timeout, WorkflowStep};

#[cfg(test)]
mod tests;
