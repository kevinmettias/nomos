//! [`crate::Run`] against a real, temporary, disk-backed ledger built from
//! `nomos-platform-std` — the same platform `nomos-cli` chooses, proving the seam by using
//! it exactly the way a composition root does, rather than by mocking the platform out.
//!
//! `nomos-cli`'s own black-box suites (`tests/takeover_is_recorded.rs`,
//! `tests/abandon_is_readable.rs`, and the three under `tests/list_tells_the_truth/`) drive
//! every verb through the compiled binary, and now drive it through this crate along the
//! way. What is worth proving here, once, directly against [`Run`], is that the dispatch
//! itself reaches the right ledger call for a representative command from each shape —
//! a read (`list`, `show`, `validate`), a write that succeeds (`add`, `claim`), and a write
//! that ends an item (`decline`) — with nothing about `nomos-platform-std` baked into how it
//! got there.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use std::time::Duration;

use nomos_ledger::{
    ClaimRefusal, FileLedger, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory,
};
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};

use crate::{ClaimRequest, EndingRequest, Run, WorkCommand, WorkOutcome};

/// A ledger under a directory unique to this process and this test, removed by nobody —
/// the temporary root is cleaned by the OS, and the suites this crate keeps its shape
/// closest to (`crates/host/nomos-cli/tests/scratch_ledger/board.rs`) make the same choice
/// for the same reason: a test that failed mid-run should leave its board for a person to
/// read, not delete the evidence on its way out.
fn Scratch_Ledger(name: &str) -> FileLedger<StdFileSystem, SystemClock, FileLock>
{
    let root = std::env::temp_dir().join(format!(
        "nomos-work-orchestration-{name}-{}",
        std::process::id()
    ));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");

    return FileLedger::At(
        root.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(root.join("ledger.lock")),
    );
}

/// A minimal, valid, unclaimed item — enough to be added and then claimed.
///
/// Reserves one file rather than [`Territory::Empty`]: an item that reserves nothing
/// excludes nobody while looking like work, and `ledger.Add` refuses it for that reason.
fn Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "a test item".to_owned(),
        why: "exercising Run generically".to_owned(),
        done_when: "the assertion below passes".to_owned(),
        kind: ItemKind::Cleanup,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files([format!("scratch/{id}.rs")]),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        declined: None,
    };
}

/// No process is ever actually launched by the commands this file dispatches, so any
/// [`nomos_platform::ProcessLauncher`] would do; a launcher that panics if called is the one
/// that also proves it.
struct Unreached;

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Unreached
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl nomos_platform::ProcessLauncher for Unreached
{
    fn Run(&self, _command: &nomos_platform::Command) -> Result<nomos_platform::ProcessOutput, String>
    {
        panic!("no command dispatched by this suite should run a process");
    }
}

/// Adds `item` to `ledger` through [`Run`], exactly the way `nomos work add` would, and
/// hands `item` back so a test can claim, decline or validate against it next.
///
/// Discards the [`WorkOutcome::Add`] itself: the tests reusing this fixture (`Claim_Then_
/// Second_Claim`, `Decline_Should_End_An_Unclaimed_Item`) are proving what happens *after*
/// the add, not the add itself -- `Test_Add_Then_Show_Should_Find_What_Add_Wrote` already
/// proves `Add`'s own outcome and keeps checking it inline.
fn Added(ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>, item: LedgerItem) -> LedgerItem
{
    let _ = Run(
        &WorkCommand::Add {
            item: Box::new(item.clone()),
            amending: Territory::Empty(),
        },
        ledger,
        &Unreached,
        Territory::Empty,
    );

    return item;
}

/// Claims `item` as `holder`, through [`Run`] the way `nomos work claim` would.
fn Claimed(ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>, item: &ItemId, holder: &str) -> WorkOutcome
{
    let request = ClaimRequest {
        item: item.clone(),
        holder: holder.to_owned(),
        lease: Duration::from_secs(60),
    };

    return Run(&WorkCommand::Claim(request), ledger, &Unreached, Territory::Empty);
}

