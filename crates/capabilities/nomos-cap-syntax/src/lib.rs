//! Band 23 — the `nomos.cap.syntax.items` contract, owned by neither provider of it.
//!
//! # Why a contract has a home of its own
//!
//! A [`nomos_capability::CapabilityContract`] is the agreed meaning of a question and the
//! ceiling on what any answer may claim. An agreement is not the property of one party to
//! it.
//!
//! Until this crate existed, `nomos-lang-rust` declared the contract and
//! `nomos-lang-rust-scan` offered against it — against a contract its peer authored, which
//! it could not name, because two providers of one capability sit at the same band and
//! `tests/contract` forbids the edge. So the second provider agreed with the first by
//! retyping four string constants, and nothing would have noticed the day one of them was
//! retyped differently. What held the invariant was `Registry::Declare` refusing a second
//! contract for one capability: the registry compensating for the layering rather than the
//! layering being right.
//!
//! # What is here and what is not
//!
//! Here: what the parties agreed. The capability's identity, the contract version, the
//! ceiling, the summary that says what the question means, and the schema every answer is
//! stamped with — because two providers writing different shapes would force every consumer
//! to know which one answered, which is the thing resolving through a registry exists to
//! avoid.
//!
//! And, since `P10-SYNTAX-SCHEMA`, the *shape* of an answer rather than only its name: the
//! grammar of `nomos.syntax.items.v1` and the one reader every consumer uses. A schema
//! identifier with no written grammar is a name for an agreement nobody wrote down, and it
//! had three independent ideas of what a well-formed payload is. The writers stay with
//! their providers on purpose — [`Parse_Payload`] carries the grammar and the reason the
//! reader and the writer are answered differently.
//!
//! Not here: anything a provider claims for itself. A `ProviderId` is a provider's own name
//! and a [`nomos_contracts::Guarantee`] is its own claim, bounded by the ceiling below and
//! checked against its output by its own tests. A contract that also declared what each
//! provider promises would be grading their work for them, which is the defect the ceiling
//! exists to prevent, one level up.
//!
//! # When a contract earns a crate
//!
//! When more than one party names it. A capability with a single provider is not wrongly
//! filed for living beside that provider — `nomos.cap.module.surface` is declared and
//! offered by the same rollup in `tests/integration`, and moving it out would buy a crate
//! and no property.
//!
//! The moment a second provider exists, ownership by one party stops being merely untidy
//! and becomes false: the first provider can change the ceiling, the version or the schema
//! its peer is bound by, and the peer cannot even see the file. That is the criterion —
//! contention, not principle. It is the same shape as `OD-STORE-001`'s: a thing earns its
//! place when something must *behave* differently, and not when a diagram would look
//! tidier.

#![forbid(unsafe_code)]

mod contract;
mod payload;
mod payload_item;
mod payload_refusal;
mod syntax_payload;

pub use contract::{
    Capability, Capability_Contract, Payload_Schema, Ceiling, CAPABILITY, CONTRACT_VERSION, SCHEMA,
};
pub use payload::{
    Escape, Function_Arity, Function_Shape, Observation, Parse_Payload, Render_Payload, Unescape,
    FUNCTION, IMPLEMENTATION, INHERENT, NOT_APPLICABLE, PUBLIC, SLICE, TRAIT, VALUE,
};
pub use payload_item::PayloadItem;
pub use payload_refusal::PayloadRefusal;
pub use syntax_payload::SyntaxPayload;
