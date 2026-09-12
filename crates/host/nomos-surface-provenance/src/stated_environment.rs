//! An environment a test states, rather than the one the test process is standing in.
//!
//! Separate from [`crate::fake_launcher`] because it stands in for a different port and
//! that file's own doc is about how a scripted launcher tells one `git` call from
//! another — a question this type has no part in.
//!
//! What it buys is specific: `--root` defaults to the working directory, and before the
//! `Environment` port existed that default was read from `std::env::current_dir` inside
//! `arguments.rs`. A test could not state it, so no test asserted it — the two tests that
//! parse a line without `--root` both check `since`, `until` and `crates` and say nothing
//! about `root`, because the only value they could have compared against was one they
//! would have had to read the same ambient way.

use nomos_platform::{DeterminismStrength, Environment, EnvironmentError, ReproducibilityScope};
use nomos_platform::{Strategy, TraceEquivalence};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// An environment standing in a directory the test names.
pub(crate) struct Stated
{
    working_directory: Option<PathBuf>,
}

impl Stated
{
    /// Standing in `working_directory`, with no variable set.
    pub(crate) fn At(working_directory: &Path) -> Self
    {
        return Self { working_directory: Some(working_directory.to_path_buf()) };
    }

    /// An environment whose working directory cannot be read — the process was started in
    /// a directory that has since been removed, or permission to resolve it was withdrawn.
    pub(crate) fn Standing_Nowhere() -> Self
    {
        return Self { working_directory: None };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Stated
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for Stated
{
    /// Nothing is set. This binary reads no environment variable — it needs the port for
    /// the working directory alone — so a fixture that answered otherwise would be
    /// describing a caller that does not exist.
    fn Variable(&self, _name: &str) -> Option<OsString>
    {
        return None;
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return self.working_directory.clone().ok_or(EnvironmentError::WorkingDirectoryUnreadable {
            cause: "this fixture is standing nowhere".to_owned(),
        });
    }
}
