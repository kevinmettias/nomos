//! Zone: Provider — the third language in this workspace to read somebody else's code, and
//! the fourth party to `nomos.cap.syntax.items` alongside `nomos-lang-rust`,
//! `nomos-lang-rust-scan` and `nomos-lang-go`. Same zone, so none of the four may name any
//! other.
//!
//! # What a third language settles that a second could not
//!
//! `nomos-lang-go` proved a second language can exist without touching Rust's
//! implementation. It could not prove that the shape it followed generalizes, because Go was
//! chosen partly for being easy on this contract: no macro system, one declaration form per
//! keyword, visibility readable from an identifier's first letter. C# agrees with none of
//! that. Visibility is a modifier list with a compound form (`protected internal`), a type
//! can be declared across several files (`partial`), a declaration can be nested arbitrarily
//! deep inside another, and a preprocessor decides which declarations a compilation contains
//! at all. Every one of those is a place the convention could have needed changing, and
//! [`Declared_Guarantee`] records which one actually did.
//!
//! # What a provider owes its callers
//!
//! The same answer its three siblings give: not "syntax facts", but [`Declared_Guarantee`]
//! stated as a value, checked against the ceiling `nomos_capability::Registry` refuses to let
//! it exceed, and checked — on the axis where this provider is weaker than its Go peer —
//! against a real reading of real C# by this crate's own tests.
//!
//! # Three outcomes, and why none of them is zero
//!
//! A file this provider will not read ([`Recognition::Unrecognized`]), a file it cannot read
//! ([`Reading::Unparseable`]) and a file that genuinely declares nothing ([`Reading::Parsed`]
//! with no items) are three different facts, for the identical reason `nomos-lang-rust`'s own
//! crate doc gives.
//!
//! # Scope
//!
//! The four declaration categories a C# file has: its namespaces, its `using` directives, its
//! types (class, struct, interface, record, enum, delegate — nested ones included, qualified
//! by what encloses them) and its type members (method, constructor, finalizer, operator,
//! property, indexer, field, event and enum member), each with the visibility its own
//! modifiers declare.
//!
//! Not statements, not local functions, not lambdas, and not an accessor inside a property's
//! own `{ get; set; }` — the same "not expressions, not statements" boundary
//! `nomos-lang-rust` states, drawn at the unit C# attaches a name and an accessibility to. A
//! positional record parameter (`record Pair(int Left, int Right)`) is not recorded either:
//! the language turns it into a property, but the file spells a parameter, and inventing the
//! property would be reporting a translation rather than a reading — the same restraint
//! `nomos-lang-go` takes for an embedded struct field.
//!
//! # The one axis where C# is weaker than Go, and why
//!
//! [`Declared_Guarantee`] claims [`nomos_contracts::Assurance::Sound`] soundness and
//! [`nomos_contracts::Assurance::Unknown`] completeness. The reason is the preprocessor:
//! `#if`/`#elif`/`#else` regions are real subtrees in this grammar, both branches of one
//! present at once, and at most one branch is in any given compilation. [`Read_Source`]
//! therefore reads no declaration inside a conditional region and counts the regions instead,
//! into the payload header every provider of this capability writes. [`guarantee`]'s own doc
//! comment carries the full reasoning, including why generics, nested types, `partial` and
//! file-scoped namespaces were each checked against the real grammar rather than assumed to
//! matter or not to.
//!
//! # Why `tree-sitter`, and why any parse error refuses the whole file
//!
//! For the identical reason `nomos-lang-go` states: `tree-sitter` does error-recovery parsing
//! and will hand back a tree for genuinely broken input, with `ERROR` and `MISSING` nodes
//! marking where it guessed. [`Read_Source`] treats any such mark anywhere in the tree as the
//! whole file being [`Reading::Unparseable`], because reading a node from a recovered region
//! as a fact would be reporting the parser's guess as something the source actually said.

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
pub use recognition::{CSHARP_EXTENSION, Recognition};
pub use syntax::{ItemKind, Read_Source, Facts, Item, Visibility};
