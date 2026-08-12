//! The apparatus every test here shares: a repository with a board, a ledger over it, and
//! a launcher whose exit codes are the thing each test actually varies.

use nomos_ledger::{
    Finishing,
    Claim, FileLedger, Finish, FinishRefusal, ItemId, ItemState,
    LedgerDocument, LedgerItem, Territory, VerificationPredicate, VerificationRecord,
};
use nomos_platform::{
    Clock, Command, ExitOutcome, ProcessLauncher, ProcessOutput, Timestamp,
};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

const NOW: i64 = 1_000_000;
const HOLDER: &str = "agent-a";

/// The workflow a derived gate step is read out of. Deliberately the real shape, so that
/// a change to the repository's own gate breaks these tests rather than passing them.
pub(crate) const WORKFLOW: &str = "name: gate\n\
                        \n\
                        jobs:\n\
                        \x20 gate:\n\
                        \x20   steps:\n\
                        \x20     - uses: actions/checkout@v4\n\
                        \x20     - name: Lint\n\
                        \x20       run: cargo clippy --workspace --all-targets -- -D warnings\n\
                        \x20     - name: Test\n\
                        \x20       run: cargo test --workspace\n";

pub(crate) struct FixedClock(i64);

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
pub(crate) struct Scripted
{
    lint_exit: i32,
    predicate_exit: i32,
    calls: RefCell<Vec<Vec<String>>>,
}

impl Scripted
{
    pub(crate) fn New(lint_exit: i32, predicate_exit: i32) -> Self
    {
        return Self {
            lint_exit,
            predicate_exit,
            calls: RefCell::new(Vec::new()),
        };
    }

    pub(crate) fn Calls(&self) -> Vec<Vec<String>>
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
        Write_The_Workflow(&directory, text);
    }

    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);
    ledger
        .Save(&LedgerDocument {
            schema_version: 1,
            items: vec![A_Claimed_Item()],
        })
        .expect("test needs a ledger");

    return directory;
}

/// The workflow the gate derives its step from, where the test declares one.
fn Write_The_Workflow(directory: &Path, text: &str)
{
    let workflows = directory.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
    std::fs::write(workflows.join("gate.yml"), text).expect("test needs a workflow");
}

/// A claimed item whose predicate is a scoped `cargo test`, exactly as every item on the real
/// ledger carries.
fn A_Claimed_Item() -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New("T-1"),
        title: "an item".to_owned(),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        territory: Territory::Of_Files(["src/a.rs"]),
        state: ItemState::Claimed,
        depends_on: Vec::new(),
        blocked: None,
        claim: Some(Claim {
            holder: HOLDER.to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(NOW),
            lease_expires_at: Timestamp::From_Unix_Seconds(NOW + 3_600),
        }),
        verification: Some(VerificationPredicate::New(vec![
            "cargo".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "nomos-ledger".to_owned(),
        ])),
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        declined: None,
    };
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

/// What the item on disk says about itself.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
pub(crate) struct Standing
{
    pub(crate) state: ItemState,
    pub(crate) verified: Option<nomos_ledger::VerificationRecord>,
}

pub(crate) fn State_Of(directory: &Path) -> Standing
{
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(directory, &clock);
    let document = ledger.Load().expect("the ledger is readable");
    let item = document
        .items
        .first()
        .cloned()
        .expect("the ledger has the item");

    return Standing {
        state: item.state,
        verified: item.verified,
    };
}

/// One test's whole apparatus: a repository, a ledger over it, and the launcher whose exit
/// codes are what the test is actually varying.
///
/// Held together because they are built together and every test needs all three — the
/// directory to point the gate at, the ledger to finish through, the launcher to script.
pub(crate) struct Bench
{
    pub(crate) directory: PathBuf,
    pub(crate) ledger: FileLedger<StdFileSystem, &'static FixedClock, FileLock>,
    pub(crate) launcher: Scripted,
}

/// The clock these tests share. `'static` so the ledger [`Bench_At`] returns can outlive it.
static AT_NOW: FixedClock = FixedClock(NOW);

/// A repository with a board, and a launcher scripted to answer the gate and the predicate.
pub(crate) fn Bench_At(name: &str, workflow: Option<&str>, launcher: Scripted) -> Bench
{
    let directory = Tree(name, workflow);
    let ledger = Ledger_At(&directory, &AT_NOW);

    return Bench {
        directory,
        ledger,
        launcher,
    };
}

/// Runs `T-1`'s verification through the ledger against a scripted launcher.
pub(crate) fn Finish_In(
    ledger: &mut FileLedger<StdFileSystem, &'static FixedClock, FileLock>,
    directory: &Path,
    launcher: &Scripted,
    item: &str,
) -> Result<VerificationRecord, FinishRefusal>
{
    return Finish(
        ledger,
        &launcher,
        &Finishing {
            item: &ItemId::New(item),
            holder: HOLDER,
        },
        Some(directory),
    );
}
