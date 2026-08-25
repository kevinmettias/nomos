//! The grammar of `nomos.syntax.items.v2`, and the reader the tree agrees on.
//!
//! Until this module existed, the schema was a [`nomos_contracts::SchemaId`] string and
//! nothing else. Two providers authored the bytes independently and three consumers read
//! them back, each with its own idea of what a well-formed payload is — so the *shape* of
//! an answer that two parties are bound by was the part of the agreement with no home,
//! which is the same objection that moved the capability, the ceiling and the version here.
//!
//! # The grammar
//!
//! A payload is UTF-8 text. Fields within a record are separated by a single tab, records
//! are terminated by `\n`, and never by `\r\n` — a digest that depends on the line ending
//! of the machine that produced it is not a content address.
//!
//! ```text
//! payload     := header item*
//! header      := "unexpanded" TAB u32 LF
//! item        := "item" TAB ordinal TAB kind TAB visibility TAB qualified-name
//!                TAB documentation TAB shape LF
//! ordinal     := u32
//! documentation := observation
//! shape         := observation
//! observation := "-" | "." | "+" escaped
//! escaped     := any text, with `\` `\t` `\n` `\r` written `\\` `\t` `\n` `\r`
//! ```
//!
//! **The header is the first record and appears exactly once.** It carries a lower bound on
//! the places the provider saw the parse tree end in unexpanded tokens. A provider that
//! cannot observe macro invocations writes `0`, and that is the format saying the same
//! thing in every payload rather than a claim that there were none.
//!
//! **`item` records are in source order** and `ordinal` is the item's position in that
//! order, starting at zero. Source order is load-bearing rather than cosmetic: a member is
//! attributed to the most recent enclosing record before it, and a consumer that sorted the
//! items would attribute members to the wrong owner.
//!
//! A file that declares nothing is a header and no items — a real answer, and
//! distinguishable from a payload that could not be read only because the empty byte string
//! is refused rather than decoded.
//!
//! **`kind` and `visibility` are labels, and the schema does not enumerate them.** A
//! provider that can distinguish fewer forms than another writes fewer labels; a kind it
//! cannot distinguish is a kind it must not claim. Fixing the vocabulary here would make
//! the weaker provider unable to answer honestly, and the ceiling already exists to bound
//! what an answer may claim.
//!
//! **`qualified-name` is the name as written, qualified by syntactic nesting** — a function
//! declared in `mod tests` arrives as `tests::Name`, and one declared in an `impl` block
//! arrives as `Type::Name`. It is not a resolved path: no provider of this capability
//! resolves names, and a `::` in it is nesting rather than a module route.
//!
//! # Not observed is not absent
//!
//! This is why there is a v2, and it is the whole of the difference.
//!
//! `documentation` and `shape` are [`Observation`]s rather than strings, because two
//! providers of one capability differ in *what they can see* and not only in how well.
//! `nomos-lang-rust-scan` cannot read a doc comment at all — it skips comment lines and
//! associates nothing with the item below them. If "this item has no documentation" and
//! "this provider does not read documentation" were the same bytes, every list read through
//! the scanner would arrive as a list declaring no mirror: a phantom mirror silently
//! downgraded to an admitted gap, which is absence becoming success in the one field a
//! completeness rule's severity ordering turns on.
//!
//! So the three states are three spellings. `-` is *not observed*; `.` is *observed and
//! there is none*; `+…` is *observed and here it is*. A consumer that cannot act on an
//! unobserved field must say so rather than treat it as empty, and the type makes that
//! difficult to get wrong by accident.
//!
//! Observation is bounded by the provider's guarantee and not by its effort. A provider
//! writes `-` when the method it used cannot see the thing, which is a property of the
//! method — the same property its declared [`nomos_contracts::Guarantee`] describes.
//!
//! ## What `shape` says, per kind
//!
//! Open like the other vocabularies, and meaningful relative to the kind:
//!
//! | For a | `shape` is |
//! |---|---|
//! | typed declaration — a constant, a static | [`SLICE`] when the declared type is a slice or an array, however many references deep, and [`VALUE`] otherwise |
//! | function | `fn/<arity>`, the number of declared parameters including a receiver — read it with [`Function_Arity`] |
//! | implementation block | [`INHERENT`] or [`TRAIT`] |
//! | struct with named fields | `fields` followed by one `name`-TAB-`type` line per field, in declaration order — read it with [`Struct_Fields`], write it with [`Struct_Shape`]. `type` is each provider's own honest answer to "the type as spelled," bounded by what that provider can see without resolving a name (`OD-CAPABILITY-010`) |
//! | struct with no named fields (a unit or tuple struct) | `.` — observed, and there are no named fields to report, the same default every other kind already has |
//! | anything else | `.` — observed, and the shape has nothing to say about this form |
//!
//! The distinctions are the ones a consumer cannot recover from the rest of the record and
//! could otherwise only get by parsing the source a second time. `pub const LIMIT: usize`
//! and `pub const TABLES: &[&str]` are identical in every other field; `fn All()` and
//! `fn All(&self)` are identical in every other field; and an `All` in `impl Display for T`
//! does not belong to `T` the way an `All` in `impl T` does.
//!
//! # What this schema declines to state
//!
//! **Whether a `Function` record is a definition or a signature.**
//!
//! It looks as though it could. `nomos-lang-rust` writes [`NOT_APPLICABLE`] for a trait
//! member, because a trait method declares no visibility of its own and recording it as
//! private would be inventing a declaration the source does not contain. So under that
//! provider, a `Function` marked [`NOT_APPLICABLE`] *is* a trait-method signature.
//!
//! It is not a property of the schema, because it is not a property of every conforming
//! answer. `nomos-lang-rust-scan` has no [`NOT_APPLICABLE`] value at all — a line reader
//! cannot see the enclosing trait — and writes `Private` for the same source construct.
//! Both payloads conform. So the presence of the mark is evidence and its **absence is
//! not**, and a consumer that reads absence as "this is a definition" is reading a weaker
//! provider's blindness as an observation.
//!
//! Visibility is the one field where that asymmetry is still unmarked, and v2 does not fix
//! it: making it an [`Observation`] would say that a provider did not observe visibility at
//! all, which is false — the scanner observes `pub` correctly and merely cannot see the
//! enclosing form. The consequence is stated instead: a consumer needing the distinction
//! must obtain it from the *guarantee* it required of the answer, not from the bytes. That
//! is what `nomos-rules` does, and `OD-RULES-001`'s floor is what makes it sound.
//!
//! # One reader, two writers
//!
//! The reader is here and canonical. Every consumer uses it, because a consumer writing its
//! own is not proving anything — it is re-deciding what a well-formed payload is, and three
//! answers to that question is how a payload comes to decode differently under one
//! capability.
//!
//! The writers stay where they are, deliberately. What makes two providers interchangeable
//! is that both produce bytes a third party can read; a shared encoder would make that true
//! by construction and prove nothing. What the duplication used to cost is that nothing
//! checked the agreement — so each provider decodes its own output through this reader in
//! its own tests, which is the check the duplication was missing rather than the duplication
//! removed.

mod observation;
mod parse;
mod render;
#[cfg(test)]
mod tests;

pub use observation::{
    FUNCTION, Function_Arity, Function_Shape, IMPLEMENTATION, INHERENT, NOT_APPLICABLE, Observation, PUBLIC, SLICE,
    Struct_Fields, Struct_Shape, TRAIT, VALUE,
};
use observation::Observed;
pub use parse::Parse_Payload;
pub use render::Escape;
pub(crate) use render::Unescape;
pub use render::Render_Payload;

use crate::PayloadItem;
use crate::PayloadRefusal;
use crate::PayloadRefusalKind;
use crate::SyntaxPayload;
