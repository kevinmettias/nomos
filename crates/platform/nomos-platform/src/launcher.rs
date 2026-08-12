//! Running something, without a shell.

mod command;
mod exit_outcome;
mod process_output;

pub use command::Command;
pub use exit_outcome::ExitOutcome;
pub use process_output::ProcessOutput;


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

