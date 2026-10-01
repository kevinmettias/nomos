//! The provider table every test under `crate::tests` composes: the crate's own table, with
//! its two compiler-backed rows admitted through one bound.
//!
//! # Why these two rows, and why only here
//!
//! `copy_clones` and `nested_locks` are the rows whose provider loads the whole resolved crate
//! graph of the root it is handed, with a real sysroot, through `ra_ap_hir`. Over this
//! repository's own root, one call holds one compiler database for as long as it runs, and
//! drops it when it returns. libtest runs every test in this crate as a thread of one process,
//! one per core, so every test that demands either family over that root began loading at the
//! same moment, and nothing in libtest knows that those few each hold a database.
//!
//! Measured on 2026-09-27 at `160a3ac5`, on the 31.8 GB, 32-thread machine this repository is
//! developed on. Nine tests here load databases over this repository's root: two in
//! `composition`, six in `fact_trail` and one in `reuse`. Between them they make twelve runs
//! that demand both families, so twenty-four loads. Run alone, one load kept one core busy for
//! 106 to 118 s, peaked at 2.1 GB committed, and fell back under 100 MB when it returned. At
//! default parallelism all nine began at once. The suite still passed, 93 of 93 in 1,618 s, but
//! its test process peaked at 19.1 GB committed and a 14.0 GB working set, and the machine fell
//! to 198 MB available. The other 84 tests had all finished within 40 s.
//!
//! # What is bounded, and what is left exactly as it was
//!
//! A compiler-backed row waits for a permit, then calls the row the composed table declares
//! with the same three arguments and hands back its answer unchanged. Every other row is the
//! composed table's own, by construction. So the bound is on how many databases exist at
//! once, which is the quantity that exhausted the machine, and not on how many tests run: a
//! test waiting here holds a libtest thread and nothing else, and it waits only when it is
//! about to load a database. No judgment moves and no answer is shared -- every call still
//! loads its own database, one after another instead of all at once.
//!
//! The bound wraps the table's rows rather than living in them.
//! `crate::composition::provider_table` is a declared constant whose rows never read a count,
//! which its own doc states under `OD-ROADMAP-003`, and a permit is a count. So the table stays
//! what it declares, and the bound is this harness's, laid over it.
//!
//! The permits are `nomos_test_permits::JudgmentPermits`, the same semaphore
//! `nomos-gate-orchestration`'s suite admits its judgments through. Only the mechanism is shared:
//! there a permit admits one judgment and here one compiler-backed provider call, so the count
//! below is this suite's own, derived from this suite's measurement.
//!
//! # What proves that the suite's tests go through the bound
//!
//! Nothing about the bound shows in an answer. A row called past it answers exactly what one
//! called through it answers, so the only other witness is the memory the unbounded suite took,
//! which is how the defect was found. Two kinds of test here guard it, one for each place it
//! could be lost. The row tests admit each wrapped row through a bound of its own and prove that
//! the wrapper waits. The routing tests take each compiler-backed row from [`Bounded_Providers`]
//! itself, the table every database-building test runs through, with a bound of one substituted
//! for the suite's on their own thread. So a `Bounded_Providers` that stopped wrapping a row
//! fails a routing test by name. Every test here asks about an empty scratch root, which each
//! family refuses without loading a database.
//!
//! What the routing tests cannot see is a new test that composes its providers some other way,
//! through `crate::composition::Composed_Providers` directly. That is kept out by the rule
//! `crate::tests`' own doc states and by a grep of this suite for that call, not by a test.

use nomos_analysis::Context;
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_test_permits::JudgmentPermits;
use std::path::Path;

use crate::composed_providers::{ProjectFactProvider, SubjectFact};
use crate::composition::Composed_Providers;
use crate::ComposedProviders;

use super::{Scratch_Directory, Source_File, SourceText, Test_Variant};

