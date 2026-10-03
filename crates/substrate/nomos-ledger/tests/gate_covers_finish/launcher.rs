//! The apparatus every test here shares: a repository with a board, a ledger over it, and
//! a launcher whose exit codes are the thing each test actually varies.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_ledger::{
    Finishing,
    Claim, FileLedger, Finish_Item, FinishRefusal, ItemId, ItemKind, ItemOrigin, ItemState,
    LedgerDocument, LedgerItem, Territory, VerificationPredicate, VerificationRecord,
};
use nomos_platform::{
    Clock, Command, ExitOutcome, ProgramLauncher, ProgramOutput, Timestamp,
};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

const NOW: i64 = 1_000_000;
const HOLDER: &str = "agent-a";

/// The lease the claimed item on every constructed board carries.
///
/// Two hours, matching `DEFAULT_LEASE`. Every test here finishes the item long before it
/// lapses, and none of them moves the clock, so the exact figure is only ever read.
const LEASE_SECONDS: i64 = 3_600;

/// The exit code a scripted gate step reports when the gate is red.
///
/// `cargo clippy` exits 101 when the build it drives fails, so a scripted step answering 101
/// is the real shape of a red gate rather than an arbitrary non-zero. Nothing in these tests
/// reads the number itself; what they read is that it is not zero.
pub(crate) const GATE_FAILED_EXIT: i32 = 101;

/// The workflow a derived gate step is read out of. Deliberately the real shape, so that
/// a change to the repository's own gate breaks these tests rather than passing them.
pub(crate) const WORKFLOW: &str = concat!(
    "name: gate\n",
    "\n",
    "jobs:\n",
    "  gate:\n",
    "    steps:\n",
    "      - uses: actions/checkout@v4\n",
    "      - name: Lint\n",
    "        run: cargo clippy --workspace --all-targets -- -D warnings\n",
    "      - name: Test\n",
    "        run: cargo test --workspace\n",
    "      - name: Rules\n",
    "        if: matrix.os == 'ubuntu-latest'\n",
    "        run: cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .\n",
);

/// The `run:` line of [`WORKFLOW`]'s `Rules` step, for the cases that rewrite it.
pub(crate) const RULES_RUN: &str = "cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .";

/// A workflow that declares no `Rules` step: KWB's, whose gate declares `Lint`, `Test` and
/// `Contract`. The same ledger serves it, and `OD-GATE-036` says a finish there runs no rules
/// step and records that it was not declared.
pub(crate) const WORKFLOW_WITHOUT_RULES: &str = concat!(
    "name: gate\n",
    "\n",
    "jobs:\n",
    "  gate:\n",
    "    steps:\n",
    "      - uses: actions/checkout@v4\n",
    "      - name: Lint\n",
    "        run: cargo clippy --workspace --all-targets -- -D warnings\n",
    "      - name: Test\n",
    "        run: cargo test --workspace --no-fail-fast\n",
    "      - name: Contract\n",
    "        run: cargo test -p kwb-contract-tests --no-fail-fast\n",
);

/// What the scripted `Rules` step prints: the shape of the one Blocking finding
/// `OD-GATE-036` reintroduced at `0000e0e3` to measure that the step catches it.
pub(crate) const RULES_OUTPUT: &str =
    "Blocking  abbreviations  crates/orchestration/nomos-gate-orchestration/src/graph.rs  Assert_Well_Formed_GraphML";

pub(crate) struct FixedClock(i64);

/// Fixed instants, so both the values and their timing reproduce.
impl Strategy for FixedClock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

/// A launcher that answers by argv rather than by running anything.
///
/// The point of these tests is the ordering and the arithmetic of three exit codes, not
/// whether clippy or the rule layer works. Running either for real here would make the suite
/// slow and would couple it to whatever the workspace happens to contain.
pub(crate) struct Scripted
{
    lint_exit: i32,
    rules_exit: i32,
    predicate_exit: i32,
    commands: RefCell<Vec<Command>>,
}

impl Scripted
{
    /// A launcher whose `Rules` step, when one runs, exits zero.
    pub(crate) fn New(lint_exit: i32, predicate_exit: i32) -> Self
    {
        return Self {
            lint_exit,
            rules_exit: 0,
            predicate_exit,
            commands: RefCell::new(Vec::new()),
        };
    }

