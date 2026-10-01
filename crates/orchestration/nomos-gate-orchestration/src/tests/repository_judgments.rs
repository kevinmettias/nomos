//! Every judgment this crate's tests make over the repository's own root goes through here, and
//! only those.
//!
//! # Why these judgments are different from every other test in the crate
//!
//! A judgment over this repository's root with every rule selected demands both compiler-backed
//! families, and each one loads the whole resolved crate graph of this workspace through
//! `ra_ap_hir` -- one after the other inside one judgment, so one database is live at a time.
//! Measured on 2026-09-26 at `0e9d7918`, on the 31.8 GB, 32-thread machine this repository is
//! developed on, one test at a time: one such judgment took 245 to 291 s and peaked at 2.1 GB.
//! libtest runs every test of a crate as a thread of one process, one thread per core, so all 27
//! tests that make one started loading a database at the same moment. That run reached a 16.1 GB
//! working set and 36.8 GB committed, left 4 MB of the machine available, and had finished none
//! of the 27 when it was stopped at 2,700 s -- not slow but stalled, so no timeout would have been
//! long enough. The other two hundred tests in the crate finished in 3 s and are left exactly as
//! libtest schedules them.
//!
//! # What is done about it, and why each half is not a preference
//!
//! **A judgment asked for twice is made once.** Several tests need the same real finding before
//! they can address it with a policy, and each used to judge the repository again to get it. A
//! judgment is a function of its inputs and of the tree it reads, and nothing a test in this crate
//! does writes into this tree: a run writes to its root only through `nomos-gate-history.json`,
//! which this tree does not have. So two calls with equal inputs are the same question, and
//! [`SharedJudgments`] answers it once for both.
//! Scratch roots are never shared: their tests write them.
//!
//! **At most [`LIVE_REPOSITORY_JUDGMENTS`] are made at once.** The remaining distinct judgments
//! still each need a database; the bound is on how many exist together, which is the quantity
//! that exhausted the machine. It is not a bound on parallelism in general: a judgment waiting
//! here holds a libtest thread and nothing else, and every other test runs on the threads that
//! remain.
//!
//! # What proves that the judgments go through the bound
//!
//! Nothing about the bound shows in an answer. A judgment made past it answers exactly what one
//! made through it answers, so the only other witness is the memory the unbounded suite took,
//! which is how the defect was found. Each of the two routes is therefore guarded by a test at
//! the end of this file. The test calls its route exactly as a database-building test does, over
//! the repository's own root, with a bound of one substituted for [`PERMITS`] on its own thread.
//! It holds that bound's only permit and asserts that the route does not answer until the permit
//! is returned. A route that stopped taking a permit answers at once and fails its test by name,
//! and so does a root check that stopped recognising the root. The judgment is over an empty
//! source list, which reaches no rule, so neither test loads a database.
//!
//! What those tests cannot see is a new test that judges the repository's root without calling
//! either route, through `Run_Gate` or `Explain_Gate` directly. That is kept out by the rule this
//! module's first line states and by a grep of the crate for those two calls, not by a test.

use super::shared_judgments::SharedJudgments;
use super::{Command_At, Gate_Platform, Repository_Root};
use crate::{Explain_Gate, Explanation, FindingQuery, GateCommand, GateExplainResult, GateRunResult, Run_Gate};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Digest128, RuleId, RunId};
use nomos_rules::{SourceFile, COMPLETENESS_MIRROR};
use nomos_test_permits::JudgmentPermits;
use std::cell::Cell;
use std::path::{Path, PathBuf};

/// How many judgments over the repository's own root may hold a compiler database at once.
///
/// Three, from two measurements and a share. One judgment peaks at 2.1 GB (above), so three are
/// about 6.3 GB -- a fifth of the 31.8 GB machine, the share a test suite can take on a
/// development machine that is also running an editor, a language server and other sessions'
/// builds, which on the day this was measured left between 4 and 14 GB of it available. Measured
/// with this bound, from a cold start and beside another session's 7 GB test run: a 6.4 GB peak
/// working set and 6.7 GB committed for the whole suite, and all 237 tests green in 3,373 s. The
/// cost is wall time: about thirty distinct judgments, three at a time.
///
/// Reopen this if one judgment's peak grows (a third compiler-backed family, or a change in how
/// the two existing ones load the crate graph, which is `P146-THE-SEVENFOLD-SELF-CHECK-COST-...`'s
/// question), if the number of distinct judgments grows enough that a third of their serial time
/// no longer fits the budget an item declares, or if the suite has to run on a machine with less
/// memory than this one.
const LIVE_REPOSITORY_JUDGMENTS: usize = 3;