/// How many compiler databases this test process may hold at once.
///
/// Three, from one measurement and a share. It is the derivation `nomos-gate-orchestration`'s
/// `LIVE_REPOSITORY_JUDGMENTS` gives for the same machine, applied to the unit measured here.
/// One load peaks at 2.1 GB committed, so three at once come to about 6.4 GB. That is a fifth
/// of the 31.8 GB machine, the share a test suite can take on a development machine that is
/// also running an editor, a language server and other sessions' builds. The cost is wall time. Twenty-four loads of about 115 s each, three
/// at a time, need at least about 920 s. The `reuse` test's own six loads run one after
/// another, so they take at least about 690 s whatever the bound.
///
/// Measured with this bound on 2026-09-27, from a cold target directory and at default
/// parallelism: 97 of 97 passed in 1,372 s. The test process peaked at 6.6 GB committed and a
/// 6.4 GB working set, where the unbounded suite had reached 19.1 GB and 14.0 GB. The `reuse`
/// test finished last, 360 s after every other test. The permits are handed out in no
/// particular order, so its six loads kept queueing behind other tests' loads.
///
/// Reopen this if one load's peak grows, which a third compiler-backed family or a change in
/// how either family loads the crate graph would do. Reopen it if enough loads are added that a
/// third of their serial time no longer fits the budget a predicate over this suite declares,
/// or if the suite has to run on a machine with less memory than this one.
const LIVE_COMPILER_DATABASES: usize = 3;

/// Which permits a compiler-backed row is admitted through.
///
/// A type rather than a value, because a row is a `fn` pointer and captures nothing: the only
/// way to hand one a bound is to name the bound in the row's own type. The tests below need
/// that, to admit the rows through bounds of their own rather than contend with this suite's
/// database-loading tests for the suite's.
trait Admission
{
    /// The permits every row admitted under this type waits on.
    fn Permits() -> &'static JudgmentPermits;
}

/// The bound every test under `crate::tests` composes its providers under.
struct ThisSuite;

thread_local! {
    /// The permits a row admitted under [`ThisSuite`] waits on when it is called on this thread,
    /// in place of the suite's own, once a routing test below has substituted a bound of its own
    /// for this one thread.
    static SUBSTITUTED: std::cell::Cell<Option<&'static JudgmentPermits>> = const { std::cell::Cell::new(None) };
}

impl Admission for ThisSuite
{
    /// The suite's own permits, unless a routing test has substituted its own for the calling
    /// thread. A routing test calls its row itself, on a thread of its own, so the substitution
    /// reaches that one call. Every other thread has none and waits on the suite's permits.
    fn Permits() -> &'static JudgmentPermits
    {
        static PERMITS: JudgmentPermits = JudgmentPermits::New(LIVE_COMPILER_DATABASES);

        return SUBSTITUTED.get().unwrap_or(&PERMITS);
    }
}

/// The composed provider table, with its two compiler-backed rows admitted through this
/// suite's bound: the one table every test under `crate::tests` runs through.
pub(super) fn Bounded_Providers<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>() -> ComposedProviders<Launcher, Fs, Env>
{
    return Admitted_By::<ThisSuite, Launcher, Fs, Env>();
}

/// The composed table, its two compiler-backed rows admitted through `Bound`'s permits and
/// every other row its own.
fn Admitted_By<Bound: Admission, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>() -> ComposedProviders<Launcher, Fs, Env>
{
    return ComposedProviders {
        copy_clones: Copy_Clones_Admitted::<Bound, Launcher, Fs, Env>,
        nested_locks: Nested_Locks_Admitted::<Bound, Launcher, Fs, Env>,
        ..Composed_Providers()
    };
}

/// The composed table's own `copy_clones` row, called once `Bound` admits it.
fn Copy_Clones_Admitted<Bound: Admission, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    root: &Path, context: &Context, environment: &Env,
) -> Result<SubjectFact, String>
{
    let declared = Composed_Providers::<Launcher, Fs, Env>().copy_clones;

    return Bound::Permits().Holding(|| return declared(root, context, environment));
}

/// The composed table's own `nested_locks` row, called once `Bound` admits it.
fn Nested_Locks_Admitted<Bound: Admission, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    root: &Path, context: &Context, environment: &Env,
) -> Result<SubjectFact, String>
{
    let declared = Composed_Providers::<Launcher, Fs, Env>().nested_locks;

    return Bound::Permits().Holding(|| return declared(root, context, environment));
}

/// How long the tests below wait before concluding a row was held back.
///
/// Only the passing direction depends on it, as in `nomos_test_permits`' own tests: a row that
/// skips its permit answers a scratch root at once, which reads as a failure well inside the
/// window, and a row that waits never answers at all while the permit is held.
const HELD_BACK_WINDOW: std::time::Duration = std::time::Duration::from_millis(500);

