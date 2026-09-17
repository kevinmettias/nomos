//! The process launcher a dispatch accepts but does not choose.

use nomos_platform::ProgramLauncher;

/// The process launcher [`crate::Run_Agent_Execute`] and [`crate::Run_Agent_Judgment`] need
/// but do not choose -- grouped into its own value the same way `nomos_correction_orchestration::
/// CorrectionEnvironment` groups the platform ports its own seam needs, for the identical
/// reason: neither chooses a platform, only accepts the one its caller already did. One
/// field rather than that type's three: dispatching a `TaskEnvelope` to a backend touches
/// no build variant and no filesystem, only a process launcher.
pub struct AgentEnvironment<'a, Launcher: ProgramLauncher>
{
    pub launcher: &'a Launcher,
}
