//! Zone: Provider — the first real second party to `nomos.cap.syntax.items`, alongside
//! `nomos-lang-rust` and `nomos-lang-rust-scan`. Same band, so none of the three may name
//! either of the others.
//!
//! `nomos-lang-rust` was the first thing in this workspace to read somebody else's code.
//! Until this crate existed, the capability it answers had exactly one provider, and every
//! choice that crate made — what an item is, how a guarantee is stated, what three outcomes
//! a reading can have — was untested against a second language that might disagree. This
//! crate makes the same choices for Go, or states plainly where Go's own shape forces a
//! different one, and is the second party that contract had never had.
//!
//! # What a provider owes its callers
//!
//! The same answer `nomos-lang-rust` gives: not "syntax facts", but [`Declared_Guarantee`]
//! stated as a value, checked against the ceiling `nomos_capability::Registry` refuses to
//! let it exceed, and checked against this provider's own output by `tests/guarantee.rs`.
//!
//! # Three outcomes, and why none of them is zero
//!
//! A file this provider will not read ([`Recognition::Unrecognized`]), a file it cannot
//! read ([`Reading::Unparseable`]) and a file that genuinely declares nothing
//! ([`Reading::Parsed`] with no items) are three different facts, for the identical reason
//! `nomos-lang-rust`'s own crate doc gives.
//!
//! # Scope
//!
//! For `nomos.cap.syntax.items`: every package-level declaration form Go has — functions,
//! methods, struct and interface types, type definitions and aliases, constants, variables,
//! imports and an interface's own method set — with the visibility each one declares. Not
//! statements, and not struct fields: the same "not expressions, not statements" boundary
//! `nomos-lang-rust` states, drawn at the unit each language attaches a name and a visibility
//! to. An interface's method set *is* included, the same way a Rust trait's member list is —
//! see [`Read_Source`]'s own module for why that inclusion, unlike the Rust provider's,
//! carries a real visibility rather than [`Visibility::NotApplicable`].
//!
//! # Sound *and* complete — the one claim `nomos-lang-rust` cannot make
//!
//! [`Declared_Guarantee`] claims [`nomos_contracts::Assurance::Sound`] on both axes, not one.
//! Go has no macro system: no construct where this provider's parse tree ends and an
//! unexpanded token stream begins in its place, which is exactly the gap that keeps
//! `nomos-lang-rust`'s own completeness at `Unknown`. [`guarantee`]'s own doc comment carries
//! the full reasoning, including why generics and build tags were checked against the real
//! grammar rather than assumed not to matter.
//!
//! # Why `tree-sitter`, and why any parse error refuses the whole file
//!
//! `syn` either parses a file or refuses it outright; `tree-sitter` does error-recovery
//! parsing and will hand back a tree for genuinely broken input, with `ERROR` and `MISSING`
//! nodes marking where it guessed. [`Read_Source`] treats any such mark anywhere in the tree
//! as the whole file being [`Reading::Unparseable`] — reading a node from a recovered region
//! as a fact would be reporting the parser's guess as something the source actually said,
//! which the [`Assurance::Sound`](nomos_contracts::Assurance::Sound) claim above forbids.

#![forbid(unsafe_code)]

#[path = "syntax_fact_production.rs"]
mod determinism;
mod guarantee;
mod materialization;
mod parse_failure;
#[path = "fact_context.rs"]
mod provider;
mod reading;
mod recognition;
mod syntax;

pub use determinism::SyntaxFactProduction;
pub use guarantee::{Declared_Guarantee, LANGUAGE, PROVIDER, Provider_Offer};
pub use materialization::Materialization;
pub use parse_failure::ParseFailure;
pub use provider::{Encode_Payload, FactContext, Materialize_Syntax_Fact};
pub use reading::Reading;
pub use recognition::{GO_EXTENSION, Recognition};
pub use syntax::{ItemKind, Read_Source, Facts, Item, Visibility};