/// How long the tests below wait for a returned permit to admit the row waiting on it. It
/// bounds only the failure, so a row that is never admitted fails rather than hangs.
const ADMITTED_WINDOW: std::time::Duration = std::time::Duration::from_secs(30);

/// A bound of one, for [`Test_The_Copy_Clones_Row_Should_Wait_For_A_Permit_And_Answer_As_Declared`] alone.
struct CopyClonesProbe;

impl Admission for CopyClonesProbe
{
    fn Permits() -> &'static JudgmentPermits
    {
        static PERMITS: JudgmentPermits = JudgmentPermits::New(1);

        return &PERMITS;
    }
}

/// A bound of one, for [`Test_The_Nested_Locks_Row_Should_Wait_For_A_Permit_And_Answer_As_Declared`] alone.
struct NestedLocksProbe;

impl Admission for NestedLocksProbe
{
    fn Permits() -> &'static JudgmentPermits
    {
        static PERMITS: JudgmentPermits = JudgmentPermits::New(1);

        return &PERMITS;
    }
}

/// The table the probes below read a row from, at the real platform's own port types.
type StdProviders = ComposedProviders<StdProgramLauncher, StdFileSystem, StdEnvironment>;

/// A real reading context, built from one placeholder source: the rows below are asked about
/// a root that is not a Cargo project, so what they are handed besides it only has to be real.
fn Placeholder_Context() -> Context
{
    let placeholder = [Source_File("placeholder.rs", SourceText("pub fn Placeholder() {}\n"))];
    let registry = crate::composition::Registered(crate::composition::provider_table::Offer_Composed_Providers).expect("fixture composition");

    return crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None).expect("a single real file ingests");
}

/// Takes `Bound`'s only permit on a thread of its own and keeps it until the returned sender
/// is used or dropped.
fn Holding_The_Only_Permit<Bound: Admission>() -> std::sync::mpsc::Sender<()>
{
    let (holding, held) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || Bound::Permits().Holding(|| {
        holding.send(()).expect("the test body is still receiving");
        // Returns either way: released, or the body gave up and dropped the sender.
        let _released = released.recv();
    }));
    held.recv_timeout(ADMITTED_WINDOW).expect("the holder takes the only permit");

    return release;
}

/// Asserts that `row`, read from a table admitted through `Bound`, waits while `Bound`'s only
/// permit is held, runs once it is returned, and then answers exactly what the declared row
/// answers for the same arguments.
///
/// The root is an empty scratch directory, which the provider refuses without loading a
/// database, so the probe costs nothing and its refusal is still the declared row's own
/// answer. Both halves of the timing are asserted for the reason `nomos_test_permits`' own tests
/// give: a row that waited forever would pass the first. The row runs on a detached thread so
/// that one never admitted fails at [`ADMITTED_WINDOW`] instead of hanging the test.
///
/// What a scratch root cannot tell apart is *which* declared row a wrapper calls, because both
/// families refuse it with the same words. That half is guarded by the database-building tests
/// themselves. A wrapper calling the other family's row files no fact for its own family, so
/// its rule is reported `DependencyUnavailable`. `composition`'s
/// `Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete` then fails on an incomplete claim,
/// for either row.
fn Assert_Row_Is_Admitted_And_Answers_As_Declared<Bound: Admission>(family: &str, row: fn(&StdProviders) -> ProjectFactProvider<StdEnvironment>)
{
    let root = Scratch_Directory(family);
    let context = Placeholder_Context();
    let declared = row(&Composed_Providers())(&root, &context, &StdEnvironment).err();
    assert!(declared.is_some(), "an empty scratch root must be refused, or this probe loaded a database: {}", root.display());

    let release = Holding_The_Only_Permit::<Bound>();
    let admitted = row(&Admitted_By::<Bound, StdProgramLauncher, StdFileSystem, StdEnvironment>());
    let (answered, answer) = std::sync::mpsc::channel();
    std::thread::spawn(move || answered.send(admitted(&root, &context, &StdEnvironment).err()).expect("the test body is still receiving"));

    assert!(answer.recv_timeout(HELD_BACK_WINDOW).is_err(), "the {family} row answered while the only permit was held, so it loads a database past the bound");
    release.send(()).expect("the holder is still waiting for this");
    let answered = answer.recv_timeout(ADMITTED_WINDOW).expect("the returned permit must admit the row that was waiting for it");
    assert_eq!(answered, declared, "the admitted {family} row must answer exactly what the declared row answers");
}

