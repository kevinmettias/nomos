//! Running a program directly, with a timeout that actually terminates it.

use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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

/// How much to take from a pipe at a time.
const CHUNK_SIZE: usize = 8_192;

/// A stream being read while the process writing it is still running.
///
/// This is a thread rather than a read after the wait because a pipe holds a fixed
/// number of bytes — 64 KiB as measured on this platform — and a child that fills one
/// blocks in `write` until somebody reads. Nothing did. A predicate loud enough to fill
/// the buffer therefore never exited, was killed at its timeout, and a run that had
/// answered was recorded as a run nobody got an answer from. The failing direction is
/// the worse one: failure detail is exactly what makes output large, so the louder the
/// failure the likelier it was reported as a timeout instead.
struct Drain
{
    /// What has been read so far.
    ///
    /// Shared with the reader rather than owned by it, so that a reader still blocked on
    /// a pipe somebody else is holding open does not take the output with it.
    collected: Arc<Mutex<Vec<u8>>>,
    /// Whether the reader reached end of file.
    finished: Arc<AtomicBool>,
}

/// Reads a stream to its end, keeping everything it yields.
///
/// A read error ends the drain exactly as end of file does. There is nothing useful to
/// report from here — the outcome belongs to the process, not to its pipe — and what was
/// read before the error is still worth keeping.
fn Drain_Into<R: Read>(source: &mut R, sink: &Mutex<Vec<u8>>)
{
    let mut chunk = [0_u8; CHUNK_SIZE];

    while let Ok(taken) = source.read(&mut chunk)
    {
        if taken == 0
        {
            break;
        }
        let Ok(mut buffer) = sink.lock()
        else
        {
            break;
        };
        let Some(slice) = chunk.get(..taken)
        else
        {
            break;
        };
        buffer.extend_from_slice(slice);
    }
}

impl Drain
{
    // rust-lifetime: allow: `std::thread::spawn` requires it. The reader is moved onto a
    // detached thread that outlives this call, so no borrow of the caller can reach it and
    // `'static` is the bound the standard library demands rather than one chosen here.
    /// Starts reading `source` on a thread of its own.
    fn Reading<R: Read + Send + 'static>(mut source: R) -> Self
    {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let finished = Arc::new(AtomicBool::new(false));
        let sink = Arc::clone(&collected);
        let reached_end = Arc::clone(&finished);

        std::thread::spawn(move || {
            Drain_Into(&mut source, &sink);
            // atomic-ordering: allow: pairs with the `Acquire` load in `Finished`. Every write
            // `Drain_Into` made into `collected` through the mutex happens before this store, so a
            // reader that observes `true` is guaranteed to observe the drained bytes as well. A
            // `Relaxed` store would publish the flag without the buffer behind it.
            reached_end.store(true, Ordering::Release);
        });

        return Self {
            collected,
            finished,
        };
    }

    /// Whether the reader reached end of file.
    fn Finished(&self) -> bool
    {
        // atomic-ordering: allow: pairs with the `Release` store in `Reading`. Reading `true` here
        // establishes happens-before against the draining thread, so the caller that then takes
        // `collected` sees the complete buffer rather than a prefix of it.
        return self.finished.load(Ordering::Acquire);
    }

    /// How many bytes have been read so far, without copying them.
    ///
    /// Cheap on purpose: this is polled once per [`POLL_INTERVAL`] for as long as a
    /// process runs, to notice whether it is still producing anything. Copying the whole
    /// buffer that often to answer a question only its length can answer would make the
    /// idle check itself the thing slowing a loud process down.
    fn Len(&self) -> usize
    {
        return self.collected.lock().map_or(0, |buffer| return buffer.len());
    }

    /// What has been read, as text.
    ///
    /// Lossy rather than strict, which is a change in what gets recorded. `read_to_string`
    /// refuses the entire stream on one byte that is not UTF-8 and the caller discarded
    /// the error, so a predicate that printed a stray byte had the whole of its output
    /// recorded as nothing at all — the same shape of loss this type exists to end.
    fn Text(&self) -> String
    {
        return self.collected.lock().map_or_else(
            |_| String::new(),
            |buffer| String::from_utf8_lossy(&buffer).into_owned(),
        );
    }
}

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
        let mut child = Spawned(command, program)?;

        // Both streams are read from the moment the child starts, and not after it ends.
        // A child cannot finish writing more than a pipeful unless somebody is taking it
        // away, so this is what makes the wait below a wait on the program rather than a
        // wait on its volume.
        let stdout = child.stdout.take().map(Drain::Reading);
        let stderr = child.stderr.take().map(Drain::Reading);

        let outcome = Waited(&mut child, program, command, stdout.as_ref(), stderr.as_ref())?;
        Settle(stdout.as_ref(), stderr.as_ref());

        return Ok(ProcessOutput {
            outcome,
            stdout: stdout.as_ref().map_or_else(String::new, Drain::Text),
            stderr: stderr.as_ref().map_or_else(String::new, Drain::Text),
        });
    }
}

