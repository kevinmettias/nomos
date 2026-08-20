//! Band 41 — the seam for the first-class Gate object `ARC-ROADMAP-001` names, starting with
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
//! Most of that is still unbuilt. Verified directly, not assumed: `nomos-check-orchestration::
//! run::Run` calls four rules unconditionally over every source it is handed --
//! `Check_Completeness_Mirrors`, `Check_Naming_Convention`, (since
//! `P13-DEPENDENCY-WIRE-1`) `Check_Dependency_Direction`, and (since
//! `P13-CONTROLFLOW-REACHABILITY-WIRE`) `Check_Unread_Reaches_A_Finding` -- so no scope or
//! rule selection exists anywhere today. What does exist, genuinely unused for scope/rule
//! selection, is [`nomos_rules::RuleRegistry`] (`OD-RULES-004`) -- its own module doc says
//! plainly that nothing consults it for that. This crate is `Gate`'s own seam, the same
//! shape `nomos-work-orchestration`, `nomos-check-orchestration` and
//! `nomos-spec-orchestration` each are for their own verb group. Its first increment gave
//! `RuleRegistry` a real consumer: [`Run`] composes a real registry from all four of this
//! workspace's shipped rules and reports what it holds as [`GatePlan`]. It composed only
//! two until `P13-GATE-REGISTRY-THIRD-RULE`, then three until
//! `P13-CONTROLFLOW-REACHABILITY-WIRE`, which is worth keeping written down: a plan smaller
//! than the run it describes reads as "some rule is unenforced" to a caller for whom it is
//! enforced on every check, so the registry being *whole* is the claim there, not the
//! registry merely existing.
//!
//! Its second increment, `P13-GATE-RUN-FIRST-INCREMENT-2`, is [`GateRunOutcome`] and
//! [`Disposition`]: `OD-GATE-014` and `OD-GATE-015` both name the same trigger for the
//! policy types this crate still does not build -- `Gate` gaining a `run` verb that
//! actually executes rules, rather than `plan`'s report-only shape. At that increment,
//! `nomos-cli`'s `gate` module owned the actual running -- it walked a tree, called
//! `nomos_check_orchestration::Run` and reduced the result itself -- because
//! `nomos-gate-orchestration` and `nomos-check-orchestration` were both band 40, and a band
//! may not depend on its own band (`README.md`, `tests/contract/tests/boundaries/graph.rs`).
//!
//! Its third increment, `P13-GATE-RUN-SEAM-CRATE`, closes that gap the way `OD-HOST-001`
//! closed the same shape of gap for the work group: this crate moved to band 41, above
//! `nomos-check-orchestration`'s band 40, so it may depend on it, and [`Run_Gate`] now owns
//! the walk-judge-reduce composition directly -- generic over `nomos-platform`'s traits the
//! same way `nomos_check_orchestration::Run` and `nomos_work_orchestration::Run` already
//! are, so a second adapter can call it without depending on `nomos-cli`. `nomos-cli`'s
//! `gate` module has not yet been migrated to call it; it still duplicates the composition
//! `Run_Gate` now also performs, until a follow-up increment retires the copy.
//!
//! # What no increment is
//!
//! None selects by scope or rule -- [`GateCommand::root`] is accepted and carried, not
//! read beyond naming the tree, because `ScopeSelector`/`RuleSelector` do not exist yet.
//! None implements `explain` or `compare` -- those verbs have no variant here at all, not a
//! stub one, the same "no invented shape ahead of a real body" this crate's own [`command`]
//! module documents. None touches `CoveragePolicy`, `BaselinePolicy`, `SuppressionPolicy`,
//! required phases, thresholds, waivers or approvals -- every other clause `WF-001` names.
//! `Claim` (coverage debt / agent-required subjects) rides through [`GateRunResult`] for
//! information only and does not affect [`GateRunOutcome`], the same choice
//! `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code.

#![forbid(unsafe_code)]

mod command;
mod composition;
mod outcome;
mod run;
mod run_gate;

#[cfg(test)]
mod tests;

pub use command::GateCommand;
pub use composition::Registered;
pub use outcome::{Disposition, GateOutcome, GatePlan, GateRunOutcome, GateRunResult};
pub use run::Run;
pub use run_gate::Run_Gate;
