//! The five `nomos.cap.*.policy` providers (naming, limits, scripting, words, goals),
//! consolidated from six crates into one.
//!
//! `OD-RULES-011` gave `nomos.cap.naming.policy` its own capability contract and its own
//! provider crate; four more families followed the identical crate-per-capability shape by
//! precedent. `OD-RULES-019` later measured the five providers' independently duplicated
//! `standards.json` read step and extracted it into a sixth crate, while deciding the five
//! providers "stay exactly as separate as `OD-RULES-011` built them."
//!
//! `OD-PACKAGE-015` asked a question neither record had: does a crate boundary, rather
//! than a module boundary, earn its packaging cost here? Grepped against every consumer in
//! this workspace, the answer was no for all six — no independent versioning, no enforced
//! isolation, and the two real consumers (`nomos-check-orchestration` and
//! `tests/integration`) already depended on all five providers together, every time. This
//! crate carries that finding out: [`naming`], [`limits`], [`scripting`], [`words`] and
//! [`goals`] are each still their own capability, their own `ProviderId`, their own
//! registration — reached through a module rather than a `Cargo.toml`. `standards_document`
//! is the shared read step `OD-RULES-019` decided the five owe, private to this crate since
//! nothing outside its five siblings ever depended on it alone. `scaffolding` is a second,
//! later extraction beneath the same five: the registration and fact-assembly plumbing every
//! one of them repeated identically, distinct from what each provider's own `reading.rs`,
//! `Declared_Guarantee` and `Encode_Payload` still decide independently -- see `scaffolding`'s
//! own doc for the boundary and why it holds.

#![forbid(unsafe_code)]

mod scaffolding;
mod standards_document;
pub mod goals;
pub mod limits;
pub mod naming;
pub mod scripting;
pub mod words;
