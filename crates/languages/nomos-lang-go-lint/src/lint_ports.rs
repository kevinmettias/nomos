//! [`LintPorts`], the three ports linting runs through.

use nomos_platform::{Environment, FileSystem, ProgramLauncher};

/// The three ports linting runs through: a filesystem to find each source's module, a launcher for
/// `go`, and the environment `go` is found and resolved in.
pub struct LintPorts<'ports, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    pub launcher: &'ports Launcher,
    pub filesystem: &'ports Fs,
    pub environment: &'ports Env,
}
