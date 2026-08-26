//! Band 19 — the ledger-agnostic primitives `OD-LEDGER-037` found underneath
//! `nomos-agent-contracts`'s reuse of `nomos-ledger`.
//!
//! [`Territory`] is a set of path patterns with `Covers`/overlap logic; [`VerificationPredicate`]
//! is a program-and-argv pair that decides whether something is done. Neither type encodes a
//! ledger claim, a lease or a holder — nothing about either shape is specific to work-ledger
//! coordination. `nomos-ledger` uses them for its own claim/overlap and finish logic over
//! `work/ledger.json`; `nomos-agent-contracts`'s `TaskEnvelope`, `NomosResolvedChangeContext` and
//! `WorkResult` use the identical shapes for `AGT-007`'s "permitted scope" and "required
//! verification" — a product-level agent-safety contract with no ledger claim anywhere near it.
//!
//! `OD-LEDGER-037` measured both types as already general-purpose in everything but name and
//! package, and named this crate's shape without building it: this crate is that follow-on.
//! Both types moved here **verbatim** — unchanged in field shape and behavior — and
//! `nomos-ledger` re-exports them for its own use exactly as `nomos-lang-rust-package`
//! re-exports `nomos-package`'s domains (`OD-PACKAGE-007`): a behavior-preserving move, not a
//! redesign.

#![forbid(unsafe_code)]

mod territory;
mod verification_predicate;

pub use territory::{Normalize_Path, Territory};
pub use verification_predicate::VerificationPredicate;
