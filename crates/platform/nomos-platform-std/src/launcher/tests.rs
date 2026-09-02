//! What the launcher promises, exercised.

use super::*;
use nomos_platform::ExitOutcome;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
    assert!(output.outcome.Is_Successful());
}

/// A failing predicate must be distinguishable from one that could not be asked.
#[test]
fn Test_A_Failing_Program_Should_Report_Its_Exit_Code()
{
    let output = StdProcessLauncher.Run(&Exit_With(3)).unwrap();

    assert_eq!(output.outcome, ExitOutcome::Exited { code: 3 });
    assert!(!output.outcome.Is_Successful());
    assert!(
        output.outcome.Has_A_Verdict(),
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

    let error = StdProcessLauncher.Run(&empty).expect_err("an empty argv names no program to run");

    assert_eq!(
        error, "a command needs a program to run",
        "the error must name the empty argv as the reason, not any of `Run`'s other failure modes"
    );
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
fn Loud(name: &str) -> LoudFixture
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

    return LoudFixture {
        command: Command::New(argv, Duration::from_secs(10)),
        path,
        text,
    };
}

/// Removes a fixture file, tolerating the one failure that is not one.
///
/// A path that is already absent is the state this asks for, so `NotFound` is success.
/// Anything else is said out loud rather than discarded: these fixtures are several
/// hundred kilobytes each, and a teardown that quietly cannot delete leaves one per run
/// in the temporary directory with nothing anywhere saying so.
fn Cleared(path: &Path)
{
    if let Err(cause) = std::fs::remove_file(path)
        && cause.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!("{} could not be cleared: {cause}", path.display());
    }
}

/// The loud fixture: the command to run, the file it prints, and what it prints.
///
/// Named rather than a triple. At three members a caller is counting positions, and
/// the position of the fixture path and the position of its contents are two facts
/// nobody should have to hold in their head.
struct LoudFixture
{
    command: Command,
    path: PathBuf,
    text: String,
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
    let LoudFixture { command, path, .. } = Loud("result");

    let output = StdProcessLauncher.Run(&command).unwrap();
    Cleared(&path);

    assert_eq!(
        output.outcome,
        ExitOutcome::Exited { code: LOUD_EXIT },
        "a program that answered in {:?} was recorded as never having answered",
        command.timeout
    );
    assert!(output.outcome.Has_A_Verdict());
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
    let LoudFixture {
        command,
        path,
        text: expected,
    } = Loud("whole");

    let output = StdProcessLauncher.Run(&command).unwrap();
    Cleared(&path);

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
    let slow = Command::New(A_Slow_Program(), Duration::from_secs(1));
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&slow).unwrap();

    assert_eq!(output.outcome, ExitOutcome::TimedOut);
    assert!(!output.outcome.Has_A_Verdict());
    assert!(
        started.elapsed() < Duration::from_secs(7),
        "the timeout did not terminate the program, it waited for it"
    );
}

/// Run directly rather than under a shell. `timeout` on Windows refuses a redirected
/// stdin and this launcher always gives it one, and a shell wrapper would put the
/// sleeping process a generation away from the kill.
fn A_Slow_Program() -> Vec<String>
{
    if cfg!(windows)
    {
        return vec![
            "ping".to_owned(),
            "-n".to_owned(),
            "8".to_owned(),
            "127.0.0.1".to_owned(),
        ];
    }

    return vec!["sleep".to_owned(), "7".to_owned()];
}

/// The distinction `P11-EXEC-IDLE` exists for: a child that produces nothing at all for
/// its idle bound is a different fact from a child that is still working when the wall
/// bound expires, and the two must not collapse into the same value.
///
/// `With_Idle_Timeout` gives this command an idle bound shorter than its wall bound, so
/// the idle bound is what ends the wait — the control below is what proves the wall
/// bound alone would not have.
#[test]
fn Test_A_Silent_Program_Should_Report_Stalled_Rather_Than_Timed_Out()
{
    let silent = Command::New(A_Silent_Program(), Duration::from_secs(30))
        .With_Idle_Timeout(Duration::from_secs(1));
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&silent).unwrap();

    assert!(
        matches!(output.outcome, ExitOutcome::Stalled { .. }),
        "a program producing nothing for its idle bound should report Stalled, not {:?}",
        output.outcome
    );
    assert!(!output.outcome.Has_A_Verdict());
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the idle bound did not terminate the program — it waited nearly the full 30-second \
         wall bound instead"
    );
}

/// The control for the test above: a short idle bound must not turn a program that is
/// genuinely still working into a stall. Progress keeps resetting the idle clock, so this
/// must still end at the wall bound, exactly as it did before an idle bound existed.
#[test]
fn Test_A_Progressing_Program_Should_Report_Timed_Out_Rather_Than_Stalled()
{
    let progressing =
        Command::New(A_Slow_Program(), Duration::from_secs(1)).With_Idle_Timeout(Duration::from_secs(5));
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&progressing).unwrap();

    assert_eq!(
        output.outcome,
        ExitOutcome::TimedOut,
        "a program still producing output when the wall bound expired must not be reported \
         as stalled"
    );
    assert!(
        started.elapsed() < Duration::from_secs(7),
        "the wall bound did not terminate the program, it waited for it"
    );
}

