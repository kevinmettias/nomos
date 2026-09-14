//! Zone: Application Service — the seam for the first-class Gate object `ARC-ROADMAP-001` names, starting with
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
//! [`Disposition_Of_Findings`]: `OD-GATE-014` and `OD-GATE-015` both name the same trigger for the
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
//! before [`Disposition_Of_Findings`] reduces them -- at this increment, a real selection of what can
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
//! [`GateFindings::suppressed_findings`], never silently. No CLI flag or config file
//! constructs a [`crate::Suppression`] yet -- inventing an authoring surface before a real
//! caller needs one would repeat the mistake `OD-GATE-015` already declined, so this
//! increment is the type and its consultation only.
//!
//! Its sixth increment, `P13-GATE-EXPLAIN-FIRST-INCREMENT`, gives `explain` its first real
//! body: [`Explain_Gate`] judges a tree exactly as [`Run_Gate`] does (through a shared
//! [`gate_environment::Judged_Sources`] helper, so the two do not duplicate the walk/empty/unreadable
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
//! [`GateFindings::baselined_findings`], never silently. Deliberately narrower than
//! `BASELINE-*`'s full shape: no new-code/diff/identity detection distinguishes tolerated
//! debt from a reintroduced or genuinely new finding, and no scope beyond the named
//! `rule`/`subject` pairs an entry lists -- `BaselineDebt`'s own doc says why. `Run_Gate`
//! checks suppression before baseline, so a finding matched by both reports as suppressed;
//! the two lists do not overlap. A declared file constructs one as of the tenth increment
//! below; until it, nothing outside a test did.
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
//! [`GateFindings::calibrated_findings`], never silently. Deliberately narrower than
//! `ADOPT-CONFIG-*`'s full corpus shape: no declared phases, thresholds or approvals, and no
//! separate consumer-owned configuration file -- [`RuleCalibration`]'s own doc says why.
//! A declared file constructs one as of the tenth increment below.
//!
//! Its ninth increment, `P14-GATE-016-COVERAGE-POLICY-FIRST-INCREMENT`, gives [`GateCommand`]
//! a real [`CoveragePolicy`] under `OD-GATE-016`'s decision -- unlike every policy before it,
//! this one is not addressed by `rule`/`subject` or by `RuleId` at all: it is a single
//! opt-in switch consulted once, after [`Disposition_Of_Findings`] already reduced
//! [`GateFindings::blocking_findings`]. Unset (`Default`), `Claim` still rides through
//! [`GateRunResult::check_outcome`] for information only, unchanged from every increment
//! before it. Set to [`CoveragePolicy::RequireCompleteness`], `Run_Gate` recomputes `Claim`
//! over the rule-and-scope-selected findings -- not the whole-run `claim` `check_outcome`
//! already carries, which `command.rules` and `command.scope` have not yet narrowed -- and
//! a disposition that would otherwise be [`GateRunOutcome::Passed`] is reported
//! [`GateRunOutcome::Indeterminate`] instead whenever that recomputed claim is incomplete.
//! A disposition that would otherwise be `Failed` is left untouched, for the reason
//! [`CoveragePolicy`]'s own doc gives. A declared file constructs a non-default one as of
//! the tenth increment below.
//!
//! Its tenth increment, `P40-GATE-POLICY-AUTHORING-3`, gives all four of those policies the
//! authoring surface the four increments above each declined to build first. A run resolves
//! `nomos-gate.json` under [`GateCommand::root`], through the `FileSystem` port [`Run_Gate`]
//! already carries, so the CLI, `nomos-api`, the transport and MCP each get it without a
//! fifth copy of the format -- the duplication `OD-GATE-011` names as a defect class. An
//! absent file resolves to every default, which is today's behavior for every existing
//! caller and for CI's own `gate run --root .`; a policy a caller built in code still wins
//! over the file, so a test that pins one is never silently overridden by a working
//! directory. A file that exists and cannot be parsed refuses: the check still runs and
//! [`GateRunResult::check_outcome`] still carries it in full, but the disposition is
//! [`GateRunOutcome::Indeterminate`], because a build that passed there would be passing
//! under policy nobody authored.
//!
//! Entries name a path, not a `SubjectId`, since that identity is a digest nobody can write
//! by hand -- `nomos_model::Subject_Of_Path` computes it, the same function every real walk
//! already uses. That reaches only findings a rule addresses by the file's own subject:
//! measured directly, `no-single-line-function-bodies` and
//! `todo-format-is-todo-name-description-ticket` are reachable and `completeness-mirror` and
//! `single-letter-names` are not. `crate::policy::gate_policy_file`'s own doc states that
//! limit and why it is not papered over with a digest field.
//!
//! Its eleventh increment, `P40-GATE-PHASES-APPROVALS-5`, gives [`GateCommand`] real
//! [`GatePhase`]s and [`PhaseApproval`]s -- `WF-001`'s last three clauses, "required phases
//! ... thresholds ... approvals," that every increment above deliberately left unbuilt
//! ([`RuleCalibration`]'s own doc names them absent by name). Unlike every policy before it,
//! a phase does not remove a finding from [`GateFindings::blocking_findings`]; it groups
//! whole rules into an ordered, named stage judged over that list, each with its own
//! [`PhaseThreshold`], and [`Evaluated_Phases`] stops at the first phase that fails
//! unapproved -- later phases are reported [`PhaseDisposition::Skipped`] rather than judged,
//! `WF-001`'s "blocking behavior" read literally. [`Phased_Disposition`] is the seam back
//! into [`GateRunResult::disposition`]: a run that would otherwise fail can still pass when
//! every blocking finding is named by some declared phase and no phase failed unapproved,
//! and stays failed when a blocking finding belongs to no phase at all -- a phase policy
//! only ever adds a way to still pass, never a silent way to stop blocking. Deliberately
//! narrower than `WF-001`'s full shape: no CLI flag or config file constructs a [`GatePhase`]
//! or [`PhaseApproval`] yet, the same "type and its consultation only" restraint
//! `P13-GATE-015-SUPPRESSION-FIRST-INCREMENT` above already held, and [`GateRunResult`]
//! itself carries no per-phase detail -- a caller that needs to see which phase did what
//! calls [`Evaluated_Phases`] directly over [`GateRunResult::findings`].
//!
//! Its twelfth increment, `P41-GATE-PLAN-IS-A-PLAN-3`, gives [`crate::run::Run`] (the
//! `plan` verb) its first real narrowing: it reads [`GateCommand::rules`] and filters
//! [`Registered`]'s offers by it, the same [`RuleSelector::Is_Included`] test
//! [`crate::Run_Gate`] already applies to a real run's findings. Two `plan` invocations
//! differing only in `rules` now produce different [`GatePlan`]s, closing the "registry
//! introspection wearing the name of a plan" gap this record's own earlier text named.
//! `root` and `scope` remain unread: `Plan` still reports the registry rather than a walk,
//! so neither has a file to narrow against -- see [`crate::gate_plan::GatePlan`]'s own doc.
//!
//! # What no increment is
//!
//! `compare` is real as a library verb: [`Compare_Gate_Runs`] takes two already-produced
//! [`GateRunResult`]s and reports each finding's [`FindingDisposition`] change as a
//! [`DispositionChange`] in a [`GateCompareResult`] -- see [`crate::gate_compare`]'s own
//! doc for why a disposition diff, not a raw finding diff. It is not CLI-wired: no flag or
//! config file reaches it from a `nomos gate` invocation, and `nomos-cli`'s own gate-command
//! parser still refuses a `compare` subcommand outright, the same "no invented shape ahead
//! of a real body" this crate's own [`command`] module documents for the wiring, if not the
//! verb itself. `GatePlan`
//! still does not vary by [`GateCommand::root`] or `scope` -- it
//! reports the registry, not a walk, so neither selection applies to it yet.

