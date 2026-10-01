//! What a `nomos gate` verb was asked for, independent of how it was spelled.
//!
//! `ARC-ROADMAP-001` names four eventual verbs -- `plan`, `run`, `explain`, `compare` -- and
//! all four have real computations behind them now. `explain` needs one more input
//! `plan`/`run` do not, `crate::FindingQuery`, which travels as `Explain_Gate`'s own
//! separate parameter rather than a field here: every verb shares `GateCommand`, `explain`
//! alone also needs to name which finding, and folding that into this struct would make
//! every other verb carry a field it never reads, the same
//! `root`/`scope`/`rules`/`suppressions`/`baseline` asymmetry this struct already has for
//! `Plan`. This stays a struct rather than an enum for the same reason it always has: the
//! verbs share every field of it, so there is nothing for an enum to gain.
//!
//! # `compare` is real, and reachable from a terminal
//!
//! This doc said `compare` was "simply absent, not stubbed" for longer than that was true,
//! which is worth stating plainly because the claim was wrong in both directions a reader
//! could check. `crate::Compare_Gate_Runs` (`gate_compare.rs`, which cites this file as the
//! doc it falsified) takes two already-produced `crate::GateRunResult`s and reports which
//! findings moved between `crate::GateFindings`' own buckets. And it is not library-only:
//! `nomos gate compare` is a shipped verb with its own required `--against` flag and its
//! own paragraph in the gate verb's printed usage, recognized by `nomos-cli`'s
//! `gate/parsing.rs`, dispatched through its `Invocation::Compare` to `gate.rs`'s
//! `Compare_Verb`, and rendered by `gate/report.rs`'s `Render_Compare`.
//!
//! The restraint this paragraph used to describe was vindicated by that body rather than
//! overturned by it, which is the part of the old text worth keeping: **`compare` added no
//! field to this struct.** It needs two whole `GateCommand`s, one per tree, and
//! `Invocation::Compare` carries exactly that -- not one command holding a second root that
//! every other verb would then ignore. A speculative `against: Option<PathBuf>` here, added
//! ahead of the real body, would have been the wrong shape and would have had to come back
//! out. That is what declining to invent an argument surface ahead of a second real case
//! (`OD-PACKAGE-006`, `OD-RULES-005`, `OD-RULES-006`) is for.

use crate::{
    AdoptionPolicy, BaselinePolicy, CoveragePolicy, EvidenceFloor, GatePhase, PhaseApproval, RuleSelector, ScopeSelector,
    SuppressionPolicy,
};
use nomos_model_package::ModelExecutionProfile;
use std::path::PathBuf;

