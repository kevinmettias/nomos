//! Starting a child process with both output streams piped, and nothing on stdin.

use nomos_platform::Command;

use super::drain::Drain;

/// Starts `program` as a child and begins draining both its streams immediately.
///
/// Reading starts from the moment the child starts, and not after it ends. A child
/// cannot finish writing more than a pipeful unless somebody is taking it away, so this
/// is what makes the wait that follows a wait on the program rather than a wait on its
/// volume.
pub(super) fn Spawned_With_Streams(
    command: &Command,
    program: &str,
) -> Result<(std::process::Child, Option<Drain>, Option<Drain>), String>
{
    let mut child = Spawned_Program(command, program)?;
    let stdout = child.stdout.take().map(Drain::Reading);
    let stderr = child.stderr.take().map(Drain::Reading);

    return Ok((child, stdout, stderr));
}

/// The child's two captured streams, carried together.
///
/// The wait loop and the length check inside it both need "how much has either stream
/// produced" and neither ever judges one stream without the other, so the pair travels
/// as one value instead of two positional parameters a caller could transpose.
pub(super) struct Streams<'a>
{
    pub(super) stdout: Option<&'a Drain>,
    pub(super) stderr: Option<&'a Drain>,
}

/// The child, started with both its streams piped and nothing on its input.
///
/// `stdin` is null rather than inherited, because a predicate that stops to read from a
/// terminal nobody is at would hang until the timeout and report as slow work.
fn Spawned_Program(command: &Command, program: &str) -> Result<std::process::Child, String>
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

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    /// The time limit this fixture allows a program that echoes one word. Generous because
    /// the value is a bound on a hang, not a measurement of anything.
    const ECHO_BOUND_SECONDS: u64 = 5;

    /// How long a case waits for its drain before calling it a hang. The child has already
    /// been waited on, so this bound is never approached.
    const SETTLE_BOUND_SECONDS: u64 = 5;

    #[test]
    fn Test_Spawned_With_Streams_Should_Capture_The_Childs_Own_Output()
    {
        let command = Echo_Command("spawned-with-streams-hello");
        let program = command.Program().expect("the fixture names a program");

        let (mut child, stdout, stderr) = Spawned_With_Streams(&command, program).expect("starts the program");
        child.wait().expect("the child runs to completion");
        Awaited(stdout.as_ref());

        let text = stdout.as_ref().map(Drain::Text).unwrap_or_default();
        assert!(
            text.contains("spawned-with-streams-hello"),
            "the piped stdout must carry what the child printed, got {text:?}"
        );
        assert!(stderr.is_some(), "stderr must also be piped even when the child writes nothing to it");
    }

    fn Echo_Command(word: &str) -> Command
    {
        let argv = if cfg!(windows)
        {
            vec!["cmd".to_owned(), "/C".to_owned(), format!("echo {word}")]
        }
        else
        {
            vec!["sh".to_owned(), "-c".to_owned(), format!("echo {word}")]
        };

        return Command::New(argv, Duration::from_secs(ECHO_BOUND_SECONDS));
    }

    /// How often this test's own wait loop re-checks a drain, distinct from
    /// `std_process_launcher`'s real `POLL_INTERVAL`, which this module does not depend on.
    const TEST_POLL_INTERVAL: Duration = Duration::from_millis(10);

    /// Waits until a drain has finished, or panics -- the reader thread is asynchronous.
    fn Awaited(drain: Option<&Drain>)
    {
        let started = std::time::Instant::now();
        while drain.is_some_and(|drain| return !drain.Is_Finished())
        {
            assert!(
                started.elapsed() < Duration::from_secs(SETTLE_BOUND_SECONDS),
                "the drain never finished"
            );
            std::thread::sleep(TEST_POLL_INTERVAL); // flakiness: allow: same poll wait.rs's real Launcher uses
        }
    }
}
