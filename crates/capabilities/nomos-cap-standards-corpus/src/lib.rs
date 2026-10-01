//! Zone: Capability Contract — the `nomos.cap.standards.corpus` contract, owned by neither
//! its provider nor any rule that reads it.
//!
//! # Why this earned a crate immediately
//!
//! `OD-CAPABILITY-002`'s criterion is contention, not principle: a contract lives beside its
//! only provider until a second real party names it. This capability has a real second party
//! from the day it was written — the reader walks a corpus the workspace does not own, and the
//! rule that reports over it is in `nomos-rules`, three zones above the provider. Between them
//! sits the only thing they agree on, which is this crate.
//!
//! # What it carries, and what it deliberately does not
//!
//! The corpus, as its own documents declare it: the population on disk, the subset declaring
//! `kind: rule`, and the documents whose declarations could not be read. It carries no
//! *judgment* about any of them. Whether a declared rule's gate is truthful against the real
//! CI wiring, whether a declared owner exists, and whether a rule reaches the files its
//! severity implies are all questions for a rule, and each of them needs evidence this
//! capability does not hold.
//!
//! What it does carry is the gate category itself, as [`nomos_contracts::GateCategory`], and
//! that is a decision rather than a convenience: the corpus's declaration and this workspace's
//! report name the identical four states, so a second type here would be the place the two
//! vocabularies drifted apart — [`standards_corpus_payload::DeclaredRule`]'s own doc carries
//! the reasoning.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); and the
//! payload shape and its canonical reader ([`standards_corpus_payload`]).
//!
//! What is not here: the front-matter reader that walks a corpus and produces a payload, which
//! is `nomos-repo-policy`'s; and every rule that reports over one, which is `nomos-rules`'s.

#![forbid(unsafe_code)]

mod contract;
mod standards_corpus_payload;

pub use contract::{Capability, Capability_Contract, Ceiling, CAPABILITY, CONTRACT_VERSION, Payload_Schema, SCHEMA};
pub use standards_corpus_payload::{
    Declared_Gate, DeclarationIssue, DeclarationKind, DeclaredRule, DocumentDeclaration, Encode_Payload, Parse_Payload,
    Refusal, RuleSeverity, StandardsCorpusPayload,
};