/// The child, started with both its streams piped and nothing on its input.
///
/// `stdin` is null rather than inherited, because a predicate that stops to read from a
/// terminal nobody is at would hang until the timeout and report as slow work.
fn Spawned(command: &Command, program: &str) -> Result<std::process::Child, String>
{
    use std::process::Stdio;

    let arguments = command.argv.get(1..).unwrap_or_default();
    let mut builder = std::process::Command::new(program);
    builder
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(directory) = &command.working_directory
    {
        builder.current_dir(directory);
    }

    return builder
        .spawn()
        .map_err(|error| return format!("could not start `{program}`: {error}"));
}

/// How many bytes have been captured on either stream so far, without copying them.
///
/// The sum is what "progress" means here: a process writing only to `stderr`, or only
/// to `stdout`, is still a process that is producing something, and the idle bound
/// exists to catch the process that is producing neither.
fn Combined_Len(stdout: Option<&Drain>, stderr: Option<&Drain>) -> usize
{
    return stdout.map_or(0, Drain::Len).saturating_add(stderr.map_or(0, Drain::Len));
}

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
fn Waited(
    child: &mut std::process::Child,
    program: &str,
    command: &Command,
    stdout: Option<&Drain>,
    stderr: Option<&Drain>,
) -> Result<ExitOutcome, String>
{
    let started = Instant::now();
    let mut last_progress = started;
    let mut last_len = Combined_Len(stdout, stderr);

    let outcome = loop
    {
        match child.try_wait()
        {
            Ok(Some(status)) =>
            {
                break status.code().map_or(ExitOutcome::Terminated, |code| {
                    return ExitOutcome::Exited { code };
                });
            }
            Ok(None) =>
            {}
            Err(error) => return Err(format!("could not wait for `{program}`: {error}")),
        }

        let current_len = Combined_Len(stdout, stderr);
        if current_len > last_len
        {
            last_len = current_len;
            last_progress = Instant::now();
        }

        let idle_elapsed = last_progress.elapsed();
        if idle_elapsed >= command.idle_timeout
        {
            Stopped(child, program)?;

            break ExitOutcome::Stalled { idle_elapsed };
        }

        if started.elapsed() >= command.timeout
        {
            Stopped(child, program)?;

            break ExitOutcome::TimedOut;
        }

        std::thread::sleep(POLL_INTERVAL);
    };

    return Ok(outcome);
}

/// Kills a child that outran its budget, along with whatever it spawned, and reaps it.
///
/// A direct kill only ever reaches the one process this launcher started. `cargo test`
/// starts the compiled test binary as its own child, and killing `cargo` alone leaves
/// that binary running — which is the process that was actually hanging, still there
/// after the thing watching it believes the hang is over. [`Kill_Tree`] is what reaches
/// the whole tree instead of the one process at its root.
///
/// A child that ended between the last poll and this kill is already in the state the kill was
/// after, and the platforms disagree about how they say so: Unix reports success against a
/// not-yet-reaped child, Windows answers `InvalidInput`. Every other failure means a process
/// this function promised to stop is still running, and reporting the timeout would be a lie
/// about it.
///
/// A wait that fails leaves the zombie this exists to avoid, so it is reported rather than
/// dropped.
fn Stopped(child: &mut std::process::Child, program: &str) -> Result<(), String>
{
    if let Err(cause) = Kill_Tree(child)
        && cause.kind() != std::io::ErrorKind::InvalidInput
    {
        return Err(format!("`{program}` outran its timeout and could not be killed: {cause}"));
    }

    child
        .wait()
        .map_err(|cause| return format!("could not reap `{program}` after killing it: {cause}"))?;

    return Ok(());
}

/// Kills a child and every process it spawned, where the platform can be asked to.
///
/// `Child::kill` is `TerminateProcess` on Windows and `SIGKILL` on Unix, and both reach
/// only the one process named by the handle or the pid — neither walks down to what that
/// process started. `taskkill /T` does: given a pid it walks the process tree rooted
/// there and terminates every process in it, which is what "kill the child" has to mean
/// when the child is `cargo test` and the thing actually hanging is the test binary
/// `cargo` spawned underneath it.
///
/// `cfg!(windows)` rather than `#[cfg(windows)]`: this workspace's gate lints only the
/// Linux leg on the strength of there being no conditionally-compiled code for it to
/// miss, and a compile-time `#[cfg]` here would make that no longer true. The runtime
/// check keeps both branches ordinary, always-compiled Rust, so nothing about this
/// function is invisible to the leg that does not run it.
///
/// A `taskkill` that cannot run at all — wrong platform, not on `PATH`, the process
/// already gone — is not reported as this function's own failure. [`Child::kill`] below
/// is the fallback that actually matters for those cases, and it already carries its own
/// accounting for "the process was already gone" via the `InvalidInput` the caller
/// checks for.
fn Kill_Tree(child: &mut std::process::Child) -> std::io::Result<()>
{
    if cfg!(windows)
    {
        let tree_killed = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if matches!(tree_killed, Ok(status) if status.success())
        {
            return Ok(());
        }
    }

    return child.kill();
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
