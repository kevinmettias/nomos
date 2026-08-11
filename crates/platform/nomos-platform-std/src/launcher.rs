//! Running a program directly, with a timeout that actually terminates it.

use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::io::Read;
use std::process::Stdio;
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

        let outcome = Waited(&mut child, program, command.timeout)?;
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

/// Waits for the child, and kills it if it outstays the timeout.
///
/// Kill and then reap. Skipping the wait leaves a zombie on Unix, and a verification
/// predicate that spawns one per timeout will exhaust the process table of a machine
/// running an agent fleet.
fn Waited(
    child: &mut std::process::Child,
    program: &str,
    timeout: std::time::Duration,
) -> Result<ExitOutcome, String>
{
    let started = Instant::now();

    loop
    {
        match child.try_wait()
        {
            Ok(Some(status)) =>
            {
                return Ok(status.code().map_or(ExitOutcome::Terminated, |code| {
                    return ExitOutcome::Exited { code };
                }));
            }
            Ok(None) =>
            {}
            Err(error) => return Err(format!("could not wait for `{program}`: {error}")),
        }

        if started.elapsed() >= timeout
        {
            let _ = child.kill();
            let _ = child.wait();

            return Ok(ExitOutcome::TimedOut);
        }

        std::thread::sleep(POLL_INTERVAL);
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
mod tests
{
    use super::*;
    use std::fmt::Write as _;
    use std::path::PathBuf;
    use std::time::Duration;

    /// A command that exits with the given code, on either platform family.
    fn Exit_With(code: i32) -> Command
    {
        let argv = if cfg!(windows)
        {
            vec![
                "cmd".to_owned(),
                "/C".to_owned(),
                format!("exit {code}"),
            ]
        }
        else
        {
            vec!["sh".to_owned(), "-c".to_owned(), format!("exit {code}")]
        };

        return Command::New(argv, Duration::from_secs(30));
    }

    #[test]
    fn Test_A_Successful_Program_Should_Report_A_Zero_Exit()
    {
        let output = StdProcessLauncher.Run(&Exit_With(0)).unwrap();

        assert_eq!(output.outcome, ExitOutcome::Exited { code: 0 });
        assert!(output.outcome.Succeeded());
    }

    /// A failing predicate must be distinguishable from one that could not be asked.
    #[test]
    fn Test_A_Failing_Program_Should_Report_Its_Exit_Code()
    {
        let output = StdProcessLauncher.Run(&Exit_With(3)).unwrap();

        assert_eq!(output.outcome, ExitOutcome::Exited { code: 3 });
        assert!(!output.outcome.Succeeded());
        assert!(
            output.outcome.Produced_A_Verdict(),
            "a non-zero exit is still an answer"
        );
    }

    /// A program that does not exist is an error, not a failed check. Reporting it as a
    /// non-zero exit would tell an author their code is wrong when the truth is that
    /// their tooling is missing.
    #[test]
    fn Test_A_Missing_Program_Should_Be_An_Error_Not_A_Failed_Check()
    {
        let missing = Command::New(
            vec!["nomos-no-such-program-exists".to_owned()],
            Duration::from_secs(5),
        );

        let result = StdProcessLauncher.Run(&missing);

        assert!(result.is_err(), "a missing program is not a verdict");
    }

    #[test]
    fn Test_An_Empty_Command_Should_Be_Refused()
    {
        let empty = Command::New(Vec::new(), Duration::from_secs(5));

        assert!(StdProcessLauncher.Run(&empty).is_err());
    }

    /// How many lines the loud fixture writes.
    ///
    /// Twenty thousand lines of thirty-three bytes is 660,000 — ten times what a pipe
    /// holds on this platform, and under a tenth of a second to emit. The volume is the
    /// whole point: a predicate that stays under the buffer never meets this defect,
    /// which is why four phases of work did not.
    const LOUD_LINES: usize = 20_000;

    /// The code the loud fixture exits with.
    ///
    /// Deliberately neither zero nor a code a launcher might invent. A launcher that
    /// lost the real code and defaulted to success would satisfy an assertion for zero.
    const LOUD_EXIT: i32 = 7;

    /// A command that writes several hundred kilobytes and then exits [`LOUD_EXIT`].
    ///
    /// Returns the text it should produce, so that a test can ask whether the transcript
    /// is the program's own output rather than whatever fitted.
    fn Loud(name: &str) -> (Command, PathBuf, String)
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-launcher-loud-{name}-{}.txt", std::process::id()));

        let mut text = String::new();
        for line in 0..LOUD_LINES
        {
            writeln!(text, "line {line:05} 012345678901234567890").expect("writes a line");
        }
        std::fs::write(&path, &text).expect("writes the fixture");

        let argv = Shout(&path.display().to_string());

        return (Command::New(argv, Duration::from_secs(10)), path, text);
    }

    /// An argv that prints a file and then exits loudly, in the shell of the host.
    fn Shout(shown: &str) -> Vec<String>
    {
        if cfg!(windows)
        {
            return vec![
                "cmd".to_owned(),
                "/C".to_owned(),
                format!("type {shown} & exit {LOUD_EXIT}"),
            ];
        }

        return vec![
            "sh".to_owned(),
            "-c".to_owned(),
            format!("cat {shown}; exit {LOUD_EXIT}"),
        ];
    }

    /// The defect this exists for.
    ///
    /// A predicate must be judged on its result and not on its volume. Against a
    /// launcher that reads the pipes only after the child has ended, the child blocks in
    /// `write` once the buffer is full, never exits, and is killed at the timeout — so a
    /// run that did find out is recorded as one where nobody found out.
    #[test]
    fn Test_A_Loud_Program_Should_Be_Judged_On_Its_Result_Not_Its_Volume()
    {
        let (command, path, _) = Loud("result");

        let output = StdProcessLauncher.Run(&command).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            output.outcome,
            ExitOutcome::Exited { code: LOUD_EXIT },
            "a program that answered in {:?} was recorded as never having answered",
            command.timeout
        );
        assert!(output.outcome.Produced_A_Verdict());
    }

    /// The control that stops the fix from trading one lie for another.
    ///
    /// Draining is only worth having if what it drained is the whole of what the program
    /// said. A launcher that kept the first bufferful and reported the exit code would
    /// pass the test above while recording a transcript that is a prefix which happened
    /// to fit — and the tail is where a failing run puts its reason.
    #[test]
    fn Test_A_Loud_Programs_Output_Should_Arrive_Whole()
    {
        let (command, path, expected) = Loud("whole");

        let output = StdProcessLauncher.Run(&command).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            output.stdout.len(),
            expected.len(),
            "the transcript is {} bytes of the {} the program wrote",
            output.stdout.len(),
            expected.len()
        );
        assert_eq!(output.stdout, expected, "the transcript is not what the program wrote");
    }

    /// The control that stops the fix from being a removed timeout.
    ///
    /// Every assertion above is satisfied by a launcher that simply waits forever, which
    /// is the one remedy this must not be.
    #[test]
    fn Test_A_Program_That_Exceeds_Its_Timeout_Should_Still_Time_Out()
    {
        // Run the slow program directly rather than under a shell. `timeout` on Windows
        // refuses a redirected stdin and this launcher always gives it one, and a shell
        // wrapper would put the sleeping process a generation away from the kill.
        let argv = if cfg!(windows)
        {
            vec![
                "ping".to_owned(),
                "-n".to_owned(),
                "8".to_owned(),
                "127.0.0.1".to_owned(),
            ]
        }
        else
        {
            vec!["sleep".to_owned(), "7".to_owned()]
        };

        let slow = Command::New(argv, Duration::from_secs(1));

        let started = Instant::now();
        let output = StdProcessLauncher.Run(&slow).unwrap();

        assert_eq!(output.outcome, ExitOutcome::TimedOut);
        assert!(!output.outcome.Produced_A_Verdict());
        assert!(
            started.elapsed() < Duration::from_secs(7),
            "the timeout did not terminate the program, it waited for it"
        );
    }

    /// A grandchild holding the pipe open must not hold the launcher open.
    ///
    /// Found while fixing the defect above rather than reported with it. The child exits
    /// and its own ends of the pipes close, but anything it handed to a background
    /// process stays open, so end of file never arrives. Reading until end of file —
    /// which is what the launcher did after the wait, and what a thread joined without a
    /// bound would do — waits for the grandchild instead of the child. Here the
    /// grandchild outlives the budget by an order of magnitude.
    #[test]
    fn Test_A_Grandchild_Holding_The_Pipe_Should_Not_Hold_The_Launcher()
    {
        let argv = if cfg!(windows)
        {
            vec![
                "cmd".to_owned(),
                "/C".to_owned(),
                "start /B ping -n 30 127.0.0.1 & echo parent-done".to_owned(),
            ]
        }
        else
        {
            vec![
                "sh".to_owned(),
                "-c".to_owned(),
                "sleep 29 & echo parent-done".to_owned(),
            ]
        };

        let orphaning = Command::New(argv, Duration::from_secs(20));

        let started = Instant::now();
        let output = StdProcessLauncher.Run(&orphaning).unwrap();
        let waited = started.elapsed();

        assert_eq!(
            output.outcome,
            ExitOutcome::Exited { code: 0 },
            "the child exited promptly and its own verdict is what was asked for"
        );
        assert!(
            waited < Duration::from_secs(20),
            "waited {waited:?} for a process that was never the one being judged"
        );
    }
}
