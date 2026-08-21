//! What a real `nomos gate run` produced, including the check facts behind it.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RunId};
use std::path::PathBuf;

use crate::GateRunOutcome;

/// What a real `nomos gate run` produced.
///
/// `check_outcome` is carried in full -- including `Claim`, for information only, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code -- so a
/// caller that wants the finer detail behind `disposition` does not have to re-walk or
/// re-judge anything to get it.
pub struct GateRunResult
{
    /// The identity of this execution -- `OD-WORKFLOW-001`'s first real consumer for
    /// `RunId`. Supplied by the caller to [`crate::Run_Gate`], not derived from anything
    /// else in this struct: two runs over the same `root` with the same findings are still
    /// two different executions.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// Exactly the findings for which `Finding::Can_Fail_A_Build` is true and no
    /// calibration, `Suppression` or baseline debt matched. Empty whenever `disposition` is
    /// not [`GateRunOutcome::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true but an `AdoptionPolicy`
    /// calibration matched their rule, so they could not fail the build -- carried rather
    /// than dropped, the same reason `suppressed_findings` and `baselined_findings` both
    /// are. Checked before `suppressed_findings` and `baselined_findings`, since calibration
    /// is a coarser, rule-wide override; a finding matched by more than one reports as
    /// calibrated, not counted twice.
    pub calibrated_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true, no calibration matched, but a
    /// `Suppression` matched, so they could not fail the build -- carried rather than
    /// dropped, because a suppressed finding that disappears from the answer is
    /// indistinguishable from one that was never found, and the corpus's own `SUP-EVID-*`
    /// requirement is that a suppressed state stays visible rather than collapsing into
    /// silence.
    pub suppressed_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true, no calibration or
    /// `Suppression` matched, but a `BaselineDebt` did, so they could not fail the build
    /// either -- carried for the same reason `suppressed_findings` is, and disjoint from it
    /// and from `calibrated_findings`: a finding matched by more than one is reported once,
    /// under the earliest of calibration, suppression, then baseline.
    pub baselined_findings: Vec<Finding>,
    /// The reduced verdict.
    pub disposition: GateRunOutcome,
}
