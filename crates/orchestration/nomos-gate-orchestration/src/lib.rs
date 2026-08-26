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
//! Most of that is still unbuilt. Verified directly, not assumed, at this crate's own start:
//! `nomos-check-orchestration::run::Run` called four rules unconditionally over every source
//! it was handed -- `Check_Completeness_Mirrors`, `Check_Naming_Convention`, (since
//! `P13-DEPENDENCY-WIRE-1`) `Check_Dependency_Direction`, and (since
//! `P13-CONTROLFLOW-REACHABILITY-WIRE`) `Check_Unread_Reaches_A_Finding` -- so no scope or
//! rule selection existed anywhere yet. What existed then, genuinely unused for scope/rule
//! selection, was [`nomos_rules::RuleRegistry`] (`OD-RULES-004`) -- its own module doc said
//! plainly that nothing consulted it for that. `OD-GATE-017` has since given `Run` itself a
//! real per-call `selected: &[RuleId]` gate over every rule it runs, in
//! `nomos-check-orchestration` directly rather than through this crate or `RuleRegistry` --
//! see this crate's fourth increment below for the layer of selection this crate built
//! first, over what a finding can fail the build for rather than over what `Run` executes.
//! This crate is `Gate`'s own seam, the same
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
//! are, so a second adapter can call it without depending on `nomos-cli`. `P13-GATE-RUN-SEAM-
//! CLI` then migrated `nomos-cli`'s `gate` module to call it instead of duplicating the
//! composition.
//!
//! Its fourth increment, `P13-GATE-014-SCOPE-RULE-SELECTORS`, gives [`GateCommand`] real
//! [`ScopeSelector`] and [`RuleSelector`] fields under the user's standing override of
//! `OD-GATE-014`'s wait for a real caller -- `OD-GATE-014`'s own module doc records the
//! override rather than this one repeating it. [`ScopeSelector`] filters
//! `nomos_rules::SourceFile::path` by textual prefix containment before
//! `nomos_check_orchestration::Run` is called, so an out-of-scope file is not judged at all.
//! [`RuleSelector`] filters findings by `nomos_contracts::RuleId` after `Run` returns,
//! before [`Disposition`] reduces them -- at this increment, a real selection of what can
//! fail a build, but honestly short of a real selection of what runs: `Run` still executed
//! every rule unconditionally, and narrowing that needed a signature change to `Run` itself
//! across every caller, which was not this increment. `OD-GATE-017` has since made that
//! signature change directly in `nomos-check-orchestration::run::Run` -- its own
//! `selected: &[RuleId]` gates each rule's materialization, not only its finding -- so
//! [`RuleSelector`] here now layers a post-hoc, per-caller finding filter over a `Run` that
//! is already selective on its own. Both fields default to select-everything, so every
//! construction site that predates them and CI's own `gate run --root .` are unchanged in
//! behavior.
//!
//! Its fifth increment, `P13-GATE-015-SUPPRESSION-FIRST-INCREMENT`, gives [`GateCommand`] a
//! real [`SuppressionPolicy`] under the same user override, taking only the suppression
//! concern of `OD-GATE-015`'s three -- `SuppressionPolicy` matches a [`nomos_contracts::
//! Finding`] by `rule` and `subject`, the identity `Finding` already carries, and a matched
//! finding cannot fail the build but still appears in [`GateRunResult::check_outcome`] and
//! [`GateRunResult::suppressed_findings`], never silently. No CLI flag or config file
//! constructs a [`crate::Suppression`] yet -- inventing an authoring surface before a real
//! caller needs one would repeat the mistake `OD-GATE-015` already declined, so this
//! increment is the type and its consultation only.
//!
//! Its sixth increment, `P13-GATE-EXPLAIN-FIRST-INCREMENT`, gives `explain` its first real
//! body: [`Explain_Gate`] judges a tree exactly as [`Run_Gate`] does (through a shared
//! [`run_gate::Judged`] helper, so the two do not duplicate the walk/empty/unreadable
//! match) and answers a [`FindingQuery`] -- a rule and one of a finding's own human-visible
//! locations, not a digest -- with an [`Explanation`]. Deliberately independent of
//! `GateCommand::scope`/`rules`: `explain` answers what one named finding looks like right
//! now, not what a scope- or rule-narrowed `run` would currently see. `suppressions` is
//! still consulted, because whether a suppression applies is part of the finding's own
//! explanation.
//!
//! Its seventh increment, `P13-GATE-015-BASELINE-FIRST-INCREMENT`, gives [`GateCommand`] a
//! real [`BaselinePolicy`] under the same user override, taking the second of `OD-GATE-015`'s
//! three concerns -- [`BaselinePolicy`] matches a [`nomos_contracts::Finding`] by `rule` and
//! `subject`, the same identity [`SuppressionPolicy`] already matches by, and a matched
//! finding cannot fail the build but still appears in [`GateRunResult::check_outcome`] and
//! [`GateRunResult::baselined_findings`], never silently. Deliberately narrower than
//! `BASELINE-*`'s full shape: no new-code/diff/identity detection distinguishes tolerated
//! debt from a reintroduced or genuinely new finding, and no scope beyond the named
//! `rule`/`subject` pairs an entry lists -- `BaselineDebt`'s own doc says why. `Run_Gate`
//! checks suppression before baseline, so a finding matched by both reports as suppressed;
//! the two lists do not overlap. No CLI flag or config file constructs a [`BaselineDebt`]
//! yet, the same absence [`SuppressionPolicy`]'s fifth increment already declined to fill.
//!
//! Its eighth increment, `P13-GATE-015-ADOPTION-FIRST-INCREMENT`, gives [`GateCommand`] a
//! real [`AdoptionPolicy`] under the same user override, taking the third and last of
//! `OD-GATE-015`'s three concerns -- deliberately a different addressing scheme than
//! [`SuppressionPolicy`] and [`BaselinePolicy`]: [`RuleCalibration`] matches by
//! [`nomos_contracts::RuleId`] alone, not `rule`/`subject`, because `ADOPT-CONFIG-*` frames
//! adoption as a layer above both -- "declared gates, phases and calibration policy" -- not
//! a third per-finding matcher of their shape. A calibrated rule cannot fail a build
//! regardless of which subject triggered it; `Run_Gate` checks calibration before
//! suppression and baseline, so a finding matched by more than one reports as calibrated,
//! and it still appears in [`GateRunResult::check_outcome`] and
//! [`GateRunResult::calibrated_findings`], never silently. Deliberately narrower than
//! `ADOPT-CONFIG-*`'s full corpus shape: no declared phases, thresholds or approvals, and no
//! separate consumer-owned configuration file -- [`RuleCalibration`]'s own doc says why. No
//! CLI flag or config file constructs one yet, the same absence [`SuppressionPolicy`]'s and
//! [`BaselinePolicy`]'s own first increments already declined to fill.
//!
//! Its ninth increment, `P14-GATE-016-COVERAGE-POLICY-FIRST-INCREMENT`, gives [`GateCommand`]
//! a real [`CoveragePolicy`] under `OD-GATE-016`'s decision -- unlike every policy before it,
//! this one is not addressed by `rule`/`subject` or by `RuleId` at all: it is a single
//! opt-in switch consulted once, after [`Disposition`] already reduced
//! [`GateRunResult::blocking_findings`]. Unset (`Default`), `Claim` still rides through
//! [`GateRunResult::check_outcome`] for information only, unchanged from every increment
//! before it. Set to [`CoveragePolicy::RequireCompleteness`], `Run_Gate` recomputes `Claim`
//! over the rule-and-scope-selected findings -- not the whole-run `claim` `check_outcome`
//! already carries, which `command.rules` and `command.scope` have not yet narrowed -- and
//! a disposition that would otherwise be [`GateRunOutcome::Passed`] is reported
//! [`GateRunOutcome::Indeterminate`] instead whenever that recomputed claim is incomplete.
//! A disposition that would otherwise be `Failed` is left untouched, for the reason
//! [`CoveragePolicy`]'s own doc gives. No CLI flag or config file constructs a non-default
//! one yet, the same absence every policy before it also declined to fill first.
//!
//! # What no increment is
//!
//! None implements `compare` -- that verb has no variant here at all, not a stub one, the
//! same "no invented shape ahead of a real body" this crate's own [`command`] module
//! documents. None touches declared phases, thresholds or approvals -- every other clause
//! `WF-001` names beyond suppression, baseline, rule calibration and coverage. `GatePlan`
//! still does not vary by [`GateCommand::root`], `scope`, `rules`, `suppressions`,
//! `baseline` or `adoption` -- it
//! reports the registry, not a walk, so no selection applies to it yet.

#![forbid(unsafe_code)]

mod adoption;
mod baseline;
mod command;
mod composition;
mod coverage;
mod explain;
mod outcome;
mod rule_selector;
mod run;
mod run_gate;
mod run_id;
mod scope_selector;
mod suppression;

#[cfg(test)]
mod tests;

pub use adoption::{AdoptionPolicy, RuleCalibration};
pub use baseline::{BaselineDebt, BaselinePolicy};
pub use command::GateCommand;
pub use composition::Registered;
pub use coverage::CoveragePolicy;
pub use explain::{Explain_Gate, Explanation, FindingQuery, GateExplainResult};
pub use outcome::{Disposition, GateOutcome, GatePlan, GateRunOutcome, GateRunResult};
pub use rule_selector::RuleSelector;
pub use run::Run;
pub use run_gate::Run_Gate;
pub use run_id::Fresh_Run_Id;
pub use scope_selector::ScopeSelector;
pub use suppression::{Suppression, SuppressionDisposition, SuppressionPolicy};
