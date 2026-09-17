//! Everything a finished process left behind.

use crate::ExitOutcome;

/// What a process produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramOutput
{
    /// How it ended.
    pub outcome: ExitOutcome,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}
