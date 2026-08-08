//! Running a program directly, with a timeout that actually terminates it.

use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::io::Read;
use std::process::Stdio;
use std::time::Instant;

/// How often to check whether a running process has finished.
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);

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
            .ok_or_else(|| "a command needs a program to run".to_owned())?;
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

        let mut child = builder
            .spawn()
            .map_err(|error| format!("could not start `{program}`: {error}"))?;

        let started = Instant::now();
        let outcome = loop
        {
            match child.try_wait()
            {
                Ok(Some(status)) =>
                {
                    break status.code().map_or(ExitOutcome::Terminated, |code| {
                        ExitOutcome::Exited { code }
                    });
                }
                Ok(None) =>
                {}
                Err(error) => return Err(format!("could not wait for `{program}`: {error}")),
            }

            if started.elapsed() >= command.timeout
            {
                // Kill and then reap. Skipping the wait leaves a zombie on Unix, and a
                // verification predicate that spawns one per timeout will exhaust the
                // process table of a machine running an agent fleet.
                let _ = child.kill();
                let _ = child.wait();
                break ExitOutcome::TimedOut;
            }

            std::thread::sleep(POLL_INTERVAL);
        };

        let mut stdout = String::new();
        let mut stderr = String::new();
        if let Some(mut handle) = child.stdout.take()
        {
            let _ = handle.read_to_string(&mut stdout);
        }
        if let Some(mut handle) = child.stderr.take()
        {
            let _ = handle.read_to_string(&mut stderr);
        }

        return Ok(ProcessOutput {
            outcome,
            stdout,
            stderr,
        });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
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
}