/// What to plan, run or explain a gate over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GateCommand
{
    /// The tree a gate would run against.
    ///
    /// Accepted and carried, not yet read by [`crate::GatePlan`]: it does not vary by root,
    /// because no scope selection composes it yet. `Run_Gate` does read it -- the tree it
    /// judges. It is here because every verb needs it and
    /// `nomos_check_orchestration::CheckCommand` already establishes `root` as this
    /// workspace's one settled way to name a tree.
    pub root: PathBuf,
    /// Which files under `root` a real run judges. Read by [`crate::Run_Gate`], not by
    /// [`crate::GatePlan`] -- the same asymmetry `root` already has, for the same reason:
    /// `Plan` reports the registry, not a walk, so nothing about a file selection applies
    /// to it yet.
    pub scope: ScopeSelector,
    /// Which rules' findings count toward a real run's disposition. Read by
    /// [`crate::Run_Gate`] only, the same asymmetry as `scope`.
    pub rules: RuleSelector,
    /// Which findings a real run must not let fail the build, despite `Finding::
    /// Can_Fail_A_Build`. Read by [`crate::Run_Gate`] only, the same asymmetry as `scope`
    /// and `rules`. Left at its default, [`crate::Run_Gate`] fills it in from the
    /// `nomos-gate.json` under `root` -- see `crate::policy::gate_policy_file` for the file's
    /// shape, why a caller's own value wins over it, and which rules a path-authored entry
    /// does not reach.
    pub suppressions: SuppressionPolicy,
    /// Existing debt a real run must not let fail the build either, checked after
    /// `suppressions` so a finding matched by both reports as suppressed. Read by
    /// [`crate::Run_Gate`] only, the same asymmetry as `scope`, `rules` and `suppressions`.
    /// Left at its default, [`crate::Run_Gate`] fills it in from the `nomos-gate.json` under
    /// `root`, the same way `suppressions` is.
    pub baseline: BaselinePolicy,
    /// Rules a real run must treat as advisory rather than blocking, checked before
    /// `suppressions` and `baseline` since it is a coarser, rule-wide override rather than a
    /// per-finding one. Read by [`crate::Run_Gate`] only, the same asymmetry as `scope`,
    /// `rules`, `suppressions` and `baseline`. Left at its default, [`crate::Run_Gate`] fills
    /// it in from the `nomos-gate.json` under `root`. A calibration matches by rule alone, so
    /// unlike `suppressions` and `baseline` it reaches every rule a file can name.
    pub adoption: AdoptionPolicy,
    /// Whether coverage debt over the rule-and-scope-selected findings should affect a real
    /// run's disposition, beyond the information-only `Claim` `check_outcome` already
    /// carries. Read by [`crate::Run_Gate`] only, the same asymmetry as `scope`, `rules`,
    /// `suppressions`, `baseline` and `adoption`. Left at [`CoveragePolicy::Unset`],
    /// [`crate::Run_Gate`] fills it in from the `nomos-gate.json` under `root`.
    pub coverage: CoveragePolicy,
    /// The lowest class of evidence a finding must carry before this run lets it block --
    /// `OD-GATE-034`'s floor, read in the partition beside `Finding::Can_Fail_A_Build`'s own
    /// two conditions rather than inside it. Read by [`crate::Run_Gate`] and
    /// [`crate::Explain_Gate`] only, the same asymmetry as `scope`, `rules`, `suppressions`,
    /// `baseline`, `adoption` and `coverage`. Left at [`EvidenceFloor::Unset`],
    /// [`crate::Run_Gate`] fills it in from the `nomos-gate.json` under `root`, and a run
    /// under neither source judges exactly as it did before this field existed.
    pub evidence_floor: EvidenceFloor,
    /// `MODEL-ROUTE-001`'s declared reference: what an agent-assisted operation running
    /// under this command should use, when one is selected. Read by nothing yet -- now the
    /// only field of this struct in that state, since `suppressions`, `baseline`, `adoption`
    /// and `coverage` all gained a declared source -- because no registered rule yields
    /// `nomos_contracts::Applicability::AgentRequired` today, so there is no real
    /// operation for a selected profile to activate. `None` is not a smaller case of
    /// this field; it is `MODEL-ROUTE-012`'s own first clause: a gate stays valid when no
    /// model is selected.
    pub model: Option<ModelExecutionProfile>,
    /// This run's own declared phases -- `WF-001`'s "required phases ... thresholds ...
    /// blocking behavior," judged in the order given over the findings `suppressions`,
    /// `baseline` and `adoption` have already reduced down to still-blocking. Read by
    /// [`crate::Run_Gate`] only, the same asymmetry as every field above. Empty is "no phase
    /// policy," the same behavior every existing caller and CI's own `gate run --root .`
    /// already have. Left empty, [`crate::Run_Gate`] fills it in from the `nomos-gate.json`
    /// under `root`, the same way `suppressions` is -- see `crate::policy::gate_policy_file`'s
    /// `declared_phases` for the file's shape and the refusals it owes an author.
    pub phases: Vec<GatePhase>,
    /// Approvals that let a phase named in `phases` pass despite exceeding its own
    /// threshold. Read by [`crate::Run_Gate`] only, the same asymmetry as `phases`.
    ///
    /// Resolved from the same source `phases` was, never mixed with it: an approval names the
    /// phase it covers, so a caller's phases paired with a file's approvals would let an
    /// approval address a stage its own source never declared. A command stating `phases`
    /// therefore states its own approvals too, including none.
    pub approvals: Vec<PhaseApproval>,
}
