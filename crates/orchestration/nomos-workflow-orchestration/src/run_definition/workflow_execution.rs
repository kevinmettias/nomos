//! Everything a definition run needs besides the definition itself.

use nomos_contracts::RunId;
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_workspace::BuildVariant;

use super::Parallelism;
use crate::Platform;

/// Everything one definition run needs besides the definition, grouped into one value.
///
/// The same grouping [`Platform`] and [`crate::ClockedPlatform`] already are, applied once
/// more and for the identical reason: this workspace's `parameter-count` rule caps a
/// function at four value parameters, [`crate::Run`] already sits at that cap with a plan,
/// a platform, a variant and a run, and [`crate::Replay`] needs a fifth thing -- the
/// bound -- alongside a record and the definition offered against it.
///
/// No clock. A definition run is the clockless entry point, so a node declaring
/// `nomos_contracts::Timeout::Seconds` reports [`crate::StepTiming::Unmeasured`] rather
/// than a bound it honored. That is the same honesty [`crate::Run_Unclocked`] already
/// keeps -- a run that measured nothing must not answer as though it had -- and a clocked
/// definition run is not built here, because nothing this item claims needs one and a
/// second entry point nobody calls is a surface with no consumer.
pub struct WorkflowExecution<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    /// The real platform this run's nodes dispatch through.
    pub platform: &'a Platform<'a, Launcher, Fs, Env>,
    /// What the compiling binary was built as, for a check, correction or gate node.
    pub variant: &'a BuildVariant,
    /// The run a gate node identifies its own execution by.
    pub run: RunId,
    /// How many independent nodes one dispatch group may hold, and in what order this run
    /// visits one.
    pub parallelism: Parallelism,
}
