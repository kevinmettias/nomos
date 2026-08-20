//! Waiting for a spawned child until it exits, stalls, or outstays its timeout.

use nomos_platform::{Command, ExitOutcome};
use std::time::Instant;

use super::drain::Drain;
use super::kill::Stopped;
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
/// bound, since [`Command::New`] starts the two equal. A caller that wants the two
/// distinguished for a genuinely silent-from-the-start process gets that distinction
/// for free; a caller that wants to catch a stall *before* the wall bound would otherwise
/// expire has to ask for a shorter idle bound with [`Command::With_Idle_Timeout`].
pub(super) fn Waited(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    streams: &Streams<'_>,
) -> Result<ExitOutcome, String>
{
    return Polled(child, program, command, streams);
}

/// Polls the child once per [`super::POLL_INTERVAL`] until it exits on its own, goes silent
/// past its idle bound, or outruns its wall bound.
///
/// This is the loop [`Waited`] hands off to. Each pass asks [`Exited`] whether the child
/// is already done, then [`Progressed`] whether either bound has now run out.
fn Polled(
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
        len: Combined_Len(streams),
        at: context.started,
    };

    loop
    {
        if let Some(outcome) = Exited(child, program)?
        {
            return Ok(outcome);
        }

        if let Some(outcome) = Progressed(child, program, &context, &mut progress)?
        {
            return Ok(outcome);
        }

        std::thread::sleep(super::POLL_INTERVAL);
    }
}

/// How many bytes have been captured on either stream so far, without copying them.
///
/// The sum is what "progress" means here: a process writing only to `stderr`, or only
/// to `stdout`, is still a process that is producing something, and the idle bound
/// exists to catch the process that is producing neither.
fn Combined_Len(streams: &Streams<'_>) -> usize
{
    let stderr_len = streams.stderr.map_or(0, Drain::Len);

    return streams.stdout.map_or(0, Drain::Len).saturating_add(stderr_len);
}

/// Whether the child has already exited, and with what outcome if so.
///
/// `None` means still running; the wait itself failing is the one case worth reporting
/// as an error rather than folding into either outcome.
fn Exited(child: &mut std::process::Child, program: &str) -> Result<Option<ExitOutcome>, String>
{
    match child.try_wait()
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
/// much progress has been seen so far — grouped so [`Progressed`] takes one reference
/// instead of three positional parameters.
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
    len: usize,
    at: Instant,
}

/// Advances `progress` to `current_len` if the child produced more since the last poll,
/// resetting the idle clock; otherwise leaves it exactly as it was.
fn Advanced(progress: Progress, current_len: usize) -> Progress
{
    if current_len > progress.len
    {
        return Progress {
            len: current_len,
            at: Instant::now(),
        };
    }

    return progress;
}

/// Updates `progress` against what has arrived since the last poll, and reports whether
/// either bound has now run out.
fn Progressed(
    child: &mut std::process::Child,
    program: &str,
    context: &PollContext<'_>,
    progress: &mut Progress,
) -> Result<Option<ExitOutcome>, String>
{
    let current_len = Combined_Len(context.streams);
    *progress = Advanced(*progress, current_len);
    let elapsed = Elapsed_Since(progress, context.started);

    return Bound_Exceeded(child, program, context.command, elapsed);
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
/// reporting which bound gave out first — idle before wall, for the reason [`Waited`]
/// documents. `None` means neither bound has expired yet.
fn Bound_Exceeded(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    elapsed: Elapsed,
) -> Result<Option<ExitOutcome>, String>
{
    if elapsed.idle >= command.idle_timeout
    {
        Stopped(child, program)?;

        return Ok(Some(ExitOutcome::Stalled {
            idle_elapsed: elapsed.idle,
        }));
    }

    if elapsed.total >= command.timeout
    {
        Stopped(child, program)?;

        return Ok(Some(ExitOutcome::TimedOut));
    }

    return Ok(None);
}
