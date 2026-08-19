//! Band 40 — the seam for the first-class Gate object `ARC-ROADMAP-001` names, starting with
//! `Plan`.
//!
//! `ARC-ROADMAP-001` distinguishes `OD-GATE-004`'s CI step, which wires this repository's own
//! rule layer into its own CI, from a product-level `Gate`: "a first-class object an
//! end-user repository would configure (`ScopeSelector`, `RuleSelector`,
//! `ApplicabilityPolicy`, `CoveragePolicy`, `BaselinePolicy`, `SuppressionPolicy`, required
//! phases, evidence requirements, failure disposition), exposed as `nomos gate plan` / `run`
//! / `explain` / `compare`". `WF-001` (the v14 corpus) makes it binding: "A gate shall define
//! policy: required phases, thresholds, coverage, unsupported-analysis policy, waivers,
//! approvals, and blocking behavior."
//!
//! Nothing in this workspace implements any of that yet. Verified directly, not assumed:
//! `nomos-check-orchestration::run::Run` calls three rules unconditionally over every source
//! it is handed -- `Check_Completeness_Mirrors`, `Check_Naming_Convention`, and (since
//! `P13-DEPENDENCY-WIRE-1`) `Check_Dependency_Direction` -- so no scope or rule selection
//! exists anywhere today. What does exist, genuinely unused, is [`nomos_rules::RuleRegistry`]
//! (`OD-RULES-004`) -- its own module doc says plainly that nothing consults it. This crate is
//! `Gate`'s own seam, the same shape `nomos-work-orchestration`, `nomos-check-orchestration`
//! and `nomos-spec-orchestration` each are for their own verb group, and its first increment
//! gives `RuleRegistry` a real consumer: [`Run`] composes a real registry from all three of
//! this workspace's shipped rules and reports what it holds as [`GatePlan`]. It composed only
//! two until `P13-GATE-REGISTRY-THIRD-RULE`, which is worth keeping written down: a plan
//! smaller than the run it describes reads as "dependency direction is unenforced" to a
//! caller for whom it is enforced on every check, so the registry being *whole* is the claim
//! here, not the registry merely existing.
//!
//! # What this increment is not
//!
//! It does not select by scope or rule -- [`GateCommand::root`] is accepted and carried, not
//! read, because `ScopeSelector`/`RuleSelector` do not exist yet. It does not implement
//! `run`, `explain` or `compare` -- those verbs have no variant here at all, not a stub one,
//! the same "no invented shape ahead of a real body" this crate's own [`command`] module
//! documents. It does not touch `CoveragePolicy`, `BaselinePolicy`, `SuppressionPolicy`,
//! required phases, thresholds, waivers, approvals or blocking behavior -- every other clause
//! `WF-001` names. It does not change what `nomos check` runs: `nomos-check-orchestration`'s
//! own composition and `Run` are untouched, and nothing calls this crate yet -- the same
//! "additive and unwired" shape `RuleRegistry` itself has carried since `OD-RULES-004`.

#![forbid(unsafe_code)]

mod command;
mod composition;
mod outcome;
mod run;

#[cfg(test)]
mod tests;

pub use command::GateCommand;
pub use composition::Registered;
pub use outcome::{GateOutcome, GatePlan};
pub use run::Run;
