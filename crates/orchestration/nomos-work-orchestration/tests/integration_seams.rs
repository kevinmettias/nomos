//! The seams `nomos_work_orchestration` reaches into `nomos_ledger` and `nomos_platform`
//! (`nomos_platform_std` is one real implementation of the latter's traits), exercised from
//! outside the crate through public types only.
//!
//! `nomos-work-orchestration`'s own internal `#[cfg(test)] mod tests` already drives these
//! seams, but only through private-item access; it proves the crate works when opened up, not
//! that the PUBLIC contract a real second adapter depends on holds. This file drives the same
//! calls -- `Run` against a real, disk-backed `nomos_ledger::FileLedger` built from
//! `nomos-platform-std`, exactly the way `nomos-cli`'s own composition root does -- compiled
//! as a separate crate that can reach nothing but `nomos_work_orchestration`'s public API.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use std::time::Duration;

use nomos_ledger::{ClaimRefusal, FileLedger, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};
use nomos_work_orchestration::{ClaimRequest, EndingRequest, Run, WorkCommand, WorkOutcome};

/// The lease this suite's claims ask for: far longer than any test here could run, so an
/// expired lease is never the reason a seam assertion failed.
const LEASE_SECONDS: u64 = 60;

/// A ledger under a directory unique to this process and this test, left for a person to read
/// if a test fails mid-run -- the OS cleans the temp root, not this suite.
fn Scratch_Ledger(name: &str) -> FileLedger<StdFileSystem, SystemClock, FileLock>
{
    let root = std::env::temp_dir().join(format!("nomos-work-orchestration-seam-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");

    return FileLedger::At(root.join("ledger.json"), StdFileSystem, SystemClock, FileLock::At(root.join("ledger.lock")));
}

/// No command dispatched by this suite launches a real process, so a launcher that panics if
/// called both satisfies the type and proves that claim.
struct Unreached;

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Unreached
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl nomos_platform::ProgramLauncher for Unreached
{
    fn Run(&self, _command: &nomos_platform::Command) -> Result<nomos_platform::ProgramOutput, String>
    {
        panic!("no command dispatched by this suite should run a process");
    }
}

fn Unclaimed_Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "a seam test item".to_owned(),
        why: "exercising nomos_work_orchestration against a real nomos_ledger/nomos_platform_std pair".to_owned(),
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
        widened: Vec::new(),
        declined: None,
    };
}

/// Adds `item` through [`Run`], asserting the real ledger took it, so each test below starts
/// from an item that is genuinely on the board rather than one whose add was dropped on the
/// floor.
fn Added_To_Ledger(ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>, item: &LedgerItem)
{
    let added = Run(
        &WorkCommand::Add {
            item: Box::new(item.clone()),
            amending: Territory::Empty(),
        },
        ledger,
        &Unreached,
        Territory::Empty,
    );

    assert!(matches!(added, WorkOutcome::Add(Ok(()))), "a real ledger accepts an add for a fresh item");
}

/// The happy path across the boundary: an item added through a real `FileLedger`, then read
/// back through the same seam.
#[test]
fn Test_Run_Should_Add_Then_Show_An_Item_Through_A_Real_Ledger()
{
    let mut ledger = Scratch_Ledger("add-show");
    let item = Unclaimed_Item("SEAM-ONE");
    Added_To_Ledger(&mut ledger, &item);

    let shown = Run(&WorkCommand::Show { item: item.id.clone() }, &mut ledger, &Unreached, Territory::Empty);
    let WorkOutcome::Show(Ok(view)) = shown
    else
    {
        panic!("the board just written through the real ledger must be readable");
    };
    assert!(view.document.items.iter().any(|found| return found.id == item.id));
}

/// An error that crosses the boundary: `nomos_ledger::ClaimRefusal` reaching this crate's own
/// `WorkOutcome::Claim`, unmodified, when a second claimant contends for an already-held item.
#[test]
fn Test_Run_Should_Surface_A_Real_Ledger_Claim_Refusal()
{
    let mut ledger = Scratch_Ledger("claim-refusal");
    let item = Unclaimed_Item("SEAM-TWO");
    Added_To_Ledger(&mut ledger, &item);

    let request = ClaimRequest { item: item.id.clone(), holder: "agent-a".to_owned(), lease: Duration::from_secs(LEASE_SECONDS) };
    let first = Run(&WorkCommand::Claim(request), &mut ledger, &Unreached, Territory::Empty);
    assert!(matches!(first, WorkOutcome::Claim(Ok(_))));

    let request = ClaimRequest { item: item.id, holder: "agent-b".to_owned(), lease: Duration::from_secs(LEASE_SECONDS) };
    let second = Run(&WorkCommand::Claim(request), &mut ledger, &Unreached, Territory::Empty);
    let WorkOutcome::Claim(Err(refusal)) = second
    else
    {
        panic!("a second claimant must be refused");
    };
    assert!(matches!(refusal, ClaimRefusal::NotClaimable { .. }), "expected NotClaimable, got {refusal:?}");
}

/// The lifecycle `Run` imposes on the ledger it is handed: `Decline` reads the board, mutates
/// it, and the change is visible to a later `Audit` over the SAME `FileLedger` value --
/// proving this crate does not hold its own hidden copy of what `nomos_ledger` persists.
#[test]
fn Test_Run_Should_Make_A_Decline_Visible_To_A_Later_Audit()
{
    let mut ledger = Scratch_Ledger("decline-audit");
    let item = Unclaimed_Item("SEAM-THREE");
    Added_To_Ledger(&mut ledger, &item);

    let declined = Run(
        &WorkCommand::Decline(EndingRequest { item: item.id.clone(), holder: "nomos work decline".to_owned(), reason: "not work".to_owned() }),
        &mut ledger,
        &Unreached,
        Territory::Empty,
    );
    assert!(matches!(declined, WorkOutcome::Decline { declined: Ok(()), board: Some(_) }),
        "a decline that ended an item carries the board its fanout is read from");

    let audited = Run(&WorkCommand::Audit, &mut ledger, &Unreached, Territory::Empty);
    let WorkOutcome::Audit(Ok(view)) = audited
    else
    {
        panic!("an auditable board must answer WorkOutcome::Audit");
    };
    let found = view.document.items.into_iter().find(|found| return found.id == item.id).expect("the item add wrote is still on the board");
    assert!(found.declined.is_some(), "the decline made through Run must be visible through a later Run(Audit, ..)");
}
