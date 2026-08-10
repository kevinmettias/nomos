//! An item cannot be finished by a predicate that checks less than the gate does.
//!
//! Every item's `verification` argv is a `cargo test` invocation. The gate lints first,
//! with `-D warnings`, so an item could be recorded verified with the gate already red.
//! `P9-SKIP` and `P9-PHASE-GAP` were both finished that way and caught afterwards by a
//! person running clippy by hand — which is the arrangement the ledger exists to replace.
//!
//! Those two instances cannot be replayed from git, and that is worth stating precisely.
//! Clippy passes on both commits (`26566c6` and `31f4450`, measured, with a confirmed-red
//! control). The defect lived in the working tree between `work finish` and `git commit`
//! — 180 and 185 seconds respectively — and was repaired before the commit was written.
//! So the instances are reproduced here in the shape they actually had: a predicate that
//! exits zero while the gate's own step does not.
//!
//! Every test has a negative control, because a guard that has never been watched failing
//! is not a guard.

use nomos_ledger::{
    Claim, FileLedger, Finish, FinishRefusal, GateUnknown, ItemId, ItemState,
    LedgerDocument, LedgerItem, Territory, VerificationPredicate,
};
use nomos_platform::{
    Clock, Command, ExitOutcome, ProcessLauncher, ProcessOutput, Timestamp,
};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Duration;

const NOW: i64 = 1_000_000;
const HOLDER: &str = "agent-a";

/// The workflow a derived gate step is read out of. Deliberately the real shape, so that
/// a change to the repository's own gate breaks these tests rather than passing them.
const WORKFLOW: &str = "name: gate\n\
                        \n\
                        jobs:\n\
                        \x20 gate:\n\
                        \x20   steps:\n\
                        \x20     - uses: actions/checkout@v4\n\
                        \x20     - name: Lint\n\
                        \x20       run: cargo clippy --workspace --all-targets -- -D warnings\n\
                        \x20     - name: Test\n\
                        \x20       run: cargo test --workspace\n";

struct FixedClock(i64);

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

/// A launcher that answers by argv rather than by running anything.
///
/// The point of these tests is the ordering and the arithmetic of two exit codes, not
/// whether clippy works. Running a real clippy here would make the suite slow and would
/// couple it to whatever the workspace happens to contain.
struct Scripted
{
    lint_exit: i32,
    predicate_exit: i32,
    calls: RefCell<Vec<Vec<String>>>,
}

impl Scripted
{
    fn New(lint_exit: i32, predicate_exit: i32) -> Self
    {
        return Self {
            lint_exit,
            predicate_exit,
            calls: RefCell::new(Vec::new()),
        };
    }

    fn Calls(&self) -> Vec<Vec<String>>
    {
        return self.calls.borrow().clone();
    }
}

impl ProcessLauncher for &Scripted
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        self.calls.borrow_mut().push(command.argv.clone());

        let is_lint = command.argv.iter().any(|argument| return argument == "clippy");
        let code = if is_lint { self.lint_exit } else { self.predicate_exit };

        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code },
            stdout: String::new(),
            stderr: "captured output".to_owned(),
        });
    }
}

fn Temp_Dir(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-gate-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// A tree with a claimed item whose predicate is a scoped `cargo test`, exactly as every
/// item on the real ledger carries.
fn Tree(name: &str, workflow: Option<&str>) -> PathBuf
{
    let directory = Temp_Dir(name);

    if let Some(text) = workflow
    {
        let workflows = directory.join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
        std::fs::write(workflows.join("gate.yml"), text).expect("test needs a workflow");
    }

    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    let mut item = LedgerItem {
        id: ItemId::New("T-1"),
        title: "an item".to_owned(),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        territory: Territory::Of_Files(["src/a.rs"]),
        state: ItemState::Claimed,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: Some(VerificationPredicate::New(vec![
            "cargo".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "nomos-ledger".to_owned(),
        ])),
        verified: None,
        abandoned: Vec::new(),
    };
    item.claim = Some(Claim {
        holder: HOLDER.to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(NOW),
        lease_expires_at: Timestamp::From_Unix_Seconds(NOW + 3_600),
    });

    ledger
        .Save(&LedgerDocument {
            schema_version: 1,
            items: vec![item],
        })
        .expect("test needs a ledger");

    return directory;
}

fn Ledger_At<'clock>(
    directory: &Path,
    clock: &'clock FixedClock,
) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

