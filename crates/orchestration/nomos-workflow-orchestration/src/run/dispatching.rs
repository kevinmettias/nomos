//! Everything one run needs besides the plan it is running.

use nomos_contracts::RunId;
use nomos_platform::{Environment, FileSystem, ProgramLauncher, Timestamp};
use nomos_workspace::BuildVariant;

use super::Platform;

/// Everything one run needs besides its plan, grouped so the functions that carry it
/// through the retry loop stay inside this workspace's four-value-parameter cap.
///
/// `read_clock` is a reader rather than a [`nomos_platform::Clock`] because the two entry
/// points differ only in whether they have one: [`super::Run_With_Clock`] supplies a
/// reader over the caller's own clock, [`super::Run_Unclocked`] supplies none, and a
/// generic clock parameter would force the clockless entry point to name a clock type it
/// does not have. `None` is what makes a bounded step report
/// [`crate::StepTiming::Unmeasured`] rather than a measurement nothing took.
pub(super) struct Dispatching<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    /// The real platform this run's steps dispatch through.
    pub(super) platform: &'a Platform<'a, Launcher, Fs, Env>,
    /// What the compiling binary was built as, for a check, correction or gate step.
    pub(super) variant: &'a BuildVariant,
    /// The run a gate step identifies its own execution by.
    pub(super) run: RunId,
    /// Where a bounded step's start and end are read from, or `None` when this run was
    /// given no clock to read.
    pub(super) read_clock: Option<&'a dyn Fn() -> Timestamp>,
}
