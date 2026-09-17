//! Waiting for a spawned child until it exits, stalls, or outstays its timeout.

use nomos_platform::{Command, ExitOutcome};
use std::time::Instant;

use super::spawn::Streams;

/// Waits for the child, and kills it if it outstays its wall bound or goes silent for
/// its idle bound.
///
/// Kill and then reap. Skipping the wait leaves a zombie on Unix, and a verification
/// predicate that spawns one per timeout will exhaust the process table of a machine
/// running an agent fleet.
///
/// The idle bound is checked first on every poll, ahead of the wall bound, so that a
/// process which produced nothing at all is reported as [`ExitOutcome::Stalled`] rather
/// than [`ExitOutcome::TimedOut`] even in the case where both bounds are about to expire
/// together — which is exactly what happens when a caller never asked for a shorter idle
/// bound, since [`Command::From_String_Arguments`] starts the two equal. A caller that wants the two
/// distinguished for a genuinely silent-from-the-start process gets that distinction
/// for free; a caller that wants to catch a stall *before* the wall bound would otherwise
/// expire has to ask for a shorter idle bound with [`Command::With_Idle_Timeout`].
pub(super) fn Waited_For_Child(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    streams: &Streams<'_>,
) -> Result<ExitOutcome, String>
{
    return Polled_Until_Settled(child, program, command, streams);
}

/// Polls the child once per [`super::POLL_INTERVAL`] until it exits on its own, goes silent
/// past its idle bound, or outruns its wall bound.
///
/// This is the loop [`Waited_For_Child`] hands off to. Each pass asks [`Already_Exited`]
/// whether the child is already done, then [`Progressed_Since_Last_Poll`] whether either
/// bound has now run out.
fn Polled_Until_Settled(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    streams: &Streams<'_>,
) -> Result<ExitOutcome, String>
{
    let context = PollContext {
        command,
        streams,
        started: Instant::now(),
    };
    let mut progress = Progress {
        length: Combined_Length(streams),
        at: context.started,
    };

    return loop
    {
        if let Some(outcome) = Already_Exited(child, program)?
        {
            return Ok(outcome);
        }

        if let Some(outcome) = Progressed_Since_Last_Poll(child, program, &context, &mut progress)?
        {
            return Ok(outcome);
        }

        std::thread::sleep(super::POLL_INTERVAL);
    };
}

/// Whether the child has already exited, and with what outcome if so.
///
/// `None` means still running; the wait itself failing is the one case worth reporting
/// as an error rather than folding into either outcome.
fn Already_Exited(child: &mut std::process::Child, program: &str) -> Result<Option<ExitOutcome>, String>
{
    return match child.try_wait()
    {
        Ok(Some(status)) =>
        {
            return Ok(Some(status.code().map_or(ExitOutcome::Terminated, |code| {
                return ExitOutcome::Exited { code };
            })));
        }
        Ok(None) => return Ok(None),
        Err(error) => return Err(format!("could not wait for `{program}`: {error}")),
    }
}

/// What every poll of a still-running child needs, aside from the child itself and how
/// much progress has been seen so far — grouped so [`Progressed_Since_Last_Poll`] takes
/// one reference instead of three positional parameters.
struct PollContext<'a>
{
    command: &'a Command,
    streams: &'a Streams<'a>,
    started: Instant,
}

/// How much output a child had produced as of one poll, and when that amount was last
/// seen to grow.
///
/// The pair travels together because the idle clock ([`Elapsed::idle`]) is only ever
/// "time since this length last changed" — neither half means anything on its own.
#[derive(Clone, Copy)]
struct Progress
{
    length: usize,
    at: Instant,
}

/// Updates `progress` against what has arrived since the last poll, and reports whether
/// either bound has now run out.
fn Progressed_Since_Last_Poll(
    child: &mut std::process::Child,
    program: &str,
    context: &PollContext<'_>,
    progress: &mut Progress,
) -> Result<Option<ExitOutcome>, String>
{
    let current_len = Combined_Length(context.streams);
    *progress = Advanced_Progress(*progress, current_len);
    let elapsed = Elapsed_Since(progress, context.started);

    return Bound_Exceeded(child, program, context.command, elapsed);
}

/// Advances `progress` to `current_len` if the child produced more since the last poll,
/// resetting the idle clock; otherwise leaves it exactly as it was.
fn Advanced_Progress(progress: Progress, current_len: usize) -> Progress
{
    if current_len > progress.length
    {
        return Progress {
            length: current_len,
            at: Instant::now(),
        };
    }

    return progress;
}

/// How long a child has been silent, and how long it has run in total, as of one poll.
struct Elapsed
{
    idle: std::time::Duration,
    total: std::time::Duration,
}

/// Reads [`Elapsed`] off `progress` and `started` as of right now.
fn Elapsed_Since(progress: &Progress, started: Instant) -> Elapsed
{
    return Elapsed {
        idle: progress.at.elapsed(),
        total: started.elapsed(),
    };
}

/// Checks the idle bound and then the wall bound against `elapsed`, killing the child and
/// reporting which bound gave out first — idle before wall, for the reason
/// [`Waited_For_Child`] documents. `None` means neither bound has expired yet.
fn Bound_Exceeded(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    elapsed: Elapsed,
) -> Result<Option<ExitOutcome>, String>
{
    use super::kill::Killed_And_Reaped;

    if elapsed.idle >= command.idle_timeout
    {
        Killed_And_Reaped(child, program)?;

        return Ok(Some(ExitOutcome::Stalled {
            idle_elapsed: elapsed.idle,
        }));
    }

    if elapsed.total >= command.timeout
    {
        Killed_And_Reaped(child, program)?;

        return Ok(Some(ExitOutcome::TimedOut));
    }

    return Ok(None);
}

/// How many bytes have been captured on either stream so far, without copying them.
///
/// The sum is what "progress" means here: a process writing only to `stderr`, or only
/// to `stdout`, is still a process that is producing something, and the idle bound
/// exists to catch the process that is producing neither.
fn Combined_Length(streams: &Streams<'_>) -> usize
{
    use super::drain::Drain;

    let stderr_len = streams.stderr.map_or(0, Drain::Length);

    return streams.stdout.map_or(0, Drain::Length).saturating_add(stderr_len);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    /// The wall bound this case gives the child: long enough that a `cmd /C exit 0` finishes
    /// well inside it, so the wait ends on the child's own exit rather than on the bound.
    const WALL_BOUND_SECONDS: u64 = 5;

    #[test]
    fn Test_Waited_For_Child_Should_Report_A_Clean_Exit()
    {
        let mut child = Spawned_Exiting_With(0);
        let command = Command::From_String_Arguments(Vec::new(), Duration::from_secs(WALL_BOUND_SECONDS));
        let streams = Streams { stdout: None, stderr: None };

        let outcome = Waited_For_Child(&mut child, "nomos-platform-std-wait-test", &command, &streams)
            .expect("the child exits on its own well within the bound");

        assert_eq!(outcome, ExitOutcome::Exited { code: 0 });
    }

    fn Spawned_Exiting_With(code: i32) -> std::process::Child
    {
        let mut command = if cfg!(windows)
        {
            let mut command = std::process::Command::new("cmd");
            command.args(["/C", &format!("exit {code}")]);
            command
        }
        else
        {
            let mut command = std::process::Command::new("sh");
            command.args(["-c", &format!("exit {code}")]);
            command
        };

        return command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawns a quick-exiting child for the wait test");
    }
}