fn State_Of(directory: &Path) -> (ItemState, Option<nomos_ledger::VerificationRecord>)
{
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(directory, &clock);
    let document = ledger.Load().expect("the ledger is readable");
    let item = document
        .items
        .first()
        .cloned()
        .expect("the ledger has the item");

    return (item.state, item.verified);
}

// ---------------------------------------------------------------------------
// The instance. A passing predicate and a red gate must not finish an item.
// ---------------------------------------------------------------------------

#[test]
fn Test_A_Passing_Predicate_Should_Not_Finish_An_Item_While_The_Gate_Is_Red()
{
    let directory = Tree("red-gate", Some(WORKFLOW));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(101, 0);

    let refusal = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    )
    .expect_err("a red gate must refuse the finish");

    assert!(
        matches!(refusal, FinishRefusal::GateFailed { .. }),
        "expected GateFailed, got {}",
        refusal.Describe()
    );

    let (state, verified) = State_Of(&directory);
    assert_eq!(state, ItemState::Claimed, "the item must not have finished");
    assert!(
        verified.is_none(),
        "an item refused by the gate must carry no verification record"
    );
}

/// The negative control. Without it, a `GateFailed` that fired unconditionally would pass
/// the test above and nothing would ever be finishable.
#[test]
fn Test_A_Green_Gate_And_A_Passing_Predicate_Should_Finish_The_Item()
{
    let directory = Tree("green-gate", Some(WORKFLOW));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(0, 0);

    let record = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    )
    .expect("a green gate and a passing predicate finish the item");

    let gate = record
        .gate
        .as_ref()
        .expect("the record must say the gate ran");
    assert_eq!(gate.exit_code, 0);
    assert!(
        gate.argv.iter().any(|argument| return argument == "clippy"),
        "the recorded gate step must be the derived one, got {:?}",
        gate.argv
    );

    let (state, verified) = State_Of(&directory);
    assert_eq!(state, ItemState::Done);
    assert!(
        verified.and_then(|record| return record.gate).is_some(),
        "the ledger must record what the gate did, or a reader cannot tell an item \
         finished under the gate from one finished before it existed"
    );
}

// ---------------------------------------------------------------------------
// Ordering. The gate runs first and short-circuits.
// ---------------------------------------------------------------------------

#[test]
fn Test_The_Gate_Should_Run_Before_The_Predicate_And_Short_Circuit()
{
    let directory = Tree("ordering", Some(WORKFLOW));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(101, 0);

    let _ = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    );

    let calls = launcher.Calls();
    assert_eq!(
        calls.len(),
        1,
        "a red gate must stop before the predicate, so the author is not told their \
         tests passed in the same breath as being told they cannot land: {calls:?}"
    );
    assert!(calls.iter().flatten().any(|argument| return argument == "clippy"));
}

/// The control for the ordering test: when the gate is green both run, gate first.
#[test]
fn Test_A_Green_Gate_Should_Still_Run_The_Predicate_Second()
{
    let directory = Tree("ordering-green", Some(WORKFLOW));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(0, 0);

    let _ = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    );

    let calls = launcher.Calls();
    assert_eq!(calls.len(), 2, "both steps must run: {calls:?}");
    assert!(calls.first().is_some_and(|first| {
        return first.iter().any(|argument| return argument == "clippy");
    }));
    assert!(calls.get(1).is_some_and(|second| {
        return second.iter().any(|argument| return argument == "test");
    }));
}

// ---------------------------------------------------------------------------
// Unknown is not permission.
// ---------------------------------------------------------------------------