#[test]
fn Test_The_Copy_Clones_Row_Should_Wait_For_A_Permit_And_Answer_As_Declared()
{
    Assert_Row_Is_Admitted_And_Answers_As_Declared::<CopyClonesProbe>("copy-clones", |providers| return providers.copy_clones);
}

#[test]
fn Test_The_Nested_Locks_Row_Should_Wait_For_A_Permit_And_Answer_As_Declared()
{
    Assert_Row_Is_Admitted_And_Answers_As_Declared::<NestedLocksProbe>("nested-locks", |providers| return providers.nested_locks);
}

/// A bound of one, substituted for the suite's own by
/// [`Test_The_Suites_Copy_Clones_Row_Should_Wait_For_The_Suites_Bound`] alone.
struct SuiteCopyClonesProbe;

impl Admission for SuiteCopyClonesProbe
{
    fn Permits() -> &'static JudgmentPermits
    {
        static PERMITS: JudgmentPermits = JudgmentPermits::New(1);

        return &PERMITS;
    }
}

/// A bound of one, substituted for the suite's own by
/// [`Test_The_Suites_Nested_Locks_Row_Should_Wait_For_The_Suites_Bound`] alone.
struct SuiteNestedLocksProbe;

impl Admission for SuiteNestedLocksProbe
{
    fn Permits() -> &'static JudgmentPermits
    {
        static PERMITS: JudgmentPermits = JudgmentPermits::New(1);

        return &PERMITS;
    }
}

/// Asserts that `row`, read from [`Bounded_Providers`], waits while the suite's bound admits
/// nothing more, and runs once a permit is returned.
///
/// [`Bounded_Providers`] is the table every database-building test in this suite composes, so
/// this is the route those tests take, and not a table built for the probe. The suite's own
/// permits are not the ones held: holding them would hold back this suite's database-building
/// tests and wait behind them. Instead `Probe`'s bound of one is substituted for the suite's on
/// the one thread the row is called on, which is what [`ThisSuite`] admits through there. Both
/// halves of the timing are asserted, for the reason the row tests give.
///
/// The root is an empty scratch directory, which the provider refuses without loading a
/// database, and the refusal is asserted so that a probe which did load one says so. Measured on
/// 2026-09-27 with [`Bounded_Providers`] returning the unbounded table, one test at a time, each
/// row refused the root in under 0.01 s, well inside [`HELD_BACK_WINDOW`].
fn Assert_The_Suites_Row_Waits_For_The_Suites_Bound<Probe: Admission>(family: &str, row: fn(&StdProviders) -> ProjectFactProvider<StdEnvironment>)
{
    let root = Scratch_Directory(&format!("suite-{family}"));
    let shown = root.display().to_string();
    let context = Placeholder_Context();

    let release = Holding_The_Only_Permit::<Probe>();
    let bounded = row(&Bounded_Providers());
    let (answered, answer) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        SUBSTITUTED.set(Some(Probe::Permits()));
        answered.send(bounded(&root, &context, &StdEnvironment).err()).expect("the test body is still receiving");
    });

    assert!(
        answer.recv_timeout(HELD_BACK_WINDOW).is_err(),
        "the suite's {family} row answered while the suite's bound admitted nothing more, so this suite's tests load databases past it"
    );
    release.send(()).expect("the holder is still waiting for this");
    let refused = answer.recv_timeout(ADMITTED_WINDOW).expect("the returned permit must admit the row that was waiting for it");
    assert!(refused.is_some(), "an empty scratch root must be refused, or this probe loaded a database: {shown}");
}

#[test]
fn Test_The_Suites_Copy_Clones_Row_Should_Wait_For_The_Suites_Bound()
{
    Assert_The_Suites_Row_Waits_For_The_Suites_Bound::<SuiteCopyClonesProbe>("copy-clones", |providers| return providers.copy_clones);
}

#[test]
fn Test_The_Suites_Nested_Locks_Row_Should_Wait_For_The_Suites_Bound()
{
    Assert_The_Suites_Row_Waits_For_The_Suites_Bound::<SuiteNestedLocksProbe>("nested-locks", |providers| return providers.nested_locks);
}
