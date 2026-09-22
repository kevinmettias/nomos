//! Zone: Capability Contract — the `nomos.cap.rust.nested_locks` contract, owned by neither
//! its provider nor any rule that reads it.
//!
//! # Why this moved out of its provider, and why only now
//!
//! The same sequence `nomos-cap-rust-copy-clones` states for the crate's other capability:
//! `OD-CAPABILITY-002`'s criterion is contention, so this contract sat inside
//! `nomos-lang-rust-compiler` from `P42-SEMANTIC-FACT-FAMILY` until a second real party
//! named it, and `OD-ANALYSIS-007` version 2 decided which party that is -- a `nomos-rules`
//! descriptor, since the provider is zoned `Provider` and `Permits` forbids `Rules` from
//! naming `Provider`.
//!
//! # Why this is its own crate and not half of a shared one
//!
//! `OD-ANALYSIS-007` version 2 measured the alternative and refused it by name: what this
//! capability shares with `nomos.cap.rust.copy_clones` is its provider's mechanics
//! (`Load_Crate`, sysroot discovery, per-crate file filtering), which stay in the provider,
//! and nothing contract-shaped. The two ceilings coincide, which is a coincidence about two
//! questions a compiler frontend answers rather than a shared vocabulary; a family crate
//! would be a home for exactly the shared program-semantics vocabulary `OD-ANALYSIS-004`
//! forbids.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]), and the
//! payload shape with its canonical encoding ([`payload`]) -- the same division every
//! `nomos-cap-*` sibling draws.
//!
//! What is not here: the `ra_ap_hir` analysis that answers this capability, which stays in
//! `nomos-lang-rust-compiler`, and the rule that judges what it produced, which is
//! `nomos-rules`'.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use payload::nested_lock_finding::NestedLockFinding;
pub use payload::nested_lock_payload::NestedLockPayload;
pub use payload::refusal::Refusal;
pub use payload::{Encode_Payload, Parse_Payload};