/// `item`, read back off `ledger` through [`Run`]'s own `audit` verb -- the same read
/// `nomos work audit` would perform.
fn On_Board(ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>, item: &ItemId) -> LedgerItem
{
    let audited = Run(&WorkCommand::Audit, ledger, &Unreached, Territory::Empty);
    let WorkOutcome::Audit(Ok(view)) = audited
    else
    {
        panic!("an auditable board must answer WorkOutcome::Audit");
    };

    return view
        .document
        .items
        .into_iter()
        .find(|found| found.id == *item)
        .expect("the item add wrote is still on the board");
}

#[test]
fn Test_List_Should_Read_An_Empty_Board()
{
    let mut ledger = Scratch_Ledger("list-empty");

    let outcome = Run(
        &WorkCommand::List { state: None },
        &mut ledger,
        &Unreached,
        Territory::Empty,
    );

    let WorkOutcome::List(Ok(view)) = outcome
    else
    {
        panic!("an unwritten ledger loads as an empty, valid board");
    };
    assert!(view.document.items.is_empty());
}

#[test]
fn Test_Add_Then_Show_Should_Find_What_Add_Wrote()
{
    let mut ledger = Scratch_Ledger("add-show");
    let item = Item("T-ONE");

    let added = Run(
        &WorkCommand::Add {
            item: Box::new(item.clone()),
            amending: Territory::Empty(),
        },
        &mut ledger,
        &Unreached,
        Territory::Empty,
    );
    assert!(matches!(added, WorkOutcome::Add(Ok(()))));

    let shown = Run(&WorkCommand::Show { item: item.id.clone() }, &mut ledger, &Unreached, Territory::Empty);
    let WorkOutcome::Show(Ok(view)) = shown
    else
    {
        panic!("the board just written must be readable");
    };
    assert!(view.document.items.iter().any(|found| found.id == item.id));
}

#[test]
fn Test_Claim_Then_Second_Claim_Should_Be_Refused_As_Held()
{
    let mut ledger = Scratch_Ledger("claim-twice");
    let item = Added(&mut ledger, Item("T-TWO"));

    let first = Claimed(&mut ledger, &item.id, "agent-a");
    assert!(matches!(first, WorkOutcome::Claim(Ok(_))));

    let second = Claimed(&mut ledger, &item.id, "agent-b");
    let WorkOutcome::Claim(Err(refusal)) = second
    else
    {
        panic!("a second claimant must be refused");
    };
    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "expected NotClaimable, got {refusal:?}"
    );
}

#[test]
fn Test_Validate_Should_Accept_A_Board_This_Run_Wrote()
{
    let mut ledger = Scratch_Ledger("validate");
    let item = Item("T-THREE");
    let _ = Run(
        &WorkCommand::Add {
            item: Box::new(item),
            amending: Territory::Empty(),
        },
        &mut ledger,
        &Unreached,
        Territory::Empty,
    );

    let validated = Run(&WorkCommand::Validate, &mut ledger, &Unreached, Territory::Empty);
    assert!(matches!(validated, WorkOutcome::Validate(Ok(_))));
}

#[test]
fn Test_Decline_Should_End_An_Unclaimed_Item()
{
    let mut ledger = Scratch_Ledger("decline");
    let item = Added(&mut ledger, Item("T-FOUR"));

    let declined = Run(
        &WorkCommand::Decline(EndingRequest {
            item: item.id.clone(),
            holder: "nomos work decline".to_owned(),
            reason: "not work".to_owned(),
        }),
        &mut ledger,
        &Unreached,
        Territory::Empty,
    );
    assert!(matches!(declined, WorkOutcome::Decline(Ok(()))));

    let found = On_Board(&mut ledger, &item.id);
    assert!(found.declined.is_some());
}
