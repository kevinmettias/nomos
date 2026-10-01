//! The seam between `nomos_ledger` and `nomos_scope_verification`.
//!
//! `nomos_ledger` does not declare [`Territory`] or [`VerificationPredicate`] itself:
//! `OD-LEDGER-037` found both ledger-agnostic in their real field shapes and moved them,
//! unchanged, into `nomos-scope-verification`, and `nomos_ledger`'s `lib.rs` and
//! `verification.rs` re-export them for its own claim/overlap and finish logic. Every test in
//! this crate that builds a [`Territory`] or a [`VerificationPredicate`] already exercises the
//! two *separately* -- `nomos_ledger`'s own logic against whatever answer the type happens to
//! give back. Nothing exercised the contract itself: that `nomos_scope_verification`'s
//! `Intersection::Unknown` is honoured as a refusal rather than a grant, that its
//! `Is_Runnable` is consulted before a predicate is ever launched, and that its `Is_Empty`
//! is what `Add` refuses a workable item over. This file is that suite, built directly against
//! `nomos_scope_verification`'s own types -- passed straight into `nomos_ledger`'s public API,
//! so a fork between the two would fail to compile before it ever failed to assert.
//!
//! Every test drives `nomos_scope_verification` through `nomos_ledger`'s public surface:
//! [`FileLedger::Add`], [`ExclusionLedger::Claim`] and [`Finish_Item`].

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_ledger::{
    ExclusionLedger, FileLedger, Finish_Item, Finishing, FinishRefusal, ItemId, ItemKind,
    ItemOrigin, ItemState, LedgerDocument, LedgerItem, ClaimRefusal, AddRefusal, SCHEMA_VERSION,
};
use nomos_platform::{Clock, Command, ExitOutcome, ProgramLauncher, ProgramOutput, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use nomos_scope_verification::{Territory, VerificationPredicate};
use std::path::{Path, PathBuf};
use std::time::Duration;

struct FixedClock(i64);

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

const NOW: i64 = 1_000_000;

/// The one-hour lease every claim in this suite takes.
///
/// One home, because the expiry a fixture records and the lease the claim verbs are given are
/// the same hour: written twice, they would be two values a later edit could pull apart.
const LEASE: Duration = Duration::from_secs(3_600);

/// The workflow a derived gate step is read out of -- the real shape, so the happy-path test
/// below exercises `Finish_Item`'s gate step exactly as the repository's own gate would.
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

/// The ledger every test here builds, named once so the helpers below can take it.
type Board<'clock> = FileLedger<StdFileSystem, &'clock FixedClock, FileLock>;

/// A ledger on a fresh temporary directory, and the directory it lives on.
struct SeamBoard<'clock>
{
    directory: PathBuf,
    ledger: Board<'clock>,
    /// The predicate the one item was authored with, so a test can compare what actually ran
    /// against what it asked for.
    predicate: VerificationPredicate,
}

fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-ledger-scope-verification-{name}-{}", std::process::id()));
    Remove_Scratch(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// Removes a scratch directory this test made, reporting a failure rather than discarding it.
///
/// A directory that outlives its test is one the next run inherits, and a failed removal is
/// the only thing that would have said so -- which is why the failure is printed rather than
/// dropped.
fn Remove_Scratch(directory: &Path)
{
    if let Err(error) = std::fs::remove_dir_all(directory)
    {
        eprintln!("could not remove the scratch directory {}: {error}", directory.display());
    }
}

fn Write_Workflow(directory: &Path)
{
    let workflows = directory.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
    std::fs::write(workflows.join("gate.yml"), WORKFLOW).expect("test needs a workflow");
}

fn Ledger_At<'clock>(directory: &Path, clock: &'clock FixedClock) -> Board<'clock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

/// Writes `document` straight to the board, past the validation `Save` performs.
///
/// The documents written this way are the ones an authoring verb would reject -- an unrunnable
/// predicate, an unexpanded pattern -- and they arrive the way a hand edit or a file written
/// before that guard existed would: as text, on disk, for `Finish_Item` and `Claim` to answer
/// honestly about rather than to trust.
fn Write_Straight_To_The_Board(ledger: &Board<'_>, document: &LedgerDocument)
{
    let raw = serde_json::to_string(document).expect("the fixture document serializes");
    std::fs::write(ledger.Path(), raw).expect("test can write the raw fixture directly");
}

/// A workable item over a `nomos_scope_verification::Territory`, carrying a
/// `nomos_scope_verification::VerificationPredicate` the caller supplies.
fn Item_Reserving(id: &str, territory: Territory, verification: Option<VerificationPredicate>) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "an item".to_owned(),
        why: "it needs doing".to_owned(),
        done_when: "the predicate passes".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory,
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}

/// A workflow, a ledger, and one item added over `territory` with `predicate` as the
/// verification it will later be finished against.
fn Board_Over<'clock>(
    name: &str,
    clock: &'clock FixedClock,
    territory: Territory,
    predicate: VerificationPredicate,
) -> SeamBoard<'clock>
{
    let directory = Temporary_Directory(name);
    Write_Workflow(&directory);
    let mut ledger = Ledger_At(&directory, clock);
    let item = Item_Reserving("SEAM-1", territory, Some(predicate.clone()));
    ledger
        .Add(&item, "agent-a", &Territory::Empty(), &Territory::Empty())
        .expect("a fresh item over a real territory must be accepted");

    return SeamBoard { directory, ledger, predicate };
}