#[test]
fn Test_A_Missing_Workflow_Should_Refuse_Rather_Than_Finish_On_The_Predicate_Alone()
{
    let directory = Tree("no-workflow", None);
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(0, 0);

    let refusal = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    )
    .expect_err("an underived gate must refuse");

    assert!(
        matches!(
            refusal,
            FinishRefusal::GateUndetermined {
                cause: GateUnknown::Unreadable { .. },
                ..
            }
        ),
        "expected GateUndetermined, got {}",
        refusal.Describe()
    );
    assert!(
        !refusal.Judged_The_Work(),
        "nobody found out whether the work passes the gate, so this must not read as \
         the work being wrong"
    );
    assert!(
        launcher.Calls().is_empty(),
        "nothing should have been run once the gate could not be established"
    );

    let (state, _) = State_Of(&directory);
    assert_eq!(state, ItemState::Claimed);
}

/// A workflow whose lint step is a shell script cannot yield an argv without guessing.
#[test]
fn Test_A_Scripted_Gate_Step_Should_Refuse_Rather_Than_Be_Guessed_At()
{
    let scripted = WORKFLOW.replace(
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo clippy && cargo doc",
    );
    let directory = Tree("scripted", Some(&scripted));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(0, 0);

    let refusal = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    )
    .expect_err("a scripted gate step must refuse");

    assert!(matches!(
        refusal,
        FinishRefusal::GateUndetermined {
            cause: GateUnknown::NotASingleCommand { .. },
            ..
        }
    ));
}

// ---------------------------------------------------------------------------
// Derivation, not duplication.
// ---------------------------------------------------------------------------

/// The test that fails if somebody writes the clippy line into `nomos-ledger` as a
/// constant. A derived step follows the workflow; a copied one silently disagrees with it
/// the day the workflow changes, and two guards for one rule is how they come to disagree.
#[test]
fn Test_Changing_The_Workflow_Should_Change_What_Finish_Runs()
{
    let altered = WORKFLOW.replace("--all-targets", "--lib");
    let directory = Tree("derived", Some(&altered));
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let launcher = Scripted::New(0, 0);

    let record = Finish(
        &mut ledger,
        &&launcher,
        &ItemId::New("T-1"),
        HOLDER,
        Some(&directory),
    )
    .expect("the altered workflow still lints");

    let gate = record.gate.expect("the gate ran");
    assert!(
        gate.argv.contains(&"--lib".to_owned()),
        "finish must run what the workflow says, got {:?}",
        gate.argv
    );
    assert!(!gate.argv.contains(&"--all-targets".to_owned()));
}

// ---------------------------------------------------------------------------
// The repository's own gate is still derivable.
// ---------------------------------------------------------------------------

/// A guard on the real workflow rather than a fixture. If the repository's gate is
/// renamed or rewritten as a script, finishing anything stops working — and this test is
/// what says so, instead of the next author discovering it mid-finish.
#[test]
fn Test_This_Repository_Gate_Should_Still_Yield_A_Lint_Step()
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let workflow = std::fs::read_to_string(nomos_ledger::Workflow_Path(&root))
        .expect("this repository has a gate workflow");

    let argv = nomos_ledger::Derive_Step(&workflow, nomos_ledger::LINT_STEP)
        .expect("the repository's gate must still yield a lint step");

    assert_eq!(argv.first().map(String::as_str), Some("cargo"));
    assert!(argv.iter().any(|argument| return argument == "clippy"));
    assert!(
        argv.iter().any(|argument| return argument == "-D"),
        "the lint step must still deny warnings, got {argv:?}"
    );
}

/// `Duration` is used by the predicate's timeout; this keeps the import honest if the
/// fixture above ever stops constructing one.
#[test]
fn Test_A_Predicate_Should_Carry_A_Timeout()
{
    let predicate = VerificationPredicate::New(vec!["cargo".to_owned()]);

    assert!(Duration::from_secs(predicate.timeout_seconds) > Duration::ZERO);
}
