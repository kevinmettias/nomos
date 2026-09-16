//! Zone: Provider — this crate's two providers: `nomos.cap.rust.copy_clones` and
//! `nomos.cap.rust.nested_locks`.
//!
//! # Why this is this workspace's first compiler-backed provider
//!
//! Every provider this workspace had before this crate reads syntax (`nomos-lang-rust`
//! through `syn`), a manifest (`nomos-lang-rust-cargo` through `cargo metadata`), or
//! another tool's own report (`nomos-lang-rust-clippy`, `nomos-lang-rust-deny`). None of
//! them consults a compiler for resolved semantics -- a question needing a name's actual
//! binding, a type's actual shape, or a trait's actual implementation has had no
//! evidence source in this system until now. `P40-COMPILER-BACKED-PROVIDER`.
//!
//! # Why this crate answers narrow questions rather than a broad one
//!
//! `crates/languages/nomos-lang-rust-compiler/src/lib.rs`'s own task was not to build a
//! `CompilerProvider` or `LspProvider` taxonomy first -- one concrete provider, real
//! evidence, and then whether a shared abstraction is warranted at all is a question the
//! evidence answers, not a decision made ahead of it. This crate's first capability
//! picked the narrowest real question a compiler frontend answers that a syntax tree
//! cannot: whether a `.clone()` call's receiver already implements `Copy`, which needs
//! full name resolution and a trait-implementation lookup, not merely a parse. Its
//! second, `nomos.cap.rust.nested_locks` (`P42-SEMANTIC-FACT-FAMILY`,
//! concurrency-and-synchronization), picks an equally narrow one: whether a
//! `std::sync::Mutex<T>`/`RwLock<T>` guards a `T` that is itself already a lock, which
//! needs resolving `T` past whatever type alias or re-export stands between the syntax
//! and the answer -- the same "needs resolved semantics, not a parse" shape as the
//! first, and neither a full move/borrow analysis (no MIR borrow-checker is exposed
//! through `ra_ap_hir`) nor something the compiler already refuses to build without.
//!
//! # Why this workspace's semantic engine is `ra_ap_hir`, not an embedded `rustc`
//!
//! `rustc`'s own semantic APIs (`rustc_interface`, `rustc_driver`) are only available
//! through `rustc_private`, which requires a nightly toolchain -- this workspace is
//! pinned to stable `1.88.0` (`rust-toolchain.toml`, `OD-GATE-012`), and a provider
//! needing nightly would force every contributor and this gate onto one. `ra_ap_hir` is
//! rust-analyzer's own semantic-analysis engine, published as an ordinary library that
//! builds on stable Rust -- real name resolution, real type inference, real trait
//! solving, without a second toolchain.
//!
//! # What is here, and what is not
//!
//! Each capability's own contract (housed here rather than under `crates/capabilities/`,
//! the same choice `OD-CAPABILITY-002` already made for `nomos.cap.module.index` — a
//! contract lives beside its only provider until a second real party names it), its own
//! guarantee and determinism declarations, the real `ra_ap_hir` analysis, and a rule that
//! judges what it produces. `crate::reading::Load_Crate` is the one piece both
//! capabilities share: sysroot discovery and per-crate file filtering, factored out once
//! rather than solved twice. Composing either rule into a real gate run —
//! `nomos-check-orchestration::Run`, `nomos-rules::DESCRIPTORS` — is those crates' own
//! territory, the same follow-up split `nomos_lang_rust_deny`'s own capability-and-
//! provider commits left for a third, separate change to draw.

#![forbid(unsafe_code)]

mod check;
mod clone_on_copy_fact_production;
mod contract;
#[path = "fact_context.rs"]
mod provider;
mod guarantee;
#[path = "nested_lock/check.rs"]
mod nested_lock_check;
#[path = "nested_lock/contract.rs"]
mod nested_lock_contract;
#[path = "nested_lock/fact_context.rs"]
mod nested_lock_fact_context;
#[path = "nested_lock/fact_production.rs"]
mod nested_lock_fact_production;
#[path = "nested_lock/guarantee.rs"]
mod nested_lock_guarantee;
#[path = "nested_lock/reading.rs"]
mod nested_lock_reading;
mod payload;
mod reading;

pub use check::{Check_Copy_Clones, COPY_CLONES};
pub use clone_on_copy_fact_production::CloneOnCopyFactProduction;
pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use nested_lock_check::{Check_Nested_Locks, NESTED_LOCKS};
pub use nested_lock_contract::{
    Capability as Nested_Locks_Capability, Capability_Contract as Nested_Locks_Capability_Contract, Ceiling as Nested_Locks_Ceiling,
    Payload_Schema as Nested_Locks_Payload_Schema, CAPABILITY as NESTED_LOCKS_CAPABILITY, CONTRACT_VERSION as NESTED_LOCKS_CONTRACT_VERSION,
    SCHEMA as NESTED_LOCKS_SCHEMA,
};
pub use nested_lock_fact_context::{Materialize_Nested_Locks, NestedLockFact};
pub use nested_lock_fact_production::NestedLockFactProduction;
pub use nested_lock_guarantee::{Declared_Guarantee as Nested_Locks_Declared_Guarantee, Provider_Offer as Nested_Locks_Provider_Offer};
pub use nested_lock_reading::Discover_Nested_Locks;
pub use payload::clone_on_copy_payload::CloneOnCopyPayload;
pub use payload::cloned_copy_type::ClonedCopyType;
pub use payload::nested_lock_finding::NestedLockFinding;
pub use payload::nested_lock_payload::NestedLockPayload;
pub use payload::refusal::Refusal;
pub use payload::{Encode_Nested_Lock_Payload, Encode_Payload, Parse_Nested_Lock_Payload, Parse_Payload};
pub use provider::{CloneOnCopyFact, FactContext, Materialize_Crate};
pub use reading::CompilerError;
