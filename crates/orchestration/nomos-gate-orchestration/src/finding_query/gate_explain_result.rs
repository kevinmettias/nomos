//! What a real `nomos gate explain` produced.

use super::Explanation;
use nomos_check_orchestration::CheckOutcome;
use std::path::PathBuf;

/// What a real `nomos gate explain` produced.
pub struct GateExplainResult
{
    /// The tree this judgment was over.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// The answer to `query`.
    pub explanation: Explanation,
}
