//! Running something, without a shell.

use std::path::PathBuf;

/// A program to run, as an argument vector.
///
/// An argument **vector**, never a command string. The prototype stored verification
/// commands as strings and split them on whitespace with quote handling, which meant a
/// bespoke parser standing between what an author wrote and what ran — and a shared
/// file that several agents write became a place where one agent's quoting could change
/// what another agent's process executed.
///
/// Storing `argv` removes the parser and the whole class of defect with it. It also
/// removes shell interpolation, globbing and chaining, which a verification predicate
/// has no business doing: a predicate answers yes or no, and one that can also pipe,
/// redirect and expand is a small script nobody reviewed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command
{
    /// The program and its arguments. The first element is the program.
    pub argv: Vec<String>,
    /// Where to run it.
    pub working_directory: Option<PathBuf>,
    /// How long to allow before giving up.
    pub timeout: std::time::Duration,
}

impl Command
{
    /// Constructs a command from an argument vector.
    #[must_use]
    pub fn New(argv: Vec<String>, timeout: std::time::Duration) -> Self
    {
        return Self {
            argv,
            working_directory: None,
            timeout,
        };
    }

    /// The program to run, if the vector is not empty.
    #[must_use]
    pub fn Program(&self) -> Option<&str>
    {
        return self.argv.first().map(String::as_str);
    }
}

/// How a process ended.
///
/// Three outcomes, not an exit code and a bool. A process that was killed for exceeding
/// its timeout has not failed its predicate — nobody found out whether the predicate
/// holds — and reporting that as a non-zero exit would turn "we did not learn anything"
/// into "the check failed", which is the same conflation this whole system exists to
/// avoid one level up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitOutcome
{
    /// The process ran to completion with this exit code.
    Exited
    {
        /// The exit code.
        code: i32,
    },
    /// The process exceeded its timeout and was terminated.
    TimedOut,
    /// The process was killed by a signal or otherwise ended abnormally.
    Terminated,
}

impl ExitOutcome
{
    /// Whether the process ran to completion and reported success.
    ///
    /// [`ExitOutcome::TimedOut`] and [`ExitOutcome::Terminated`] are not successes and
    /// are not failures of what was being checked. They are the absence of a result.
    #[must_use]
    pub const fn Succeeded(self) -> bool
    {
        return matches!(self, Self::Exited { code: 0 });
    }

    /// Whether anything was actually learned about the thing being checked.
    #[must_use]
    pub const fn Produced_A_Verdict(self) -> bool
    {
        return matches!(self, Self::Exited { .. });
    }
}

/// What a process produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessOutput
{
    /// How it ended.
    pub outcome: ExitOutcome,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}

/// Runs a program directly, without shell mediation.
pub trait ProcessLauncher
{
    /// Runs the command to completion and captures its output.
    ///
    /// # Errors
    ///
    /// Returns a message describing why the process could not be started at all. A
    /// process that started and then failed is not an error here — that is an
    /// [`ExitOutcome`], and the distinction is between "we could not ask" and "we asked
    /// and the answer was no".
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    #[test]
    fn Test_Only_A_Zero_Exit_Should_Succeed()
    {
        assert!(ExitOutcome::Exited { code: 0 }.Succeeded());
        assert!(!ExitOutcome::Exited { code: 1 }.Succeeded());
        assert!(!ExitOutcome::TimedOut.Succeeded());
        assert!(!ExitOutcome::Terminated.Succeeded());
    }

    /// The distinction the enum exists for. A timeout must not be recorded as the
    /// predicate having been checked and found false.
    #[test]
    fn Test_A_Timeout_Should_Not_Count_As_A_Verdict()
    {
        assert!(!ExitOutcome::TimedOut.Produced_A_Verdict());
        assert!(!ExitOutcome::Terminated.Produced_A_Verdict());
        assert!(ExitOutcome::Exited { code: 1 }.Produced_A_Verdict());
    }

    #[test]
    fn Test_An_Empty_Argv_Should_Have_No_Program()
    {
        let empty = Command::New(Vec::new(), Duration::from_secs(1));

        assert_eq!(empty.Program(), None);
    }
}
