//! What the launcher promises, exercised.

use super::*;
use nomos_platform::ExitOutcome;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// This file is itself reached through a `#[path]` attribute in `std_process_launcher.rs`, and a
// module loaded that way does not own a directory: a bare `mod process_tree;` here would be
// looked for beside `drain.rs`, not under `tests/`. The attribute says where the file is.
#[path = "tests/process_tree.rs"]
mod process_tree;

/// A wall bound no test in this file waits out. Every fixture given it ends -- or is ended --
/// for some other reason first, so a bound this long only ever shows that the bound which
/// actually fired was not this one.
const UNREACHED_WALL_BOUND: Duration = Duration::from_secs(30);

/// The exit code a failing program reports, chosen neither zero nor a code a launcher might
/// invent, so that an assertion on it distinguishes the real code from a default.
const A_FAILING_EXIT_CODE: i32 = 3;

/// A bound for a command the launcher refuses before it spawns anything. The bound is never
/// reached; it is here because `Command::New` takes one.
const REFUSAL_BOUND: Duration = Duration::from_secs(5);

/// The bound the loud fixture is given. Generous, because the point of that fixture is that
/// the launcher drains it rather than letting it block in `write`.
const LOUD_BOUND: Duration = Duration::from_secs(10);

/// How long a test tolerates before concluding the launcher waited for the program instead of
/// ending it. The bound under test is one second, so a wait under this is the bound firing
/// rather than the fixture finishing.
const ENDED_WELL_WITHIN: Duration = Duration::from_secs(7);

/// The idle bound given to a fixture whose wall bound is shorter, so that the wait is ended by
/// a clock the test deliberately did not let expire.
const IDLE_BOUND_LONGER_THAN_WALL: Duration = Duration::from_secs(5);

/// How long a test tolerates the silent fixture's one-second idle bound taking to end it. The
/// wall bound is [`UNREACHED_WALL_BOUND`], so a wait anywhere near this means the idle clock
/// was never what ended the wait.
const SILENT_END_BOUND: Duration = Duration::from_secs(10);

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

    return Command::New(argv, UNREACHED_WALL_BOUND);
}

#[test]
fn Test_A_Successful_Program_Should_Report_A_Zero_Exit()
{
    let output = StdProcessLauncher.Run(&Exit_With(0)).expect("exit 0 is a program the host shell runs to completion");

    assert_eq!(output.outcome, ExitOutcome::Exited { code: 0 });
    assert!(output.outcome.Is_Successful());
}

/// A failing predicate must be distinguishable from one that could not be asked.
#[test]
fn Test_A_Failing_Program_Should_Report_Its_Exit_Code()
{
    let output = StdProcessLauncher.Run(&Exit_With(A_FAILING_EXIT_CODE))
        .expect("exit 3 is a program the host shell runs to completion");

    assert_eq!(output.outcome, ExitOutcome::Exited { code: A_FAILING_EXIT_CODE });
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
        REFUSAL_BOUND,
    );

    let result = StdProcessLauncher.Run(&missing);

    assert!(result.is_err(), "a missing program is not a verdict");
}

#[test]
fn Test_An_Empty_Command_Should_Be_Refused()
{
    let empty = Command::New(Vec::new(), REFUSAL_BOUND);

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
        command: Command::New(argv, LOUD_BOUND),
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

    let output = StdProcessLauncher.Run(&command).expect("the loud fixture is a shell command this host runs");
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

    let output = StdProcessLauncher.Run(&command).expect("the loud fixture is a shell command this host runs");
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
/// is the one remedy this must not be. What this holds is that a program outstaying the
/// bounds it was given is *ended*, under the bounds [`Command::New`] hands out by default.
///
/// # Why the outcome is not asserted to be `TimedOut` specifically
///
/// `Command::New` starts the idle and wall bounds equal, and `wait`'s own doc states that
/// the idle bound is deliberately checked first, so a process which produced nothing at all
/// reports [`ExitOutcome::Stalled`] rather than [`ExitOutcome::TimedOut`] when the two
/// expire together. Whether this program produces anything is a property of the program,
/// and [`A_Slow_Program`] used not to be the same program on both hosts: Windows ran `ping`,
/// which prints a reply about once a second and so keeps resetting the idle clock, while
/// Unix ran `sleep`, which is silent for its whole life. Asserting `TimedOut` here made this
/// fail on Linux permanently and pass on Windows for a reason the test never stated (`P83`).
///
/// `P84` since made that fixture `ping` on both hosts, so the two would now agree — but this
/// stays tolerant of either outcome deliberately. What this control exists to hold is that a
/// program outstaying its bounds is *ended*, and pinning it to the one outcome a chatty
/// fixture happens to produce would make it fail again the day the fixture changes for a
/// reason having nothing to do with what it asserts. Both outcomes discharge it equally:
/// each is the launcher ending a program rather than waiting for it, which `Has_A_Verdict`
/// returning false is the type-level statement of. Progress itself is asserted, on both
/// hosts, by [`Test_A_Progressing_Program_Should_Report_Timed_Out_Rather_Than_Stalled`].
///
/// The wall bound specifically *is* exercised, deterministically and on both hosts, by
/// [`Test_A_Progressing_Program_Should_Report_Timed_Out_Rather_Than_Stalled`], which asks
/// for an idle bound longer than its wall bound instead of leaving the two equal. Nothing
/// is lost here by declining to assert it a second time, on a fixture that cannot deliver
/// it everywhere.
#[test]
fn Test_A_Program_That_Exceeds_Its_Timeout_Should_Still_Time_Out()
{
    let slow = Command::New(A_Slow_Program(), Duration::from_secs(1));
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&slow).expect("ping is present on every host this suite runs on");

    assert!(
        matches!(output.outcome, ExitOutcome::TimedOut | ExitOutcome::Stalled { .. }),
        "a program outstaying its bounds must be ended by one of them, got {:?}",
        output.outcome
    );
    assert!(!output.outcome.Has_A_Verdict());
    assert!(
        started.elapsed() < ENDED_WELL_WITHIN,
        "the timeout did not terminate the program, it waited for it"
    );
}