/// The one bound every judgment over the repository's root is admitted through.
static PERMITS: JudgmentPermits = JudgmentPermits::New(LIVE_REPOSITORY_JUDGMENTS);

thread_local! {
    /// The permits a judgment made on this thread waits for in place of [`PERMITS`], once a
    /// routing test below has substituted a bound of its own for this one thread.
    static SUBSTITUTED: Cell<Option<&'static JudgmentPermits>> = const { Cell::new(None) };
}

/// The permits a judgment over the repository's root made on this thread is admitted through:
/// [`PERMITS`], unless a routing test below has substituted its own for this thread alone.
///
/// A judgment is made on the thread that asks for it, because [`SharedJudgments`] runs the
/// first caller's judgment on that caller's thread. So a substitution reaches the probe's own
/// judgment and no other test's.
fn Admitting() -> &'static JudgmentPermits
{
    return SUBSTITUTED.get().unwrap_or(&PERMITS);
}

/// Every `Run_Gate` judgment over the repository's root this process has made.
static RUNS: SharedJudgments<RunInputs, GateRunResult> = SharedJudgments::New();

/// Every `Explain_Gate` judgment over the repository's root this process has made.
static EXPLANATIONS: SharedJudgments<ExplainInputs, ExplainAnswer> = SharedJudgments::New();

/// Everything a `Run_Gate` judgment is a function of, besides the tree: the platform is
/// [`Gate_Platform`] for every judgment made here, so it is not a part of the key that can differ.
#[derive(PartialEq)]
struct RunInputs
{
    sources: Vec<SourceFile>,
    command: GateCommand,
    run: RunId,
}

/// Everything an `Explain_Gate` judgment is a function of, besides the tree.
#[derive(PartialEq)]
struct ExplainInputs
{
    sources: Vec<SourceFile>,
    command: GateCommand,
    query: FindingQuery,
}

/// A [`GateExplainResult`]'s fields, kept so that one can be handed to each caller.
/// `GateExplainResult` is not `Clone`, and a test's need to share one is no reason to widen the
/// crate's public surface.
#[derive(Clone)]
struct ExplainAnswer
{
    root: PathBuf,
    check_outcome: CheckOutcome,
    explanation: Explanation,
}

/// `Run_Gate` over `sources` under `command`, recorded as `run`, under [`Gate_Platform`].
///
/// Over the repository's own root the judgment is shared and bounded as this module's doc says;
/// over any other root it is made directly, exactly as before.
pub(crate) fn Judged_Run(sources: Vec<SourceFile>, command: &GateCommand, run: RunId) -> GateRunResult
{
    if !Is_Repository_Root(&command.root)
    {
        return Run_Gate(Some(sources), Gate_Platform(), command, run);
    }

    let inputs = RunInputs { sources, command: command.clone(), run };

    return RUNS.Answer_For(inputs, |inputs| {
        return Admitting().Holding(|| return Run_Gate(Some(inputs.sources.clone()), Gate_Platform(), &inputs.command, inputs.run));
    });
}

/// `Explain_Gate` answering `query` over `sources` under `command`, under [`Gate_Platform`],
/// shared and bounded over the repository's own root exactly as [`Judged_Run`] is.
pub(crate) fn Judged_Explanation(sources: Vec<SourceFile>, command: &GateCommand, query: &FindingQuery) -> GateExplainResult
{
    if !Is_Repository_Root(&command.root)
    {
        return Explain_Gate(Some(sources), Gate_Platform(), command, query);
    }

    let inputs = ExplainInputs { sources, command: command.clone(), query: query.clone() };
    let answer = EXPLANATIONS.Answer_For(inputs, |inputs| {
        let result = Admitting().Holding(|| return Explain_Gate(Some(inputs.sources.clone()), Gate_Platform(), &inputs.command, &inputs.query));
        return ExplainAnswer { root: result.root, check_outcome: result.check_outcome, explanation: result.explanation };
    });

    return GateExplainResult { root: answer.root, check_outcome: answer.check_outcome, explanation: answer.explanation };
}

/// Whether `root` is this repository's own root -- the one tree every fixture here reads and
/// none writes, and the one a compiler-backed provider loads a crate graph for.
fn Is_Repository_Root(root: &Path) -> bool
{
    return root == Repository_Root();
}

/// How long a routing test waits before concluding that a judgment was held back.
///
/// Only the failing direction depends on it. A route that skips its permit judges an empty
/// source list, which reaches no rule and loads no database. Measured on 2026-09-27 with each
/// route made to skip its permit, one test at a time, each answered in under 0.01 s. A route
/// that waits never answers while the permit is held, however long this is.
const HELD_BACK_WINDOW: std::time::Duration = std::time::Duration::from_secs(5);

