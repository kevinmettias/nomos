//! [`TypesPorts`], the three ports typing runs through.

use nomos_platform::{Environment, FileSystem, ProgramLauncher};

/// The three ports typing runs through: a filesystem to find each source's module and to write the
/// helper, a launcher for `go`, and the environment `go` and a temporary directory are found in.
pub struct TypesPorts<'ports, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    pub launcher: &'ports Launcher,
    pub filesystem: &'ports Fs,
    pub environment: &'ports Env,
}
