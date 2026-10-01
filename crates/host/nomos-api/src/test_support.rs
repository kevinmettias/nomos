//! Test-only fixtures shared by more than one response module's own test suite --
//! `crate::spec` and `crate::response` today. Declared `#[cfg(test)]` by this
//! crate's own `lib.rs`, so nothing here compiles into a real build, the same discipline
//! `crate::work::tests_support` already keeps for its own verbs.
//!
//! Every fixture here hands its failure back rather than unwrapping its own steps. A fixture
//! is not the code under test: a scratch directory the machine will not let it create, or a
//! staged edit it cannot write, says nothing about the under-test code, and only the caller
//! knows which claim it was trying to prove when the step failed.
//!
//! # The one bound every gate judgment a test makes is admitted through
//!
//! [`Judged_Run`] and [`Judged_Compare`] are how `crate::response::gate_run_response`'s and
//! `crate::response::gate_compare_response`'s tests run a gate, and no test calls either
//! handler any other way. A gate run over a tree with a Cargo manifest loads that tree's
//! resolved crate graph, with a real sysroot, through `ra_ap_hir` -- once for each of the two
//! compiler-backed rule families, one after the other -- and libtest runs every test in this
//! crate as a thread of one process, one per core, so every such test used to begin loading at
//! the same moment. Nothing in libtest knows that a few of this crate's tests each hold a
//! compiler database. These two routes are the place that does: a judgment waits here for a
//! permit, and every other test runs exactly as libtest would have run it.

use crate::{GateCompareResponse, GateRunResponse, Handle_Gate_Compare, Handle_Gate_Run};
use nomos_gate_orchestration::GateCommand;
use nomos_test_permits::JudgmentPermits;
use serde::Serialize;
use std::cell::Cell;
use std::path::{Path, PathBuf};