/// A program that runs for a while and *talks while it does*, on both hosts.
///
/// `ping` either way, differing only in the flag that spells a count, because the property
/// two tests read off this fixture is that it makes progress — and a fixture that was chatty
/// on one host and silent on the other let one of them claim a property it never exercised
/// there. It was `sleep 7` on Unix, which produces nothing for its whole life, so
/// [`Test_A_Progressing_Program_Should_Report_Timed_Out_Rather_Than_Stalled`] passed on Linux
/// only because its idle bound was longer than its wall bound; progress had never been
/// checked on any host but Windows (`P84`, the same asymmetry `P83` corrected for this
/// fixture's other consumer).
///
/// Run directly rather than under a shell. `timeout` on Windows refuses a redirected
/// stdin and this launcher always gives it one, and a shell wrapper would put the
/// long-running process a generation away from the kill.
fn A_Slow_Program() -> Vec<String>
{
    // Both print a line as they start and roughly one per second after, which is what keeps
    // resetting an idle clock. The count flag is the only real difference: `-n` on Windows,
    // `-c` everywhere else.
    let count_flag = if cfg!(windows) { "-n" } else { "-c" };

    return vec!["ping".to_owned(), count_flag.to_owned(), "8".to_owned(), "127.0.0.1".to_owned()];
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
    let silent = Command::New(A_Silent_Program(), UNREACHED_WALL_BOUND)
        .With_Idle_Timeout(Duration::from_secs(1));
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&silent).expect("the silent fixture is a shell command this host runs");

    assert!(
        matches!(output.outcome, ExitOutcome::Stalled { .. }),
        "a program producing nothing for its idle bound should report Stalled, not {:?}",
        output.outcome
    );
    assert!(!output.outcome.Has_A_Verdict());
    assert!(
        started.elapsed() < SILENT_END_BOUND,
        "the idle bound did not terminate the program — it waited nearly the full 30-second \
         wall bound instead"
    );
}

/// The control for the test above: a short idle bound must not turn a program that is
/// genuinely still working into a stall. Progress keeps resetting the idle clock, so this
/// must still end at the wall bound, exactly as it did before an idle bound existed.
///
/// # Why the premise is asserted and not assumed
///
/// The bounds here are deliberately ordered idle-longer-than-wall, so the wall bound would
/// have expired first whether this program spoke or not. That makes the outcome alone a weak
/// witness: a silent fixture reaches `TimedOut` too, by never being subject to the idle
/// bound at all, which is exactly how this test passed on Linux for a year without ever
/// exercising progress (`P84`). So the output is checked as well. If a host's `ping` cannot
/// run, this fails saying the fixture was silent rather than passing on the bound ordering —
/// a test must not claim a property its fixture cannot produce on the host it runs on.
#[test]
fn Test_A_Progressing_Program_Should_Report_Timed_Out_Rather_Than_Stalled()
{
    let progressing =
        Command::New(A_Slow_Program(), Duration::from_secs(1))
            .With_Idle_Timeout(IDLE_BOUND_LONGER_THAN_WALL);
    let started = Instant::now();
    let output = StdProcessLauncher.Run(&progressing).expect("ping is present on every host this suite runs on");

    assert!(
        !output.stdout.is_empty(),
        "this fixture produced nothing, so progress was never exercised and the outcome below \
         rests on the wall bound expiring first rather than on the idle clock being reset"
    );
    assert_eq!(
        output.outcome,
        ExitOutcome::TimedOut,
        "a program still producing output when the wall bound expired must not be reported \
         as stalled"
    );
    assert!(
        started.elapsed() < ENDED_WELL_WITHIN,
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
