//! Band 2 — the first thing in this workspace that reads somebody else's code.
//!
//! Everything above this band has so far been machinery reasoning about facts it
//! produced itself. This crate produces them from a corpus it did not write, which is
//! the first place the design meets something that can disagree with it.
//!
//! # What a provider owes its callers
//!
//! Not "syntax facts". A caller cannot plan around that phrase, and every real analyzer
//! is weaker than its name suggests in a way its users discover at the worst moment. So
//! this crate states its limits as a value: [`Declared_Guarantee`] says the resolution
//! level, whether the output is sound, whether it is complete, and how finely it can be
//! refreshed. [`nomos_capability::Registry`] refuses an offer that claims more than the
//! capability's ceiling, so the claim is checked rather than believed.
//!
//! The guarantee is also checked against the output. `tests/guarantee.rs` asserts one
//! property per axis, including the axis this provider *fails* — completeness is
//! [`nomos_contracts::Assurance::Unknown`] and there is a test that would fail if the
//! output were quietly complete after all. A declared weakness nobody exercises is a
//! comment.
//!
//! # Three outcomes, and why none of them is zero
//!
//! A file this provider will not read ([`Recognition::Unrecognized`]), a file it cannot
//! read ([`Reading::Unparseable`]) and a file that genuinely declares nothing
//! ([`Reading::Parsed`] with no items) are three different facts about the world, and
//! the prototype learned the expensive way that collapsing any pair of them produces a
//! run that reports a clean corpus because it never read one. None of these types has an
//! `is_ok`, a `Default`, or an `Option` a caller could `unwrap_or_default` into silence.
//!
//! # Scope
//!
//! For `nomos.cap.syntax.items`: items — every declaration form Rust has — at every
//! syntactic nesting depth, with the visibility each one declares. Not expressions, not
//! statements, not enum variants and not struct fields. The boundary is stated rather than
//! discovered: an item is the unit at which Rust attaches visibility and a name, which is
//! the unit that capability is about.
//!
//! [`reachability`] is narrower reach into expressions than the sentence above once
//! claimed for this whole crate — one arm shape inside one function body, for
//! `nomos.cap.controlflow.reachability`. The two readings are independent passes over the
//! same parse tree; neither claims the other's boundary.
//!
//! # Three capabilities, and why the second and third are here
//!
//! [`Materialize_Syntax_Fact`] answers `nomos-cap-syntax`'s capability about one file, from its bytes.
//! [`rollup`] answers a second one about a *module*, from the first one's facts — the only
//! producer in this workspace that derives a fact from other facts, and therefore the only
//! one that declares a dependency edge. [`reachability::Materialize_Reachability_Fact`] answers a third,
//! `nomos-cap-controlflow`'s, about one file, from its bytes — the same shape as the
//! first, and housed here for the identical reason `nomos-cap-controlflow`'s own crate doc
//! gives: this crate is a real second party to `nomos.cap.syntax.items` from the day this
//! third capability was written, and it already owns the `syn` parse both readings run.
//!
//! Rollup is not a widening of the first. A capability whose semantic input is one file's
//! text can never depend on another answer, so the dependent half of `nomos-analysis`'s
//! invalidation had no producer it could possibly have had until a second capability
//! existed. `docs/records/OD-ANALYSIS-002` records that, and why the second contract lives
//! beside its only provider rather than under `crates/capabilities`. The third capability's
//! own contract, unlike the second's, lives under `crates/capabilities` from the start —
//! `nomos-cap-controlflow`'s own crate doc says why.

#![forbid(unsafe_code)]

#[path = "syntax_fact_production.rs"]
mod determinism;
mod guarantee;
mod outcome;
#[path = "fact_context.rs"]
mod provider;
#[path = "reachability_reading.rs"]
pub mod reachability;
pub mod rollup;
mod syntax;

pub use determinism::SyntaxFactProduction;
pub use guarantee::{Declared_Guarantee, PROVIDER, Provider_Offer};
pub use outcome::{Materialization, ParseFailure, Reading, Recognition, RUST_EXTENSION};
pub use provider::{Encode_Payload, FactContext, Materialize_Syntax_Fact};
pub use syntax::{Facts, Item, ItemKind, Read_Source, Visibility};
