//! Running something, without a shell.

use nomos_contracts::Strategy;

mod command;
mod exit_outcome;
mod process_output;

pub use command::Command;
pub use exit_outcome::ExitOutcome;
pub use process_output::ProcessOutput;

/// Runs a program directly, without shell mediation.
/// # What an implementor promises
///
/// The supertrait is [`nomos_contracts::Strategy`], so every implementor states its
/// determinism triple. This is the seam where that question is sharpest and where it had
/// no answer: the twenty-nine types in this workspace that declared a triple were rules,
/// providers and formats, and not one of them was a port -- while the implementations
/// that actually cross the machine boundary, and the doubles that stand in for them,
/// declared nothing. The real one promises nothing and says so; a double built from fixed
/// data reproduces and says that. A caller reading `S::STRENGTH` can tell them apart
/// without knowing either type.
pub trait ProcessLauncher: Strategy
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

