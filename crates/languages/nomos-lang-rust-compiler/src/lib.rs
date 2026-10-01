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
//! The real `ra_ap_hir` analysis, each capability's own guarantee and determinism
//! declaration, and the fact each analysis produces. `crate::reading::Load_Crate` is the one
//! piece both capabilities share: sysroot discovery and per-crate file filtering, factored
//! out once rather than solved twice.
//!
//! What is **not** here any more is either capability's contract or either rule. Both
//! contracts lived here while this provider was each capability's only party, which
//! `OD-CAPABILITY-002` licenses; `OD-ANALYSIS-007` version 2 decided the second party that
//! ends that arrangement is a `nomos-rules` descriptor naming the capability -- because this
//! crate is zoned `Provider` and `Permits` forbids `Rules` from naming `Provider` -- and
//! that the contracts then move to one crate per capability rather than one for the family.
//! They are `nomos-cap-rust-copy-clones` and `nomos-cap-rust-nested-locks`, and this crate
//! offers against both. The two rules moved with them, into `nomos-rules`, where every other
//! composed rule is written. What that leaves here is a crate that declares no contract and
//! judges nothing, which is exactly the criterion `OD-CAPABILITY-015` settles the zone on:
//! still `Provider`.

#![forbid(unsafe_code)]

mod clone_on_copy_fact_production;
#[path = "fact_context.rs"]
mod provider;
mod guarantee;
#[path = "nested_lock/fact_context.rs"]
mod nested_lock_fact_context;
#[path = "nested_lock/nested_lock_fact_production.rs"]
mod nested_lock_fact_production;
#[path = "nested_lock/guarantee.rs"]
mod nested_lock_guarantee;
#[path = "nested_lock/reading.rs"]
mod nested_lock_reading;
mod reading;

pub use clone_on_copy_fact_production::CloneOnCopyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use nested_lock_fact_context::{Materialize_Nested_Locks, NestedLockFact};
pub use nested_lock_fact_production::NestedLockFactProduction;
pub use nested_lock_guarantee::{Declared_Guarantee as Nested_Locks_Declared_Guarantee, Provider_Offer as Nested_Locks_Provider_Offer};
pub use nested_lock_reading::Discover_Nested_Locks;
pub use provider::{CloneOnCopyFact, FactContext, Materialize_Crate};
pub use reading::CompilerError;
