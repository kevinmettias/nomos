//! The two ports a subprocess-backed materialization runs through.

use nomos_platform::{Environment, ProgramLauncher};

/// The two ports a subprocess-backed materialization needs, grouped so each
/// `Materialize_*` beside it stays inside this workspace's own `parameter-count` limit --
/// the same reason `MaterializationEnvironment` groups its own five.
pub struct Subprocess<'a, Launcher: ProgramLauncher, Env: Environment>
{
    /// What the provider's subprocess runs through.
    pub launcher: &'a Launcher,
    /// Where the provider reads `CARGO` from, rather than from this process's own state.
    pub environment: &'a Env,
}