    /// The same launcher, with its `Rules` step exiting `code`.
    pub(crate) fn With_Rules_Exit(self, code: i32) -> Self
    {
        return Self { rules_exit: code, ..self };
    }

    /// The argv of every command run, in the order it ran.
    pub(crate) fn Calls(&self) -> Vec<Vec<String>>
    {
        return self.commands.borrow().iter().map(|command| return command.argv.clone()).collect();
    }

    /// Every command run, whole, for the cases about how a step was bounded.
    pub(crate) fn Commands(&self) -> Vec<Command>
    {
        return self.commands.borrow().clone();
    }
}

/// Whether an argv is the `Rules` step's: the one command here that runs `gate run`.
pub(crate) fn Is_Rules_Step(argv: &[String]) -> bool
{
    return argv.iter().any(|argument| return argument == "gate");
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for &Scripted
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        self.commands.borrow_mut().push(command.clone());

        let is_lint = command.argv.iter().any(|argument| return argument == "clippy");
        let is_rules = Is_Rules_Step(&command.argv);
        let code = if is_lint { self.lint_exit } else if is_rules { self.rules_exit } else { self.predicate_exit };
        let stdout = if is_rules { RULES_OUTPUT } else { "" };

        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code },
            stdout: stdout.to_owned(),
            stderr: "captured output".to_owned(),
        });
    }
}

fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-gate-{name}-{}", std::process::id()));

    // A process id that repeats finds the previous run's tree still here. Nothing to clear
    // is the ordinary case and is not worth a word; anything else means a stale tree is
    // about to be read as this run's own.
    match std::fs::remove_dir_all(&path)
    {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => eprintln!(
            "a leftover scratch directory at {} could not be cleared: {error}",
            path.display()
        ),
    }

    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// A tree with a claimed item whose predicate is a scoped `cargo test`, exactly as every
/// item on the real ledger carries.
fn Workflow_Tree_On_Disk(name: &str, workflow: Option<&str>) -> PathBuf
{
    let directory = Temporary_Directory(name);
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
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(["src/a.rs"]),
        state: ItemState::Claimed,
        depends_on: Vec::new(),
        blocked: None,
        claim: Some(Claim {
            holder: HOLDER.to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(NOW),
            lease_expires_at: Timestamp::From_Unix_Seconds(NOW + LEASE_SECONDS),
        }),
        verification: Some(VerificationPredicate::From_String_Arguments(vec![
            "cargo".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "nomos-ledger".to_owned(),
        ])),
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
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
    pub(crate) ledger: BenchLedger,
    pub(crate) launcher: Scripted,
}

/// The ledger every bench finishes through.
pub(crate) type BenchLedger = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// Removes `key` from every verification record in the bench's ledger file.
///
/// A record written before a field existed has no key at all, which no `Save` of the current
/// type can produce: it writes `null` for an absent value. Editing the file is the only way to
/// put the older shape on disk, and the older shape is the one the real board holds.
pub(crate) fn Strip_Key_From_Ledger(directory: &Path, key: &str)
{
    let path = directory.join("ledger.json");
    let text = std::fs::read_to_string(&path).expect("the bench's ledger is readable");
    let mut document: serde_json::Value = serde_json::from_str(&text).expect("the bench's ledger is JSON");
    let items = document.get_mut("items").and_then(serde_json::Value::as_array_mut).expect("a ledger holds items");

    for item in items
    {
        if let Some(record) = item.get_mut("verified").and_then(serde_json::Value::as_object_mut)
        {
            record.remove(key);
        }
    }

    std::fs::write(&path, serde_json::to_string_pretty(&document).expect("JSON serializes"))
        .expect("test needs to rewrite its ledger");
}

/// The clock these tests share. `'static` so the ledger [`Bench_At`] returns can outlive it.
static AT_NOW: FixedClock = FixedClock(NOW);

/// A repository with a board, and a launcher scripted to answer the gate and the predicate.
pub(crate) fn Bench_At(name: &str, workflow: Option<&str>, launcher: Scripted) -> Bench
{
    let directory = Workflow_Tree_On_Disk(name, workflow);
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
    return Finish_Item(
        ledger,
        &launcher,
        &Finishing {
            item: &ItemId::New(item),
            holder: HOLDER,
        },
        Some(directory),
    );
}