/// How long a routing test waits for a returned permit to admit the judgment waiting on it. It
/// bounds only the failure, so a judgment that is never admitted fails rather than hangs.
const ADMITTED_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

/// A bound of one, for [`Test_Judged_Run_Should_Wait_For_A_Permit_Before_It_Judges_The_Repository_Root`]
/// alone.
static RUN_PROBE: JudgmentPermits = JudgmentPermits::New(1);

/// A bound of one, for
/// [`Test_Judged_Explanation_Should_Wait_For_A_Permit_Before_It_Judges_The_Repository_Root`] alone.
static EXPLANATION_PROBE: JudgmentPermits = JudgmentPermits::New(1);

/// The run identity the run probe is recorded under.
///
/// Its own, because the probe's inputs have to be a key [`RUNS`] files for the probe alone. One
/// test already judges an empty source list over the repository's root, under the fixed
/// identity every fixture uses. Sharing that key would hand whichever of the two asked second
/// the other's answer: the probe could then answer without waiting for anything, or that test
/// could wait behind the permit the probe holds.
const ROUTING_PROBE_RUN: RunId = RunId::From_Digest(Digest128::From_Bytes([0xA5; Digest128::BYTE_LENGTH]));

/// The location the explanation probe asks about, which no other query in this crate names, so
/// the probe's inputs are a key [`EXPLANATIONS`] files for the probe alone.
const ROUTING_PROBE_LOCATION: &str = "routing-probe.rs";

/// Takes `probe`'s only permit on a thread of its own and keeps it until the returned sender is
/// used or dropped.
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

/// Runs `route` on a thread whose judgments `probe` admits, and asserts that it waits while
/// `probe`'s only permit is held and answers once that permit is returned.
///
/// `route` calls [`Judged_Run`] or [`Judged_Explanation`] exactly as a database-building test
/// does. Only the permits behind it are this test's, substituted for its own thread, so it
/// neither waits behind the suite's judgments nor holds any of them back. Both halves are
/// asserted, because a route that never ran at all would pass the first. The route runs on a
/// detached thread, so one that is never admitted fails at [`ADMITTED_WINDOW`] instead of
/// hanging the test.
fn Assert_Admitted_Through_The_Bound<Answer: Send + 'static>(probe: &'static JudgmentPermits, route: impl FnOnce() -> Answer + Send + 'static) -> Answer
{
    let release = Holding_The_Only_Permit(probe);
    let (answered, answer) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        SUBSTITUTED.set(Some(probe));
        answered.send(route()).expect("the test body is still receiving");
    });

    assert!(
        answer.recv_timeout(HELD_BACK_WINDOW).is_err(),
        "the route answered while the only permit was held, so a judgment over the repository's root runs past the bound"
    );
    release.send(()).expect("the holder is still waiting for this");

    return answer.recv_timeout(ADMITTED_WINDOW).expect("the returned permit must admit the judgment waiting for it");
}

/// The route every run over the repository's root takes waits for a permit before it judges.
///
/// Without this, [`Judged_Run`] judging without a permit would pass every other test in the
/// crate, and the only witness would be the unbounded suite's 36.8 GB.
#[test]
fn Test_Judged_Run_Should_Wait_For_A_Permit_Before_It_Judges_The_Repository_Root()
{
    let command = Command_At(Repository_Root());

    let result = Assert_Admitted_Through_The_Bound(&RUN_PROBE, move || return Judged_Run(Vec::new(), &command, ROUTING_PROBE_RUN));

    assert!(
        matches!(result.check_outcome, CheckOutcome::NoSource),
        "the probe must judge nothing, or it loaded a database: {:?}",
        result.check_outcome
    );
}

/// The route every explanation over the repository's root takes waits for a permit before it
/// judges.
///
/// The same guard as the run's, for the other route. An explanation judges every rule whatever
/// its command selects, so every explanation over the repository's root with a source to judge
/// loads both compiler databases.
#[test]
fn Test_Judged_Explanation_Should_Wait_For_A_Permit_Before_It_Judges_The_Repository_Root()
{
    let command = Command_At(Repository_Root());
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: ROUTING_PROBE_LOCATION.to_owned() };

    let result = Assert_Admitted_Through_The_Bound(&EXPLANATION_PROBE, move || return Judged_Explanation(Vec::new(), &command, &query));

    assert!(
        matches!(result.check_outcome, CheckOutcome::NoSource),
        "the probe must judge nothing, or it loaded a database: {:?}",
        result.check_outcome
    );
}
