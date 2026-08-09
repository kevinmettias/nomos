//! Band 25 — the second answer to `nomos.cap.syntax.items`.
//!
//! # Why a second provider of one capability exists
//!
//! With one provider, `nomos_capability::Registry::Resolve` is a lookup. A caller naming a
//! preference always gets it, no offer is ever below a requirement, and no result is ever
//! `Applicability::SupportedWithFallback`. Every branch that makes a registry worth having
//! is unreachable, and unreachable code is not code that works — it is code nobody has
//! observed failing.
//!
//! This crate makes those branches reachable, over the real corpus rather than over a
//! fixture.
//!
//! # Why a scanner and not another language
//!
//! Two languages are two providers of one capability only in the sense that they answer the
//! same *question*; they never answer it about the same *subject*. A Rust file and a Python
//! file are different subjects, so two language providers never contend, and none of the
//! registry branches above would be reached by adding one. Contention needs two offers over
//! one subject.
//!
//! A semantically-resolved Rust provider would contend, and needs a compiler. That is a
//! phase of work rather than a provider, and the registry's behaviour is the thing under
//! test here — not the depth of the second answer.
//!
//! # Why this one is worth having on its own terms
//!
//! It is not a strawman. `syn` refuses seven files in the scale corpus outright: they carry
//! a stray byte order mark mid-file and are not valid Rust. A parser's honest answer there
//! is nothing at all, and nothing is a bad answer to "what does this directory declare".
//!
//! A line-reader answers, weakly, and says how weakly. That is the shape of the trade this
//! whole capability system exists to make explicit — not "is there an answer" but "what is
//! this answer worth, and did the caller ask for one that good".

#![forbid(unsafe_code)]

mod guarantee;
mod provider;
mod scan;

pub use guarantee::{
    Declared_Guarantee, Payload_Schema, Provider_Offer, CAPABILITY, CONTRACT_VERSION, PROVIDER,
    SCHEMA,
};
pub use provider::{Encode_Payload, FactContext, Materialize};
pub use scan::{ItemKind, Scan, ScannedFile, ScannedItem, Visibility};
