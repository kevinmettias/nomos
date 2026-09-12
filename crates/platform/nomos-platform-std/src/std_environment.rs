//! The process's real environment.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{Environment, EnvironmentError};
use std::ffi::OsString;
use std::path::PathBuf;

/// Reads the environment the operating system actually started this process with.
///
/// The only implementation that should exist outside tests. A component takes an
/// [`Environment`] rather than calling [`std::env`] directly, which is what lets a test
/// decide what `CARGO` names and where a relative path resolves from — neither of which
/// was reachable from a test before, even where the process launcher was already injected.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdEnvironment;

/// Reaches the real machine, so it reproduces nothing and says so.
///
/// Worth stating explicitly rather than inheriting by analogy with [`crate::SystemClock`]:
/// the environment is *more* stable within a run than a clock is — two reads of `CARGO` a
/// second apart agree, where two clock reads do not — and that invites a stronger claim
/// than this type can honour. It cannot honour one because the value is chosen by whoever
/// invoked the process, so the same code on the same input reproduces only for a caller
/// who also reproduces the environment, which is exactly the thing this type does not
/// control.
impl Strategy for StdEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl Environment for StdEnvironment
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return std::env::var_os(name);
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return std::env::current_dir().map_err(|error| {
            return EnvironmentError::WorkingDirectoryUnreadable {
                cause: error.to_string(),
            };
        });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `CARGO` is set by cargo itself for anything it runs, so a test run by cargo is a
    /// context where this variable is known present — which is what makes this an
    /// assertion about reading rather than about the fixture.
    #[test]
    fn Test_Variable_Should_Read_A_Variable_The_Test_Runner_Itself_Sets()
    {
        assert!(StdEnvironment.Variable("CARGO").is_some());
    }

    #[test]
    fn Test_Variable_Should_Report_None_For_A_Name_Nothing_Sets()
    {
        assert!(StdEnvironment.Variable("NOMOS_A_VARIABLE_NOTHING_SETS").is_none());
    }

    /// The control for the test above it: a port method that always answered `None` would
    /// satisfy the absent case and nothing else, and a port method that always answered
    /// `Some` would satisfy the present case and nothing else. Neither passes both.
    #[test]
    fn Test_Working_Directory_Should_Be_An_Absolute_Existing_Path()
    {
        let directory = StdEnvironment
            .Working_Directory()
            .expect("a test process has a readable working directory");

        assert!(directory.is_absolute(), "{} is not absolute", directory.display());
        assert!(directory.exists(), "{} does not exist", directory.display());
    }
}