/// A program that runs for a while and produces nothing on either stream.
///
/// Redirected inside its own shell rather than left to inherit stdio, so what this
/// launcher's pipes see is genuinely nothing — not merely "nothing yet", the way
/// [`A_Slow_Program`]'s own first line is.
fn A_Silent_Program() -> Vec<String>
{
    if cfg!(windows)
    {
        return vec![
            "cmd".to_owned(),
            "/C".to_owned(),
            "ping -n 30 127.0.0.1 >nul 2>&1".to_owned(),
        ];
    }

    return vec!["sleep".to_owned(), "30".to_owned()];
}

/// A kill that reaches only the direct child is not the kill `cargo test` needs: the
/// hanging process is usually the compiled test binary a step above `cargo` spawned, and
/// Windows does not cascade a terminated process's own kill to what it started. `cmd`
/// running `ping` as its own child reproduces that shape at two processes deep — this
/// launcher's direct child is `cmd`, and `ping` is what `cmd` itself started and waited
/// on, exactly as `cargo` starts and waits on its test binary.
///
/// `ping`'s output is redirected to a marker file rather than captured, so what is being
/// asked is not "did the launcher see more output" but "did anything in the tree keep
/// running after the launcher returned" — a question a kill that only reached `cmd` would
/// answer wrong, because `ping` would still be there appending to the file on its own
/// schedule.
///
/// Windows only. There is nothing platform-specific about the claim, but proving it
/// without `taskkill` would need a mechanism this item's territory does not build; see
/// `docs/records/OD-PLATFORM-001` for the scope this leaves open.
#[test]
fn Test_A_Kill_Should_Reach_The_Whole_Process_Tree_Not_Only_The_Direct_Child()
{
    if !cfg!(windows)
    {
        return;
    }

    let marker = Fresh_Marker_Path("tree");
    let nested = Nested_Ping_Command(&marker);

    let output = StdProcessLauncher.Run(&nested).unwrap();
    Assert_Killed_Before_Completion(&output);

    let (right_after_kill, settled) = Bytes_Written_Before_And_After_Settling(&marker);
    Cleared(&marker);

    Assert_Marker_Stopped_Growing(right_after_kill, settled);
}

/// A fresh path for a marker file, with anything left over from a previous run cleared
/// first so a stale marker cannot be mistaken for one this run wrote.
fn Fresh_Marker_Path(name: &str) -> PathBuf
{
    let mut marker = std::env::temp_dir();
    marker.push(format!("nomos-launcher-{name}-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&marker);

    return marker;
}

/// A command whose direct child is `cmd`, and whose actual work is `ping` running two
/// generations down — reproducing the shape `cargo test` and its compiled test binary
/// take, at a short enough bound that the test does not have to wait it out.
fn Nested_Ping_Command(marker: &Path) -> Command
{
    return Command::New(
        vec![
            "cmd".to_owned(),
            "/C".to_owned(),
            format!("ping -n 30 127.0.0.1 > {}", marker.display()),
        ],
        Duration::from_secs(2),
    );
}

/// Verifies the wait ended because the launcher's bound expired, not because `ping`
/// itself finished — the distinction that makes the marker file's later behavior mean
/// anything.
fn Assert_Killed_Before_Completion(output: &ProcessOutput)
{
    assert!(
        !output.outcome.Has_A_Verdict(),
        "the wait must have ended at a bound, not at the program's own completion"
    );
}

/// How many bytes the marker holds right after the launcher returns, and again after
/// giving anything still running a settling window to keep writing.
fn Bytes_Written_Before_And_After_Settling(marker: &Path) -> (u64, u64)
{
    let right_after_kill = Bytes_Written(marker);
    std::thread::sleep(Duration::from_millis(1_500)); // flakiness: allow: proves an absence, no event to wait on
    let settled = Bytes_Written(marker);

    return (right_after_kill, settled);
}

/// Verifies the marker stopped growing the moment the launcher returned — proof that
/// `ping`, and not only the `cmd` above it, was actually killed.
fn Assert_Marker_Stopped_Growing(right_after_kill: u64, settled: u64)
{
    assert_eq!(
        right_after_kill, settled,
        "the marker file kept growing after the launcher returned, so `ping` was still \
         running under a `cmd` this launcher believed it had already killed"
    );
}

/// How many bytes a file holds, or zero if it cannot be read — a marker file that a
/// killed tree never got to create is itself evidence the tree is dead.
fn Bytes_Written(path: &Path) -> u64
{
    return std::fs::metadata(path).map_or(0, |metadata| return metadata.len());
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
    let orphaning = Command::New(A_Program_That_Orphans(), Duration::from_secs(20));
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

/// A shell that hands a pipe to a background process and then exits, so the grandchild
/// outlives the budget by an order of magnitude.
fn A_Program_That_Orphans() -> Vec<String>
{
    if cfg!(windows)
    {
        return vec![
            "cmd".to_owned(),
            "/C".to_owned(),
            "start /B ping -n 30 127.0.0.1 & echo parent-done".to_owned(),
        ];
    }

    return vec![
        "sh".to_owned(),
        "-c".to_owned(),
        "sleep 29 & echo parent-done".to_owned(),
    ];
}
