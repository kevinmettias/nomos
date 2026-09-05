//! Band 25 — the one provider of `nomos.cap.rust.copy_clones`.
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
//! # Why this crate answers one narrow question rather than a broad one
//!
//! `crates/languages/nomos-lang-rust-compiler/src/lib.rs`'s own task was not to build a
//! `CompilerProvider` or `LspProvider` taxonomy first -- one concrete provider, real
//! evidence, and then whether a shared abstraction is warranted at all is a question the
//! evidence answers, not a decision made ahead of it. This crate picks the narrowest
//! real question a compiler frontend answers that a syntax tree cannot: whether a
//! `.clone()` call's receiver already implements `Copy`, which needs full name
//! resolution and a trait-implementation lookup, not merely a parse.
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
//! The capability contract (housed here rather than under `crates/capabilities/`, the
//! same choice `OD-CAPABILITY-002` already made for `nomos.cap.module.index` — a
//! contract lives beside its only provider until a second real party names it), this
//! provider's own guarantee and determinism declarations, the real `ra_ap_hir` analysis,
//! and a rule that judges what it produces. Composing that rule into a real gate run —
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
mod payload;
mod reading;

pub use check::{Check_Copy_Clones, COPY_CLONES};
pub use clone_on_copy_fact_production::CloneOnCopyFactProduction;
pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use payload::clone_on_copy_payload::CloneOnCopyPayload;
pub use payload::cloned_copy_type::ClonedCopyType;
pub use payload::refusal::Refusal;
pub use payload::{Encode_Payload, Parse_Payload};
pub use provider::{CloneOnCopyFact, FactContext, Materialize_Crate};
pub use reading::CompilerError;