/// The caller's own subsystem (`"spec-commit"`, `"gate-explain"`, and so on) -- the half of a
/// scratch path's name a module fixes rather than varies.
///
/// A type of its own rather than a second `&str`, which is what made
/// `Unique_Scratch_Directory("commit", "spec-commit")` a call the compiler would have
/// accepted and nobody would have noticed.
pub(crate) struct Area(pub(crate) &'static str);

/// A scratch directory of this test's own -- never a real shared directory a live session
/// writes to concurrently. `area` names the caller's own subsystem (`"spec-commit"`,
/// `"gate-explain"`, and so on) so two areas naming the same `label` still land in different
/// places; a call-local counter on top of that, since several fixtures in one area share one
/// `label`, and the default test runner's threads would otherwise race on one directory a
/// bare pid gave them. `process::id()` alone tells two runs of the whole suite apart; it says
/// nothing about two calls inside one.
///
/// # Errors
///
/// Returns whatever [`std::fs::create_dir_all`] refuses -- a temp directory that is not
/// writable, or a file already sitting where this call's own name computes to.
pub(crate) fn Unique_Scratch_Directory(area: Area, label: &str) -> Result<PathBuf, std::io::Error>
{
    // scope: allow this test-only counter has no owner beyond disambiguating calls within one
    // process; a bare pid does not distinguish two calls in the same test run.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    // atomic-ordering: allow: only used to give two calls in this process different numbers;
    // nothing else synchronizes on it or reads memory ordered by this counter.
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let directory = std::env::temp_dir().join(format!("nomos-api-{}-{label}-{}-{unique}", area.0, std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;

    return Ok(directory);
}

/// The directory name a [`Probe_Tree`] is created under -- the half of that fixture's own
/// naming that varies from probe to probe, `area` being the half its caller fixes.
///
/// A type of its own rather than a second `&str`, the same reason [`Area`] is one: a call
/// reading `Probe_Tree(area, name, policy)` would take two transposable strings beside the
/// one input that is not a name, and the compiler would raise nothing.
pub(crate) struct TreeName<'a>(pub(crate) &'a str);

/// A one-crate tree with real source and a declared `nomos-gate.json`, so a gate run over it
/// reaches `Judged` and resolves a policy of its own -- the two things an empty directory
/// cannot do.
///
/// `area` and `name` name the scratch directory and nothing else; the tree itself is fixed
/// here, and `policy` is the one thing a caller varies, because the policy is what a gate run
/// reads. Built by `crate::response::gate_compare_response` and
/// `crate::response::gate_run_response`, which wrote byte-identical trees under their own
/// `<prefix>-{name}-{pid}` names before this existed.
///
/// # Errors
///
/// Returns whatever [`Unique_Scratch_Directory`] or the three writes below refuse -- a temp
/// directory that is not writable, or a name already taken.
pub(crate) fn Probe_Tree(area: Area, name: TreeName, policy: &str) -> Result<PathBuf, std::io::Error>
{
    let root = Unique_Scratch_Directory(area, name.0)?;
    std::fs::create_dir_all(root.join("src"))?;
    let manifest = "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    std::fs::write(root.join("Cargo.toml"), manifest)?;
    std::fs::write(root.join("src").join("lib.rs"), "pub fn thing() -> i32 { return 1; }\n")?;
    std::fs::write(root.join("nomos-gate.json"), policy)?;

    return Ok(root);
}

/// How many gate judgments this test process may be making at once.
///
/// Three, from measurements and a share. Measured on 2026-09-27 at `a9f8feec`, on the 31.8 GB,
/// 32-thread machine this repository is developed on, one test at a time:
///
/// - a run over this crate's own directory, whose manifest resolves this whole workspace's crate
///   graph, took 207 s and peaked at 1,949 MB committed. That is two loads of about 100 s on one
///   core each, one per compiler-backed family, falling back to about 250 MB between them;
/// - a comparison of that directory with itself took 421 s, four such loads one after another,
///   and peaked at 1,957 MB -- no more than one run, because its judgments never overlap;
/// - a run over a [`Probe_Tree`], a one-crate manifest and the sysroot, took 28 s and peaked at
///   358 MB.
///
/// So a permit admits at most one database of about 1.9 GB at a time, and three come to about
/// 5.9 GB. That is under a fifth of the machine, the share `nomos-gate-orchestration`'s
/// `LIVE_REPOSITORY_JUDGMENTS` and `nomos-check-orchestration`'s `LIVE_COMPILER_DATABASES` take
/// for the same reason: a development machine that is also running an editor, a language server
/// and other sessions' builds. The count is of databases, not tests. Twelve tests build them,
/// and between them they make ten judgments of this crate's directory -- four runs and three
/// comparisons of two -- and seven of a probe tree. A probe-tree judgment takes a permit at the
/// larger judgment's price, and so do the runs of the two tests whose roots have no manifest
/// and load no database at all, so that one rule covers every route a test takes to a gate.
///
/// Unbounded, measured the same day at default parallelism, all twelve began at once. The test
/// process peaked at 13,490 MB committed and a 13,065 MB working set, the machine fell to 205 MB
/// available, and 176 of 176 passed in 822 s of test time. With this bound, measured the same
/// day from a cold target directory at default parallelism, 178 of 178 passed in 860 s of test
/// time. The test process peaked at 5,613 MB committed and a 5,489 MB working set, and the
/// machine never fell below 5,051 MB available, with two other sessions' test processes running
/// beside it.
///
/// The cost is wall time, and it is small. Those seventeen judgments are about 2,290 s of serial
/// work, so three at a time need at least about 760 s, and each comparison holds its permit for
/// about 420 s. Measured, the bound cost 38 s against the unbounded run, which held more than
/// twice the memory to save them.
///
/// Reopen this if one load's peak grows, which a third compiler-backed family, a change in how
/// either family loads the crate graph, or a larger dependency graph for this crate would do.
/// Reopen it if enough judgments are added that a third of their serial time no longer fits the
/// budget a predicate over this suite declares, or if the suite has to run on a machine with
/// less memory than this one.
const LIVE_GATE_JUDGMENTS: usize = 3;

/// The one bound every gate judgment a test in this crate makes is admitted through.
static PERMITS: JudgmentPermits = JudgmentPermits::New(LIVE_GATE_JUDGMENTS);

thread_local! {
    /// The permits a judgment made on this thread waits for in place of [`PERMITS`], once a
    /// routing test below has substituted a bound of its own for this one thread.
    static SUBSTITUTED: Cell<Option<&'static JudgmentPermits>> = const { Cell::new(None) };
}

/// The permits a judgment made on this thread is admitted through: [`PERMITS`], unless a
/// routing test has substituted its own for this thread alone.
fn Admitting() -> &'static JudgmentPermits
{
    return SUBSTITUTED.get().unwrap_or(&PERMITS);
}

/// [`Handle_Gate_Run`] over `command`, once this suite's bound admits it.
///
/// The handler is called unchanged, so a test driving this still drives the path a caller
/// takes; what is decided here is only when it starts.
pub(crate) fn Judged_Run(command: &GateCommand) -> GateRunResponse
{
    return Admitting().Holding(|| return Handle_Gate_Run(command));
}

/// [`Handle_Gate_Compare`] over `baseline` and `candidate`, once this suite's bound admits it.
///
/// One permit for both judgments, because the handler makes them one after the other and each
/// drops its databases before the next begins: a comparison holds no more at once than one run
/// does, only for twice as long.
pub(crate) fn Judged_Compare(baseline: &GateCommand, candidate: &GateCommand) -> GateCompareResponse
{
    return Admitting().Holding(|| return Handle_Gate_Compare(baseline, candidate));
}

/// Asserts that `response` serializes, round-trips through `serde_json`, and carries
/// `expected_outcome` under the `"outcome"` field every `#[serde(tag = "outcome", ...)]`
/// response in this crate tags itself with.
///
/// # Errors
///
/// Returns whatever [`serde_json`] refuses while serializing `response` or parsing that text
/// back. A mismatched outcome is an assertion rather than an error, because a wrong tag is
/// precisely what this function was called to report.
pub(crate) fn Assert_Round_Trips_As_Json<Response: Serialize>(
    response: &Response,
    expected_outcome: &str,
) -> Result<(), serde_json::Error>
{
    let json = serde_json::to_string(response)?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(
        parsed.get("outcome"),
        Some(&serde_json::Value::String(expected_outcome.to_owned())),
        "{json}"
    );

    return Ok(());
}

/// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
/// through this crate's own `Handle_Spec_Markdown`, so this needs no corpus and touches no
/// file this repository tracks), writes it to `staged.md` under `into`, and hands back the
/// path it was written to. Shared by every `Handle_Spec_*` test fixture that needs a real
/// staged edit to preview or commit.
///
/// # Errors
///
/// [`std::io::ErrorKind::NotFound`] when `id` names no record this binary carries, carrying
/// the store's own refusal text -- every id this fixture is called with names one, so that
/// arm means the call site asked for a record that does not exist rather than that a runtime
/// condition went wrong. And whatever [`std::fs::write`] refuses when the staged text cannot
/// be written under `into`.
pub(crate) fn Staged_Heading_Rename(id: &str, into: &Path) -> Result<PathBuf, std::io::Error>
{
    let request = nomos_spec_orchestration::RecordRequest { id: id.to_owned(), revision: None };

    let markdown = match crate::spec::Handle_Spec_Markdown(&request)
    {
        crate::spec::MarkdownResponse::Resolved { markdown, .. } => markdown,
        crate::spec::MarkdownResponse::Refused { cause } =>
        {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, cause));
        }
    };

    let edited = markdown.replace("## Decision", "## The decision");
    let staged = into.join("staged.md");
    std::fs::write(&staged, &edited)?;

    return Ok(staged);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The area every fixture test in this module builds its scratch directories under.
    const TEST_SUPPORT_AREA: Area = Area("test-support");

    #[test]
    fn Test_Unique_Scratch_Directory_Should_Give_Two_Calls_In_One_Area_Different_Directories()
    {
        let first = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "same-label")
            .expect("the temp directory is writable and this call's own name is fresh");
        let second = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "same-label")
            .expect("the temp directory is writable and the counter gave this call a fresh name");

        assert_ne!(first, second);
        assert!(first.is_dir());
        assert!(second.is_dir());

        let _ignored = std::fs::remove_dir_all(&first);
        let _ignored = std::fs::remove_dir_all(&second);
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Accept_Every_Tagged_Values_Own_Outcome_Name()
    {
        for (value, outcome) in Sample_Outcomes()
        {
            Assert_Round_Trips_As_Json(&value, outcome)
                .expect("every Sample variant serializes and parses back as a tagged object");
        }
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Panic_When_The_Outcome_Field_Does_Not_Match()
    {
        for (value, _outcome) in Sample_Outcomes()
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // The assertion is the subject here, so the returned error is not what this
                // test is asking about and the panic it raises is.
                let _ignored = Assert_Round_Trips_As_Json(&value, "not-a-real-outcome");
            }));

            assert!(result.is_err(), "{value:?} must panic when checked against the wrong outcome");
        }
    }

    #[test]
    fn Test_Staged_Heading_Rename_Should_Replace_The_Decision_Heading_In_A_Real_Record()
    {
        let into = Unique_Scratch_Directory(TEST_SUPPORT_AREA, "staged-heading-rename")
            .expect("the temp directory is writable and this call's own name is fresh");

        let staged = Staged_Heading_Rename("D-132", &into)
            .expect("D-132 is a governing record embedded in this binary");
        let content = std::fs::read_to_string(&staged).expect("reads the staged file back");

        let _ignored = std::fs::remove_dir_all(&into);

        assert!(content.contains("## The decision"), "{content}");
        assert!(!content.contains("## Decision\n"), "{content}");
    }

    #[derive(Debug, Serialize)]
    #[serde(tag = "outcome", rename_all = "snake_case")]
    enum Sample
    {
        Accepted,
        Completed,
        Rejected,
    }

    /// Every `Sample` variant paired with the exact snake_case address `serde`'s own
    /// `rename_all = "snake_case"` gives it -- the one outcome name `Assert_Round_Trips_As_Json`
    /// must accept for each, and every other name it must reject.
    fn Sample_Outcomes() -> Vec<(Sample, &'static str)>
    {
        return vec![
            (Sample::Accepted, "accepted"),
            (Sample::Completed, "completed"),
            (Sample::Rejected, "rejected"),
        ];
    }

    /// The area the routing tests' empty roots are created under.
    const ROUTING_AREA: Area = Area("routing");

    /// How long a routing test waits before concluding that a judgment was held back.
    ///
    /// Only the failing direction depends on it. A route that skips its permit judges an empty
    /// root, which loads no database. Measured with each route made to skip its permit, one
    /// test at a time, the run answered in under 0.1 s and the comparison in under 0.6 s
    /// counting the test process's own start; in the unbounded suite the empty-root run test
    /// finished within 0.1 s of 176 tests starting at once. A route that waits never answers
    /// while the permit is held, however long this is.
    const HELD_BACK_WINDOW: std::time::Duration = std::time::Duration::from_secs(5);

    /// How long a routing test waits for a returned permit to admit the judgment waiting on it.
    /// It bounds only the failure, so a judgment that is never admitted fails rather than hangs.
    const ADMITTED_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

    /// A bound of one, for [`Test_Judged_Run_Should_Wait_For_A_Permit_Before_It_Judges`] alone.
    static RUN_PROBE: JudgmentPermits = JudgmentPermits::New(1);

    /// A bound of one, for [`Test_Judged_Compare_Should_Wait_For_A_Permit_Before_It_Judges`]
    /// alone.
    static COMPARE_PROBE: JudgmentPermits = JudgmentPermits::New(1);

    /// Takes `probe`'s only permit on a thread of its own and keeps it until the returned
    /// sender is used or dropped.
    fn Holding_The_Only_Permit(probe: &'static JudgmentPermits) -> std::sync::mpsc::Sender<()>
    {
        let (holding, held) = std::sync::mpsc::channel();
        let (release, released) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || probe.Holding(|| {
            holding.send(()).expect("the test body is still receiving");
            // Returns either way: released, or the body gave up and dropped the sender.
            let _released = released.recv();
        }));
        held.recv_timeout(ADMITTED_WINDOW).expect("the holder takes the only permit");

        return release;
    }

    /// Runs `route` on a thread whose judgments `probe` admits, and asserts that it waits
    /// while `probe`'s only permit is held and runs once that permit is returned.
    ///
    /// `route` calls [`Judged_Run`] or [`Judged_Compare`] exactly as a database-building test
    /// does. Only the permits behind it are this test's, substituted for its own thread, so it
    /// neither waits behind the suite's database-building tests nor holds any of them back.
    /// Both halves are asserted, because a route that never ran at all would pass the first.
    /// The route runs on a detached thread, so one that is never admitted fails at
    /// [`ADMITTED_WINDOW`] instead of hanging the test.
    fn Assert_Admitted_Through_The_Bound<Answer: Send + 'static>(
        probe: &'static JudgmentPermits,
        route: impl FnOnce() -> Answer + Send + 'static,
    ) -> Answer
    {
        let release = Holding_The_Only_Permit(probe);
        let (answered, answer) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            SUBSTITUTED.set(Some(probe));
            answered.send(route()).expect("the test body is still receiving");
        });

        assert!(
            answer.recv_timeout(HELD_BACK_WINDOW).is_err(),
            "the route answered while the only permit was held, so a test's gate judgment runs past the bound"
        );
        release.send(()).expect("the holder is still waiting for this");

        return answer
            .recv_timeout(ADMITTED_WINDOW)
            .expect("the returned permit must admit the judgment waiting for it");
    }

    /// The route every gate-run test in this crate takes waits for a permit before it judges.
    ///
    /// Without this, [`Judged_Run`] calling the handler directly would pass every other test in
    /// the crate, and the only witness would be the unbounded suite's 13 GB.
    #[test]
    fn Test_Judged_Run_Should_Wait_For_A_Permit_Before_It_Judges()
    {
        let root = Unique_Scratch_Directory(ROUTING_AREA, "run")
            .expect("the temp directory is writable and this call's own name is fresh");
        let command = GateCommand { root: root.clone(), ..Default::default() };

        let response = Assert_Admitted_Through_The_Bound(&RUN_PROBE, move || return Judged_Run(&command));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(response.root, root, "the admitted route must judge the root it was handed");
    }

    /// The route every gate-compare test in this crate takes waits for a permit before it judges.
    ///
    /// The same guard as the run's, for the other route, which a comparison test could keep
    /// bypassing after the run's route was fixed.
    #[test]
    fn Test_Judged_Compare_Should_Wait_For_A_Permit_Before_It_Judges()
    {
        let root = Unique_Scratch_Directory(ROUTING_AREA, "compare")
            .expect("the temp directory is writable and this call's own name is fresh");
        let command = GateCommand { root: root.clone(), ..Default::default() };

        let response =
            Assert_Admitted_Through_The_Bound(&COMPARE_PROBE, move || return Judged_Compare(&command, &command));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_ne!(response.baseline, response.candidate, "the admitted route must have made both judgments");
    }
}