#![forbid(unsafe_code)]

mod admissibility;
mod composition;
mod rule_composition_error;
mod finding_query;
mod gate_command;
mod gate_compare;
mod gate_environment;
mod gate_phase;
mod gate_plan;
mod policy;
mod run;
mod run_id;

#[cfg(test)]
mod tests;

pub use rule_composition_error::RuleCompositionError;
pub use admissibility::{Admissibility, Admits, Admits_Under, DependedCrate, DependingCrate, ARCHITECTURE_DECLARATION_FILE};
pub use composition::Registered;
pub use finding_query::{Explain_Gate, Explanation, FindingQuery, GateExplainResult};
pub use gate_command::GateCommand;
pub use gate_compare::{Compare_Gate_Runs, DispositionChange, FindingDisposition, GateCompareResult};
pub use gate_environment::{GateEnvironment, Run_Gate};
pub use gate_phase::{Evaluated_Phases, GatePhase, PhaseApproval, PhaseDisposition, PhaseOutcome, PhaseThreshold, Phased_Disposition};
pub use gate_plan::{Disposition_Of_Findings, GateFindings, GateOutcome, GatePlan, GateRunOutcome, GateRunResult};
pub use policy::{
    AdoptionPolicy, BaselineDebt, BaselinePolicy, CoveragePolicy, RuleCalibration, RuleSelector, ScopeSelector,
    Suppression, SuppressionDisposition, SuppressionPolicy, SuppressionReason, SuppressionStatus,
};
pub use run::Run;
pub use run_id::Fresh_Run_Id;
