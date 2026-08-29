//! Band 23 — the `nomos.cap.dependency.edges` contract, owned by neither its provider nor
//! any rule that reads it.
//!
//! # Why this earned a crate immediately, unlike `nomos.cap.module.index`
//!
//! `nomos.cap.module.index` lives inside `nomos-lang-rust`, beside its only provider,
//! because nothing else names it yet — `OD-CAPABILITY-002`'s criterion is contention, and
//! there has been none. This capability has a real second party from the day it was
//! written: `nomos-rules`' own `Check_Dependency_Direction` reads it, and `nomos-rules`'
//! `Cargo.toml` already states the rule this crate exists to keep — "a rule states a
//! floor and the registry says who can serve it; naming a provider here would be this
//! crate deciding the thing the registry exists to decide." A rule that depended on
//! `nomos-lang-rust-cargo` directly to reach `Capability()`/`Payload_Schema()` would be
//! exactly that: naming its provider. So the contract lives here, below both the provider
//! (`nomos-lang-rust-cargo`, band 25) and the rule that reads it (`nomos-rules`, band 30),
//! named by neither.
//!
//! # What is here
//!
//! What the parties agree on — the capability's identity, the contract version, the
//! ceiling, the summary, and the schema every answer is stamped with — plus the payload
//! shape and its canonical reader, the same division `nomos-cap-syntax` draws between its
//! `contract` and its `payload` modules.

#![forbid(unsafe_code)]

mod contract;
mod dependency_kind;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use dependency_kind::{DependencyKind, Encode_Payload, Parse_Payload};
pub use dependency_kind::dependency_edge::DependencyEdge;
pub use dependency_kind::dependency_payload::DependencyPayload;
pub use dependency_kind::payload_refusal::PayloadRefusal;
