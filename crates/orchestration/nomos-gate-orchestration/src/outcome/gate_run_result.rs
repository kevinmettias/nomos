//! What a real `nomos gate run` produced, including the check facts behind it.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
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
    /// The tree this run judged.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// Exactly the findings for which `Finding::Can_Fail_A_Build` is true. Empty whenever
    /// `disposition` is not [`GateRunOutcome::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// The reduced verdict.
    pub disposition: GateRunOutcome,
}
