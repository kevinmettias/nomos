//! What a real `nomos gate run` produced, including the check facts behind it.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RunId;
use std::path::PathBuf;

use crate::GateRunOutcome;
use super::GateFindings;

/// What a real `nomos gate run` produced.
///
/// `check_outcome` is carried in full -- including `Claim`, for information only, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code -- so a
/// caller that wants the finer detail behind `disposition` does not have to re-walk or
/// re-judge anything to get it.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Every finding this run reduced, grouped by why it does or does not block.
    pub findings: GateFindings,
    /// The reduced verdict. When [`crate::GateCommand::phases`] declares a phase policy,
    /// this is [`crate::Phased_Disposition`]'s own answer rather than the flat
    /// [`crate::Disposition_Of_Findings`] every earlier increment computed alone -- a phase
    /// policy can turn a run that would otherwise fail into one that passes, when every
    /// blocking finding is named by some phase and no phase failed unapproved. `findings`
    /// still carries every finding this run reduced, in full, regardless of what any phase
    /// decided about them.
    pub disposition: GateRunOutcome,
}