/// A workflow, a ledger, and one claimed item whose predicate cannot be run.
///
/// `Validate_Document` itself refuses an unrunnable predicate, so `Add` and `Save` would refuse
/// this document outright. It is written directly instead, the way an item authored before that
/// guard existed would still read today -- exactly the document `Finish_Item` still has to
/// answer honestly about rather than trust.
fn Board_With_An_Unrunnable_Predicate<'clock>(clock: &'clock FixedClock) -> SeamBoard<'clock>
{
    let directory = Temporary_Directory("unrunnable");
    Write_Workflow(&directory);
    let ledger = Ledger_At(&directory, clock);
    let predicate = VerificationPredicate::From_String_Arguments(Vec::new());
    let mut item = Item_Reserving("SEAM-2", Territory::Of_Files(["src/other.rs"]), Some(predicate.clone()));
    item.state = ItemState::Claimed;
    item.claim = Some(nomos_ledger::Claim {
        holder: "agent-a".to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(NOW),
        lease_expires_at: Timestamp::From_Unix_Seconds(NOW).Plus(LEASE),
    });
    Write_Straight_To_The_Board(
        &ledger,
        &LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![item] },
    );

    return SeamBoard { directory, ledger, predicate };
}

/// Puts `item` on the board by hand and claims it for `holder`, handing back the refusal.
///
/// `Save` would refuse to write this document for the reason `Add` refused the same item, so it
/// goes to the file directly rather than through a verb -- and the claim is what the helper
/// exists to reach, because a board can hold what no authoring verb would have accepted.
fn Claim_An_Item_Written_By_Hand(
    ledger: &mut Board<'_>,
    item: LedgerItem,
    holder: &str,
) -> ClaimRefusal
{
    let id = item.id.clone();
    let mut document = ledger.Load().expect("the ledger is readable");
    document.items.push(item);
    Write_Straight_To_The_Board(ledger, &document);

    return ledger
        .Claim(&id, holder, LEASE)
        .expect_err("an item whose independence is unknown can never be claimed");
}

/// A launcher that always exits zero, standing in for a lint step and a predicate that both
/// pass -- what is under test is that the predicate crosses the boundary and gets run at all,
/// not what a real `cargo test` reports.
struct AlwaysZero;

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for AlwaysZero
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for &AlwaysZero
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: "all good".to_owned(),
            stderr: String::new(),
        });
    }
}

