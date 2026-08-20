//! Running a program directly, with a timeout that actually terminates it.

use nomos_platform::{Command, ProcessLauncher, ProcessOutput};
use std::time::Instant;

mod drain;
mod kill;
mod spawn;
mod wait;

use drain::Drain;
use spawn::{Spawned_With_Streams, Streams};
use wait::Waited;

/// How often to check whether a running process has finished.
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);

/// How long to keep reading after the child itself has ended.
///
/// Bounded rather than unbounded, and that is the whole of the reason it exists. Once
/// the child has ended its own ends of the pipes are closed, so end of file arrives at
/// once and this never elapses. What does not close is a pipe the child handed to a
/// background process before exiting: that grandchild can hold it open for as long as it
/// likes, and reading to end of file then waits for a process that was never the one
/// being judged. Measured at 29 seconds against a child that exited immediately.
///
/// So the trade is deliberate: a grandchild's trailing output can be cut off here, and
/// the verdict of the process actually under test is never held hostage to it.
const DRAIN_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

/// Runs programs as direct child processes.
///
/// No shell, ever. The program is `argv[0]` and the arguments are the rest, so there is
/// nothing to quote, nothing to expand, and no way for a string written into a shared
/// file to become a command somebody else's process runs.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdProcessLauncher;

impl ProcessLauncher for StdProcessLauncher
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        let program = command
            .Program()
            .ok_or_else(|| return "a command needs a program to run".to_owned())?;
        let (mut child, stdout, stderr) = Spawned_With_Streams(command, program)?;
        let streams = Streams {
            stdout: stdout.as_ref(),
            stderr: stderr.as_ref(),
        };

        let outcome = Waited(&mut child, program, command, &streams)?;
        Settle(streams.stdout, streams.stderr);

        return Ok(ProcessOutput {
            outcome,
            stdout: stdout.as_ref().map_or_else(String::new, Drain::Text),
            stderr: stderr.as_ref().map_or_else(String::new, Drain::Text),
        });
    }
}

/// Gives the readers a bounded moment to finish what is left in the pipes.
///
/// The child has ended, so they are draining rather than waiting on a process. See
/// `DRAIN_GRACE` for why this is bounded and not a join.
fn Settle(stdout: Option<&Drain>, stderr: Option<&Drain>)
{
    let settled = Instant::now();

    while settled.elapsed() < DRAIN_GRACE
    {
        if stdout.is_none_or(Drain::Finished) && stderr.is_none_or(Drain::Finished)
        {
            break;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests;
