//! Zone: Capability Contract — the `nomos.cap.rust.copy_clones` contract, owned by neither
//! its provider nor any rule that reads it.
//!
//! # Why this moved out of its provider, and why only now
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside
//! its only provider until a second real party names it, which is why this one sat inside
//! `nomos-lang-rust-compiler` from `P40-COMPILER-BACKED-PROVIDER` until this crate existed.
//! `OD-ANALYSIS-007` version 2 decided which second party counts and when: a `nomos-rules`
//! descriptor naming this capability, because the compiler-backed provider is zoned
//! `Provider` and `Permits` forbids `Rules` from naming `Provider`. That descriptor now
//! exists, so the contract is here.
//!
//! # Why this is its own crate and not half of a shared one
//!
//! `OD-ANALYSIS-007` version 2 refuses a shared `nomos-cap-rust-semantics` by name. This
//! capability and `nomos.cap.rust.nested_locks` share their provider's *mechanics* —
//! loading a crate through `ra_ap_hir`, sysroot discovery, per-crate file filtering — and
//! nothing contract-shaped: two identities, two schemas, two payload vocabularies, two
//! ceilings that happen to coincide. A family crate would be a home for exactly the shared
//! program-semantics vocabulary `OD-ANALYSIS-004` forbids, so there are two crates in the
//! shape the five policy contracts already have and no third between them.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]), and the
//! payload shape with its canonical encoding ([`payload`]) — the same division
//! `nomos-cap-dependency-policy` and `nomos-cap-lint` each draw between their own `contract`
//! and `payload` modules.
//!
//! What is not here: the `ra_ap_hir` analysis that answers this capability, which stays in
//! `nomos-lang-rust-compiler` where its dependencies are, and the rule that judges what that
//! analysis produced, which is `nomos-rules`'.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::clone_on_copy_payload::CloneOnCopyPayload;
pub use payload::cloned_copy_type::ClonedCopyType;
pub use payload::refusal::Refusal;
pub use payload::{Encode_Payload, Parse_Payload};