/// The happy path: a `Territory` built by `nomos_scope_verification` decides exclusion through
/// `Claim`, and a `VerificationPredicate` built by it is the thing `Finish_Item` actually runs
/// and records. Both types cross the boundary and come back out the other side as a closed,
/// verified item -- not merely as values nomos_ledger stored without reading.
#[test]
fn Test_A_Territory_And_A_Predicate_Cross_The_Boundary_On_The_Happy_Path()
{
    let clock = FixedClock(NOW);
    let mut board = Board_Over(
        "happy-path",
        &clock,
        Territory::Of_Files(["src/widget.rs"]),
        VerificationPredicate::From_String_Arguments(vec!["a-predicate".to_owned()]),
    );

    let reservation = board
        .ledger
        .Claim(&ItemId::New("SEAM-1"), "agent-a", LEASE)
        .expect("an uncontended territory must be claimable");
    assert_eq!(reservation.holder, "agent-a");
    let launcher = AlwaysZero;
    let record = Finish_Item(
        &mut board.ledger,
        &&launcher,
        &Finishing { item: &ItemId::New("SEAM-1"), holder: "agent-a" },
        Some(&board.directory),
    )
    .expect("a zero-exit predicate behind a green gate must finish");
    assert_eq!(record.argv, board.predicate.argv, "the predicate that actually ran must be the one supplied");
    let reloaded = board.ledger.Load().expect("the release must have been written");
    let closed = reloaded.items.first().expect("the item survives finishing");
    assert_eq!(closed.state, ItemState::Done);
}

/// A `VerificationPredicate`'s own judgment of itself -- `Is_Runnable`, computed entirely
/// inside `nomos_scope_verification` -- must be consulted before `nomos_ledger` ever launches
/// anything. An empty argv is a predicate in name only, and finishing must refuse it as a
/// broken predicate rather than attempt to run it.
#[test]
fn Test_An_Unrunnable_Predicate_Refuses_Rather_Than_Reaching_The_Launcher()
{
    let clock = FixedClock(NOW);
    let mut board = Board_With_An_Unrunnable_Predicate(&clock);
    let launcher = AlwaysZero;
    let refusal = Finish_Item(
        &mut board.ledger,
        &&launcher,
        &Finishing { item: &ItemId::New("SEAM-2"), holder: "agent-a" },
        Some(&board.directory),
    )
    .expect_err("an empty argv is not a predicate the launcher can be asked to run");

    assert!(
        matches!(refusal, FinishRefusal::CouldNotRun { .. }),
        "an unrunnable predicate must refuse as a broken predicate, not run: {refusal:?}"
    );
}

/// A territory `nomos_scope_verification::Territory::Intersect` cannot compare -- here, one
/// side carrying an unexpanded pattern -- must come back out of `nomos_ledger`'s own claim
/// logic as a refusal, never as permission. `Refusal_From` is the one place that rule is
/// applied on the `nomos_ledger` side; this is the boundary proof that `Intersection::Unknown`
/// actually reaches it rather than being read as `Disjoint` somewhere on the way.
#[test]
fn Test_An_Unknown_Intersection_Refuses_A_Claim_Rather_Than_Granting_It()
{
    let directory = Temporary_Directory("unknown-intersection");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    let holder = Item_Reserving("SEAM-3A", Territory::Of_Files(["src/shared.rs"]), None);
    // An unexpanded pattern: `nomos_scope_verification::Territory` records it rather than
    // rejecting it, and every comparison touching one answers `Intersection::Unknown` --
    // OD-LEDGER-013 is why nothing on the authoring side ever produces one, and this is the
    // fail-closed guard that still has to hold if a hand-edited document ever carries one.
    let contesting = Item_Reserving("SEAM-3B", Territory::Empty().With_Pattern("src/**"), None);
    ledger
        .Add(&holder, "agent-a", &Territory::Empty(), &Territory::Empty())
        .expect("the first item is a fresh, valid identifier");
    ledger
        .Claim(&ItemId::New("SEAM-3A"), "agent-a", LEASE)
        .expect("the first item's own territory is uncontended");
    let refusal = ledger
        .Add(&contesting, "agent-b", &Territory::Empty(), &Territory::Empty())
        .expect_err("an item carrying an unexpanded pattern violates Validate_Document's own guard");
    assert!(matches!(refusal, AddRefusal::WouldBeInvalid { .. }), "got {refusal:?}");

    // The same fact, reached the other way: a document that already carries one (as this one
    // now would if written by hand) must refuse a claim on it rather than grant one, because
    // `Territory::Intersect` cannot answer `Disjoint` for it. `Save` would refuse writing this
    // document for the same reason `Add` just did, so the claim below is what this test is
    // about, reached through an item only a hand edit could have put on the board.
    let claim_refusal = Claim_An_Item_Written_By_Hand(&mut ledger, contesting, "agent-c");
    assert!(
        matches!(claim_refusal, ClaimRefusal::UnknownIndependence { .. }),
        "unknown independence must refuse, never grant: {claim_refusal:?}"
    );
}
