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
//! Items — every declaration form Rust has — at every syntactic nesting depth, with the
//! visibility each one declares. Not expressions, not statements, not enum variants and
//! not struct fields. The boundary is stated rather than discovered: an item is the unit
//! at which Rust attaches visibility and a name, which is the unit the capability is
//! about.

#![forbid(unsafe_code)]

mod determinism;
mod guarantee;
mod provider;
mod recognition;
mod syntax;

pub use determinism::SyntaxFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{Encode_Payload, FactContext, Materialization, Materialize};
pub use recognition::{Recognition, RUST_EXTENSION};
pub use syntax::{ItemKind, ParseFailure, Read_Source, Reading, SyntaxFacts, SyntaxItem, Visibility};
