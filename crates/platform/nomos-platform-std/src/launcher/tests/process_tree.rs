//! The kill reaching the whole process tree rather than only the direct child.
//!
//! Kept apart from the rest of `tests.rs` because its fixture is three processes deep and every
//! assertion here is about what is still running after the launcher returned, rather than about
//! what the launcher itself reported -- a different question, asked of a different shape.
//!
//! The `ping` fixtures are not a second copy of the ones beside them: `ping` either way, but
//! the command line nests it one generation down, which is what makes "was it killed" a
//! question about the tree instead of about the direct child.

use super::*;

use nomos_platform::ExitOutcome;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The bound the nested `ping` is given. Long enough that `ping` is still running when the
/// bound expires -- which is what makes the kill, rather than the program's own exit, the
/// thing being observed -- and short enough that the test does not wait it out.
const NESTED_PING_BOUND: Duration = Duration::from_secs(2);

/// How long the marker is watched after the launcher returns, before it is read a second time.
///
/// A window rather than a wait on an event, because what is being established is an absence:
/// nothing is still writing to the marker. Long enough that a surviving `ping`, which prints
/// about once a second, would have appended a line or two more.
const SETTLING_WINDOW: Duration = Duration::from_millis(1_500);

/// The bound the orphaning child is given, and the ceiling its wait is asserted to stay under.
/// The grandchild it leaves behind lives far longer, so a launcher that waited for the
/// grandchild rather than the child would blow past this.
const ORPHANING_BOUND: Duration = Duration::from_secs(20);

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
fn Test_A_Kill_Should_Reach_Every_Descendant_Not_Only_The_Direct_Child()
{
    if !cfg!(windows)
    {
        return;
    }

    let marker = Fresh_Marker_Path("tree");
    let nested = Nested_Ping_Command(&marker);

    let output = StdProgramLauncher.Run(&nested).expect("cmd runs on this host, and this test returns early elsewhere");
    Assert_Killed_Before_Completion(&output);

    let growth = Bytes_Written_Before_And_After_Settling(&marker);
    Remove_Fixture_File(&marker);

    Assert_Marker_Stopped_Growing(growth);
}

/// A fresh path for a marker file, with anything left over from a previous run cleared
/// first so a stale marker cannot be mistaken for one this run wrote.
fn Fresh_Marker_Path(name: &str) -> PathBuf
{
    let mut marker = std::env::temp_dir();
    marker.push(format!("nomos-launcher-{name}-{}.txt", std::process::id()));
    Remove_Fixture_File(&marker);

    return marker;
}

/// A command whose direct child is `cmd`, and whose actual work is `ping` running two
/// generations down — reproducing the shape `cargo test` and its compiled test binary
/// take, at a short enough bound that the test does not have to wait it out.
fn Nested_Ping_Command(marker: &Path) -> Command
{
    return Command::From_String_Arguments(
        vec![
            "cmd".to_owned(),
            "/C".to_owned(),
            format!("ping -n 30 127.0.0.1 > {}", marker.display()),
        ],
        NESTED_PING_BOUND,
    );
}

/// Verifies the wait ended because the launcher's bound expired, not because `ping`
/// itself finished — the distinction that makes the marker file's later behavior mean
/// anything.
fn Assert_Killed_Before_Completion(output: &ProgramOutput)
{
    assert!(
        !output.outcome.Has_A_Verdict(),
        "the wait must have ended at a bound, not at the program's own completion"
    );
}

/// What the marker held right after the launcher returned, and again after the settling
/// window. Two facts of one type, which is why they are named rather than returned as a pair.
struct MarkerGrowth
{
    right_after_kill: u64,
    settled: u64,
}

/// How many bytes the marker holds right after the launcher returns, and again after
/// giving anything still running a settling window to keep writing.
fn Bytes_Written_Before_And_After_Settling(marker: &Path) -> MarkerGrowth
{
    let right_after_kill = Bytes_Written(marker);
    std::thread::sleep(SETTLING_WINDOW); // flakiness: allow: proves an absence, no event to wait on
    let settled = Bytes_Written(marker);

    return MarkerGrowth { right_after_kill, settled };
}

/// Verifies the marker stopped growing the moment the launcher returned — proof that
/// `ping`, and not only the `cmd` above it, was actually killed.
fn Assert_Marker_Stopped_Growing(growth: MarkerGrowth)
{
    assert_eq!(
        growth.right_after_kill, growth.settled,
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
    let orphaning = Command::From_String_Arguments(A_Program_That_Orphans(), ORPHANING_BOUND);
    let started = Instant::now();
    let output = StdProgramLauncher.Run(&orphaning).expect("the orphaning fixture is a shell command this host runs");
    let waited = started.elapsed();

    assert_eq!(
        output.outcome,
        ExitOutcome::Exited { code: 0 },
        "the child exited promptly and its own verdict is what was asked for"
    );
    assert!(
        waited < ORPHANING_BOUND,
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
