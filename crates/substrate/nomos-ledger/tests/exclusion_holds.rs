//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.

use nomos_ledger::{
    Claim, ClaimRefusal, ExclusionLedger, FileLedger, Finish, FinishRefusal, ItemId, ItemState,
    LedgerDocument, LedgerError, LedgerItem, ReleaseOutcome, Territory as ItemTerritory, Validate,
    VerificationPredicate, VerificationRecord,
};
use nomos_model::SetResolution;
use nomos_platform::{Clock, FileSystem, FileSystemError, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;
use std::time::Duration;

/// A clock the tests hold still, so lease expiry is reached by arithmetic rather than by
/// sleeping. A suite that sleeps to reach a deadline is a suite that is slow and
/// intermittently wrong.
struct FixedClock(i64);

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

const NOW: i64 = 1_000_000;

fn At(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

fn Territory(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().copied());
}

fn Item(id: &str, files: &[&str]) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        territory: Territory(files),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
    };
}

fn Held_By(mut item: LedgerItem, holder: &str, expires: i64) -> LedgerItem
{
    item.state = ItemState::Claimed;
    item.claim = Some(Claim {
        holder: holder.to_owned(),
        acquired_at: At(NOW),
        lease_expires_at: At(expires),
    });
    return item;
}

fn Document(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: 1,
        items,
    };
}

fn Temp_Dir(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-ledger-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    Write_Gate(&path);
    return path;
}

/// Every tree these tests build is a repository with a gate, because finishing now reads
/// one and refuses when it cannot.
///
/// The lint step is `cargo --version` rather than the real clippy invocation. These tests
/// are about what a *predicate's* exit code does to an item; running a real workspace lint
/// in each of them would make the suite take minutes and would couple it to whatever the
/// workspace currently contains. What the derived step actually is, and that it comes from
/// the workflow rather than from a constant, is covered in `gate_covers_finish.rs`.
fn Write_Gate(directory: &Path)
{
    let workflows = directory.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
    std::fs::write(
        workflows.join("gate.yml"),
        "jobs:\n\
         \x20 gate:\n\
         \x20   steps:\n\
         \x20     - name: Lint\n\
         \x20       run: cargo --version\n",
    )
    .expect("test needs a workflow");
}

fn Ledger_At<'clock>(
    directory: &Path,
    clock: &'clock FixedClock,
) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
{
    let ledger_path = directory.join("ledger.json");
    let lock_path = directory.join("ledger.lock");

    return FileLedger::At(
        ledger_path,
        StdFileSystem,
        clock,
        FileLock::At(lock_path),
    );
}


// ---------------------------------------------------------------------------
// Acceptance 1 — two active claims on overlapping territory are refused.
// ---------------------------------------------------------------------------

#[test]
fn Test_Validate_Should_Refuse_Two_Active_Claims_On_Overlapping_Territory()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/a.rs", "src/b.rs"]), "agent-a", NOW + 3_600),
        Held_By(Item("T-2", &["src/b.rs", "src/c.rs"]), "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        violations.iter().any(|violation| violation.contains("overlapping")),
        "overlapping active claims must be a violation, got: {violations:?}"
    );
}

/// The negative control. Remove the overlap and the same two claims must be fine — if
/// this fails, the rule above is firing on everything and proves nothing.
#[test]
fn Test_Validate_Should_Accept_Two_Active_Claims_On_Disjoint_Territory()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 3_600),
        Held_By(Item("T-2", &["src/c.rs"]), "agent-b", NOW + 3_600),
    ]);

    assert_eq!(
        Validate(&document, At(NOW)),
        Vec::<String>::new(),
        "disjoint territory must not be reported as a conflict"
    );
}

/// A lapsed claim stops excluding. Two overlapping claims are fine when one of them
/// belongs to an agent that went away hours ago.
#[test]
fn Test_A_Lapsed_Claim_Should_Not_Conflict()
{
    let document = Document(vec![
        Held_By(Item("T-1", &["src/b.rs"]), "agent-a", NOW - 1),
        Held_By(Item("T-2", &["src/b.rs"]), "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        !violations.iter().any(|violation| violation.contains("overlapping")),
        "a lapsed claim must stop excluding, got: {violations:?}"
    );
}

/// Territory that cannot be compared must be reported, not passed over. This is the
/// arm that keeps an unanswered question from reading as an answer.
#[test]
fn Test_Incomparable_Territory_Should_Be_Reported_Not_Ignored()
{
    let mut second = Item("T-2", &["src/b.rs"]);
    second.territory.resolution = SetResolution::Symbol;

    let document = Document(vec![
        Held_By(Item("T-1", &["src/b.rs"]), "agent-a", NOW + 3_600),
        Held_By(second, "agent-b", NOW + 3_600),
    ]);

    let violations = Validate(&document, At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("cannot be shown independent")),
        "incomparable territory must be reported, got: {violations:?}"
    );
}

// ---------------------------------------------------------------------------
// Acceptance 2 — an item cannot be done without recorded verification.
// ---------------------------------------------------------------------------

#[test]
fn Test_A_Done_Item_Should_Require_Recorded_Verification()
{
    let mut finished = Item("T-1", &["src/a.rs"]);
    finished.state = ItemState::Done;

    let violations = Validate(&Document(vec![finished]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("no recorded verification")),
        "prose is not a predicate, got: {violations:?}"
    );
}

/// The negative control: with a verification record, the same item is fine.
#[test]
fn Test_A_Verified_Done_Item_Should_Be_Accepted()
{
    let mut finished = Item("T-1", &["src/a.rs"]);
    finished.state = ItemState::Done;
    finished.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "test result: ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
    });

    assert_eq!(Validate(&Document(vec![finished]), At(NOW)), Vec::<String>::new());
}

#[test]
fn Test_An_Unrunnable_Predicate_Should_Be_Refused()
{
    let mut item = Item("T-1", &["src/a.rs"]);
    item.verification = Some(VerificationPredicate::New(Vec::new()));

    let violations = Validate(&Document(vec![item]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("cannot be run")),
        "an empty argv is a filled-in field, not a predicate, got: {violations:?}"
    );
}

#[test]
fn Test_A_Blocked_Item_Should_Say_Why()
{
    let mut item = Item("T-1", &["src/a.rs"]);
    item.state = ItemState::Blocked;

    let violations = Validate(&Document(vec![item]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("without saying why"))
    );
}

/// Every violation must be reported, not just the first. An author who has to re-run to
/// discover the next problem is an author who stops re-running.
#[test]
fn Test_Validation_Should_Report_Every_Violation_At_Once()
{
    let mut blocked = Item("T-1", &["src/a.rs"]);
    blocked.state = ItemState::Blocked;
    let mut done = Item("T-2", &["src/b.rs"]);
    done.state = ItemState::Done;
    let mut dangling = Item("T-3", &["src/c.rs"]);
    dangling.depends_on = vec![ItemId::New("T-99")];

    let violations = Validate(&Document(vec![blocked, done, dangling]), At(NOW));

    assert!(
        violations.len() >= 3,
        "expected every violation, got: {violations:?}"
    );
}

// ---------------------------------------------------------------------------
// Acceptance 3 — claiming through the ledger honours the same rules.
// ---------------------------------------------------------------------------

#[test]
fn Test_Claiming_Overlapping_Territory_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-overlap");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs", "src/shared.rs"]),
            Item("T-2", &["src/shared.rs", "src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("the first claim is uncontended");

    let refusal = ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect_err("overlapping territory must be refused");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
    assert!(refusal.Is_Retryable(), "a held item is a queue, not a wall");
    assert!(refusal.Describe().contains("agent-a"));

    let _ = std::fs::remove_dir_all(&directory);
}

/// The negative control: disjoint territory claims concurrently, which is the entire
/// point of doing any of this.
#[test]
fn Test_Claiming_Disjoint_Territory_Should_Succeed_Concurrently()
{
    let directory = Temp_Dir("claim-disjoint");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs"]),
            Item("T-2", &["src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");
    ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect("disjoint territory must be claimable concurrently");

    ledger.Validate_Current().expect("both claims are legitimate");

    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// Acceptance 5 — finishing runs the predicate, and believes it.
// ---------------------------------------------------------------------------

/// A command that exits with the given code, on either platform family.
fn Exits_With(code: i32) -> Vec<String>
{
    return if cfg!(windows)
    {
        vec!["cmd".to_owned(), "/C".to_owned(), format!("exit {code}")]
    }
    else
    {
        vec!["sh".to_owned(), "-c".to_owned(), format!("exit {code}")]
    };
}

fn Item_Verified_By(id: &str, files: &[&str], argv: Vec<String>) -> LedgerItem
{
    let mut item = Item(id, files);
    item.verification = Some(VerificationPredicate::New(argv));
    return item;
}

/// The Phase 0 acceptance criterion: a completion whose predicate exits non-zero is
/// refused, and the item does not become done.
#[test]
fn Test_Finishing_Should_Be_Refused_When_The_Predicate_Fails()
{
    let directory = Temp_Dir("finish-fails");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item_Verified_By(
            "T-1",
            &["src/a.rs"],
            Exits_With(1),
        )]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = Finish(
        &mut ledger,
        &StdProcessLauncher,
        &ItemId::New("T-1"),
        "agent-a",
        Some(&directory),
    )
    .expect_err("a predicate that exits non-zero must refuse the completion");

    assert!(matches!(refusal, FinishRefusal::PredicateFailed { .. }));
    assert!(refusal.Judged_The_Work());

    let after = ledger.Load().expect("readable");
    assert_eq!(
        after.items.first().map(|item| &item.state),
        Some(&ItemState::Claimed),
        "a refused completion must leave the item claimed, not done"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The negative control. Without it, a `Finish` that refused everything unconditionally
/// would pass the test above.
#[test]
fn Test_Finishing_Should_Succeed_When_The_Predicate_Passes()
{
    let directory = Temp_Dir("finish-passes");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item_Verified_By(
            "T-1",
            &["src/a.rs"],
            Exits_With(0),
        )]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let record = Finish(
        &mut ledger,
        &StdProcessLauncher,
        &ItemId::New("T-1"),
        "agent-a",
        Some(&directory),
    )
    .expect("a passing predicate must finish the item");

    assert_eq!(record.exit_code, 0);
    assert_eq!(record.verified_at, At(NOW));

    let after = ledger.Load().expect("readable");
    let finished = after.items.first().expect("the item survives");
    assert_eq!(finished.state, ItemState::Done);
    assert!(
        finished.verified.is_some(),
        "a done item carries the evidence that made it done"
    );
    ledger
        .Validate_Current()
        .expect("a verified done item is a valid ledger");

    let _ = std::fs::remove_dir_all(&directory);
}

/// An item with nothing to run cannot be shown to be finished. "There was nothing to
/// check" must not read the same as "everything checked out".
#[test]
fn Test_Finishing_Should_Be_Refused_Without_A_Predicate()
{
    let directory = Temp_Dir("finish-no-predicate");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = Finish(
        &mut ledger,
        &StdProcessLauncher,
        &ItemId::New("T-1"),
        "agent-a",
        Some(&directory),
    )
    .expect_err("an item with no predicate cannot be finished");

    assert!(matches!(refusal, FinishRefusal::NoPredicate { .. }));
    assert!(
        !refusal.Judged_The_Work(),
        "nothing was learned about the work"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// A predicate that cannot be started says nothing about the work. Reporting it as a
/// failed check would tell an author their code is wrong when their tooling is missing.
#[test]
fn Test_An_Unstartable_Predicate_Should_Not_Judge_The_Work()
{
    let directory = Temp_Dir("finish-unstartable");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item_Verified_By(
            "T-1",
            &["src/a.rs"],
            vec!["nomos-no-such-program-exists".to_owned()],
        )]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = Finish(
        &mut ledger,
        &StdProcessLauncher,
        &ItemId::New("T-1"),
        "agent-a",
        Some(&directory),
    )
    .expect_err("a missing program is not a verdict");

    assert!(matches!(refusal, FinishRefusal::CouldNotRun { .. }));
    assert!(!refusal.Judged_The_Work());

    let _ = std::fs::remove_dir_all(&directory);
}

/// The state transition to `Done` carries its own evidence, so an item cannot arrive
/// there by any route that skipped verification.
#[test]
fn Test_Releasing_As_Finished_Should_Record_The_Verification()
{
    let directory = Temp_Dir("finish-records");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Finished(VerificationRecord {
                argv: vec!["cargo".to_owned(), "test".to_owned()],
                exit_code: 0,
                output_tail: "ok".to_owned(),
                verified_at: At(NOW),
                gate: None,
            }),
        )
        .expect("a release carrying evidence must be accepted");

    let after = ledger.Load().expect("readable");
    let finished = after.items.first().expect("the item survives");
    assert_eq!(finished.state, ItemState::Done);
    assert_eq!(
        finished.verified.as_ref().map(|record| record.exit_code),
        Some(0)
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The reason these tests assert on.
///
/// Prose rather than a marker, and asserted as text rather than as presence. A field that
/// exists and holds an empty string satisfies `is_some()`, which is exactly the assertion
/// that would have let the old behaviour through.
const REASON: &str =
    "the fixture never reproduced the shape that broke it, so the control proved nothing";

/// The arm adjacent to the one above, which used to throw its evidence away.
///
/// [`ReleaseOutcome::Abandoned`] has always carried `reason: String` non-optionally — the
/// same technique the doc comment praises the finished arm for — and the store matched it
/// with `{ .. }` and set the state and nothing else. One match, two arms, one keeping its
/// evidence and one discarding it.
#[test]
fn Test_Releasing_As_Abandoned_Should_Record_Who_Stopped_And_Why()
{
    let directory = Temp_Dir("abandon-records");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Abandoned {
                reason: REASON.to_owned(),
            },
        )
        .expect("a holder may give up its own claim");

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");
    let abandonment = item
        .abandoned
        .first()
        .expect("the abandonment must survive the release that produced it");

    assert_eq!(abandonment.reason, REASON, "the reason the holder gave was not kept");
    assert_eq!(abandonment.holder, "agent-a", "the record does not say who stopped");
    assert_eq!(abandonment.abandoned_at, At(NOW), "the record does not say when");

    let _ = std::fs::remove_dir_all(&directory);
}

/// The second control, holding a line the crate already drew.
///
/// An abandonment is a record of something that stopped. A record that went on excluding
/// people would be a worse defect than the one it replaced, so the item must go back to
/// `Ready`, the claim must go, and — the part worth checking rather than inferring —
/// somebody else must actually be able to take it.
#[test]
fn Test_An_Abandoned_Item_Should_Return_To_Ready_And_Stop_Excluding()
{
    let directory = Temp_Dir("abandon-releases");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");
    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Abandoned {
                reason: REASON.to_owned(),
            },
        )
        .expect("a holder may give up its own claim");

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");
    assert_eq!(item.state, ItemState::Ready);
    assert!(item.claim.is_none(), "a claim that survives an abandonment goes on excluding");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect("an abandoned item must be claimable by somebody else");

    let taken = ledger.Load().expect("readable");
    let again = taken.items.first().expect("the item survives");
    assert_eq!(
        again.abandoned.len(),
        1,
        "the next claim erased the record of the last one"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Why the field is a list and not the most recent one.
///
/// An item abandoned twice was abandoned twice. Keeping only the latest would discard the
/// earlier reason, which is the loss this whole item is about, one scale down.
#[test]
fn Test_An_Item_Abandoned_Twice_Should_Keep_Both_Reasons()
{
    let directory = Temp_Dir("abandon-twice");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");

    for (holder, reason) in [("agent-a", "ran out of lease"), ("agent-b", REASON)]
    {
        ledger
            .Claim(&ItemId::New("T-1"), holder, Duration::from_secs(3_600))
            .expect("an abandoned item is claimable again");
        ledger
            .Release(
                &ItemId::New("T-1"),
                holder,
                ReleaseOutcome::Abandoned {
                    reason: reason.to_owned(),
                },
            )
            .expect("a holder may give up its own claim");
    }

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");

    let said: Vec<(&str, &str)> = item
        .abandoned
        .iter()
        .map(|entry| return (entry.holder.as_str(), entry.reason.as_str()))
        .collect();

    assert_eq!(
        said,
        vec![("agent-a", "ran out of lease"), ("agent-b", REASON)],
        "both abandonments must survive, oldest first"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The question `OD-LEDGER-006` settles, asserted here rather than left in the prose.
///
/// A lapsed claim is given no synthesized abandonment. Nobody was there to write a reason,
/// and inventing one — "the lease expired" — would be filler wearing a record's clothes.
/// What a lapse leaves is the claim itself: it stops counting as active without being
/// removed, so a reader can still see who held it and when they stopped. The two paths now
/// differ by what is actually knowable rather than by which one ran.
///
/// What this test deliberately does not assert is that the next agent can take the item.
/// It cannot: a lapse leaves `state` at `Claimed`, and `Claim_Refusal` rejects anything
/// that is not `Ready` before it ever reaches the lease. That contradicts `item.rs`, which
/// says a lapse "stops excluding, which is what lets the next agent take the item", and it
/// is a different defect from this one — recorded in `OD-LEDGER-006` and on the ledger,
/// not asserted here, because an assertion would pin the behaviour in place.
#[test]
fn Test_A_Lapsed_Claim_Should_Stay_Visible_And_Invent_No_Reason()
{
    let directory = Temp_Dir("abandon-lapse");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let later = FixedClock(NOW + 7_200);
    let lapsed = Ledger_At(&directory, &later);
    let after = lapsed.Load().expect("readable");
    let item = after.items.first().expect("the item survives");

    assert!(
        item.abandoned.is_empty(),
        "a lapse wrote a reason nobody gave: {:?}",
        item.abandoned
    );
    assert!(
        item.claim.is_some(),
        "the lapsed claim was removed, so nothing says the work was ever started"
    );
    assert!(
        !item.Has_Active_Claim(At(NOW + 7_200)),
        "a lapsed claim must stop counting as an active claim"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn Test_A_Lease_Beyond_The_Ceiling_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-lease");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");

    let refusal = ledger
        .Claim(
            &ItemId::New("T-1"),
            "agent-a",
            nomos_ledger::MAXIMUM_LEASE + Duration::from_secs(1),
        )
        .expect_err("an unbounded lease defeats the ledger");

    assert!(matches!(refusal, ClaimRefusal::LeaseTooLong { .. }));

    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn Test_Renewing_Someone_Elses_Claim_Should_Be_Refused()
{
    let directory = Temp_Dir("renew-foreign");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = ledger
        .Renew(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err("renewing another holder's claim must be refused");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));

    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// Durability — a corrupt ledger is never mistaken for an empty one.
// ---------------------------------------------------------------------------

/// A missing ledger is a repository that has not started tracking work. A *corrupt*
/// ledger is somebody's roadmap that got damaged, and treating it as empty would let
/// every agent claim everything.
#[test]
fn Test_A_Corrupt_Ledger_Should_Be_An_Error_Not_An_Empty_One()
{
    let directory = Temp_Dir("corrupt");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a corrupt ledger must not read as empty");

    assert!(matches!(error, LedgerError::Malformed { .. }));

    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn Test_A_Missing_Ledger_Should_Read_As_Empty()
{
    let directory = Temp_Dir("missing");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    let document = ledger.Load().expect("a missing ledger is not an error");

    assert!(document.items.is_empty());

    let _ = std::fs::remove_dir_all(&directory);
}

/// An invalid document must be refused *before* it is written. A ledger that is written
/// and then found invalid is one somebody has to repair by hand, and until they do
/// every agent is reading something the system itself says is wrong.
#[test]
fn Test_Saving_An_Invalid_Ledger_Should_Be_Refused_Before_The_Write()
{
    let directory = Temp_Dir("refuse-invalid");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    let mut blocked = Item("T-1", &["src/a.rs"]);
    blocked.state = ItemState::Blocked;

    let error = ledger
        .Save(&Document(vec![blocked]))
        .expect_err("an invalid ledger must not be persisted");

    assert!(matches!(error, LedgerError::Invalid { .. }));
    assert!(
        !directory.join("ledger.json").exists(),
        "nothing may be written when validation fails"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Round-tripping must be lossless. If it is not, an agent's claim silently changes
/// meaning the next time somebody else writes the file.
#[test]
fn Test_The_Ledger_Should_Round_Trip_Losslessly()
{
    let directory = Temp_Dir("round-trip");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    let original = Document(vec![
        Held_By(Item("T-1", &["src/a.rs", "src/b.rs"]), "agent-a", NOW + 3_600),
        Item("T-2", &["src/c.rs"]),
    ]);

    ledger.Save(&original).expect("valid");
    let reloaded = ledger.Load().expect("readable");

    assert_eq!(reloaded, original);

    let _ = std::fs::remove_dir_all(&directory);
}

/// A dependency edge that only `validate` reads is a comment. Claiming has to refuse an
/// item whose prerequisite is unfinished, or the ordering is advice.
#[test]
fn Test_Claiming_An_Item_With_An_Unfinished_Dependency_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-dependency");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"]), dependent]))
        .expect("a fresh ledger is valid");

    let refusal = ledger
        .Claim(&ItemId::New("T-2"), "agent-a", Duration::from_secs(3_600))
        .expect_err("an unfinished dependency must refuse the claim");

    assert!(matches!(refusal, ClaimRefusal::DependencyUnmet { .. }), "{}", refusal.Describe());
    assert!(refusal.Is_Retryable(), "finishing T-1 is what resolves this");
    assert!(refusal.Describe().contains("T-1"), "{}", refusal.Describe());

    let _ = std::fs::remove_dir_all(&directory);
}

/// The negative control. A satisfied dependency must not stand in the way.
#[test]
fn Test_A_Finished_Dependency_Should_Not_Block_A_Claim()
{
    let directory = Temp_Dir("claim-dependency-met");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let mut finished = Item("T-1", &["src/a.rs"]);
    finished.state = ItemState::Done;
    finished.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
    });
    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![finished, dependent]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-2"), "agent-a", Duration::from_secs(3_600))
        .expect("a met dependency must not refuse");

    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// A lapse must not brick the board. P10-LAPSE-BRICKS.
// ---------------------------------------------------------------------------

/// The defect, and the reason it needed an experiment rather than a reading.
///
/// Validation used to call `Claimed` with no *active* claim a violation, and a lease
/// expiring is precisely that. So a document written valid stopped being valid on its own,
/// `Save` refuses an invalid document, and every claim saves — which meant one lapsed lease
/// refused every claim on the board, including items sharing no territory with it. The
/// lease expiring caused exactly what `MAXIMUM_LEASE` exists to prevent.
///
/// The unit level was right the whole time and that is what hid it: `Has_Active_Claim`
/// returns false on a lapsed claim, exclusion honours that, and validation refused the
/// document before exclusion was ever consulted.
#[test]
fn Test_A_Lapsed_Lease_Should_Not_Stop_The_Rest_Of_The_Board()
{
    let directory = Temp_Dir("lapse-bricks");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs"]),
            Item("T-2", &["src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "dead-agent", Duration::from_secs(3_600))
        .expect("uncontended");

    let later = FixedClock(NOW + 7_200);
    let mut after = Ledger_At(&directory, &later);
    let document = after.Load().expect("the file is still readable");

    // One: the document is not called invalid because time passed.
    assert_eq!(
        Validate(&document, At(NOW + 7_200)),
        Vec::<String>::new(),
        "a lapsed lease made the whole document invalid, so nothing can be written to it"
    );
    after
        .Validate_Current()
        .expect("validate must not call a board with a lapsed lease broken");

    // Two: an unrelated item is still claimable. `src/b.rs` shares nothing with `src/a.rs`,
    // so a refusal here is not exclusion — it is the board refusing to be written at all.
    after
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| {
            panic!(
                "an item sharing no territory with the lapsed one was refused: {}",
                refusal.Describe()
            )
        });

    let _ = std::fs::remove_dir_all(&directory);
}

/// Three: what the lapsed item itself does, which is a decision rather than a consequence.
///
/// It stays `Claimed` and nobody else may take it. `OD-LEDGER-009` states the grounds:
/// `Claim` overwrites `claim`, and `claim` is the only thing recording that the work was
/// ever started — which `OD-LEDGER-006` decided must survive, having refused to synthesize
/// an `Abandonment` for a lapse because `Abandonment::reason` is the words the holder gave
/// and a lapse has none. Taking a lapsed item over is a different operation from claiming a
/// free one, and it does not exist yet.
///
/// Asserted rather than left implicit, because the refusal is now deliberate. What must not
/// happen is that it becomes claimable by accident and quietly erases who was working on it.
#[test]
fn Test_A_Lapsed_Item_Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible()
{
    let directory = Temp_Dir("lapse-takeover");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "dead-agent", Duration::from_secs(3_600))
        .expect("uncontended");

    let later = FixedClock(NOW + 7_200);
    let mut after = Ledger_At(&directory, &later);

    let refusal = after
        .Claim(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err(
            "a lapsed item is not claimable, and silently allowing it would erase the only \
             record that the work was started",
        );

    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "the refusal must say the item is not in a claimable state rather than blame \
         territory or the identifier: {}",
        refusal.Describe()
    );

    let held = after.Load().expect("readable");
    let item = held.items.first().expect("the item survives");
    assert_eq!(
        item.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("dead-agent".to_owned()),
        "the lapsed claim was replaced, so nothing says who walked away from this"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The holder's own recovery still works, and is now the whole recovery story.
///
/// `Renew` and `Release` match on the holder and never took the validating path, so an
/// agent that came back could always rescue its own claim. That was the only recovery there
/// was while the board was bricked; it is still the only way a lapsed item returns to the
/// pool, and the lapse now blocks only that item rather than every item.
#[test]
fn Test_The_Holder_Should_Still_Recover_Its_Own_Lapsed_Claim()
{
    let directory = Temp_Dir("lapse-recover");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let later = FixedClock(NOW + 7_200);
    let mut after = Ledger_At(&directory, &later);

    after
        .Renew(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| {
            panic!("the holder must be able to renew: {}", refusal.Describe())
        });

    let held = after.Load().expect("readable");
    assert!(
        held.items
            .first()
            .is_some_and(|item| return item.Has_Active_Claim(At(NOW + 7_200))),
        "renewing a lapsed claim must make it active again"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The positive control: validation still catches the corruption it was aimed at.
///
/// Dropping the time-dependent rule must not become "validation stopped looking". An item
/// marked `Claimed` with no claim at all is a real corruption — nothing can say whose work
/// it is or was — and unlike a lapse it cannot arrive by the passage of time.
#[test]
fn Test_A_Claimed_Item_Recording_No_Claim_Should_Still_Be_Invalid()
{
    let mut item = Item("T-1", &["src/a.rs"]);
    item.state = ItemState::Claimed;
    item.claim = None;

    let violations = Validate(&Document(vec![item]), At(NOW));

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("records no claim")),
        "a claimed item with no claim is a corruption and must still be reported: \
         {violations:?}"
    );
}

/// A lapsed claim is not that corruption, stated beside it so the pair cannot drift.
#[test]
fn Test_A_Lapsed_Claim_Should_Not_Be_Reported_As_A_Violation()
{
    let document = Document(vec![Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW - 1)]);

    assert_eq!(
        Validate(&document, At(NOW)),
        Vec::<String>::new(),
        "a lease that ran out is the normal end of an agent that died, not a broken file"
    );
}

// ---------------------------------------------------------------------------
// A load failure names its cause. P10-LAPSE-BRICKS, second half.
// ---------------------------------------------------------------------------

/// Two causes wore one name, which is why the defect above needed an experiment.
///
/// Every load and save failure in `Claim`, `Renew` and `Release` discarded its error and
/// returned `NoSuchItem`, so an unreadable file, a parse error and an invalid document all
/// told the operator that their identifier matched nothing. The text is asserted here
/// rather than only the variant, because the defect was precisely what the operator read.
#[test]
fn Test_An_Unusable_Ledger_Should_Not_Be_Reported_As_A_Missing_Item()
{
    let directory = Temp_Dir("unusable-ledger");
    let clock = FixedClock(NOW);

    // A genuinely missing item, over a ledger that is fine.
    let mut sound = Ledger_At(&directory, &clock);
    sound
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    let missing = sound
        .Claim(&ItemId::New("T-NOPE"), "agent-a", Duration::from_secs(3_600))
        .expect_err("no such item");

    // The same call over a file that is not a ledger at all.
    std::fs::write(directory.join("ledger.json"), "{ not json").expect("writes the corruption");
    let mut broken = Ledger_At(&directory, &clock);
    let unusable = broken
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect_err("a ledger that will not parse cannot be claimed against");

    assert!(
        matches!(missing, ClaimRefusal::NoSuchItem { .. }),
        "{}",
        missing.Describe()
    );
    assert!(
        matches!(unusable, ClaimRefusal::LedgerUnusable { .. }),
        "a broken ledger was reported as {}",
        unusable.Describe()
    );
    assert_ne!(
        missing.Describe(),
        unusable.Describe(),
        "the two causes must not read the same, which is the defect"
    );
    assert!(
        unusable.Describe().contains("malformed"),
        "the refusal must carry what the store said: {}",
        unusable.Describe()
    );
    assert!(
        !unusable.Describe().contains("no item named"),
        "a broken ledger still sends the operator to check a spelling: {}",
        unusable.Describe()
    );

    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// Two writers, one ledger, and neither update is lost. P10-LOCK-BYPASS.
// ---------------------------------------------------------------------------

/// How long the harness lets the second writer run before it releases the first one.
///
/// This is not a timing assumption about the machine. Under the arrangement these tests
/// exist to hold, the second writer *cannot* finish while the first one is between its read
/// and its write, so this wait is expected to expire — it is the bound on how long the
/// harness waits to learn that. Without the arrangement the second writer finishes in well
/// under a millisecond and the wait ends immediately, so the failing case is fast and the
/// passing case pays this once.
const SECOND_WRITER_LIMIT: Duration = Duration::from_millis(750);

/// A one-way gate: opened once, and waited on by whoever needs to know it opened.
///
/// A [`std::sync::Barrier`] would be shorter and cannot express the wait that is *meant* to
/// time out, which is the whole of what the second half of this harness observes.
struct Gate
{
    open: Mutex<bool>,
    changed: Condvar,
}

impl Gate
{
    fn New() -> Self
    {
        return Self {
            open: Mutex::new(false),
            changed: Condvar::new(),
        };
    }

    fn Open(&self)
    {
        let mut open = self.open.lock().expect("the harness never panics under this lock");
        *open = true;
        self.changed.notify_all();
    }

    fn Wait(&self)
    {
        let open = self.open.lock().expect("the harness never panics under this lock");
        let _held = self
            .changed
            .wait_while(open, |open| return !*open)
            .expect("the harness never panics under this lock");
    }

    /// Waits up to `limit`, and reports whether the gate opened within it.
    fn Opened_Within(&self, limit: Duration) -> bool
    {
        let open = self.open.lock().expect("the harness never panics under this lock");
        let (_held, timing) = self
            .changed
            .wait_timeout_while(open, limit, |open| return !*open)
            .expect("the harness never panics under this lock");
        return !timing.timed_out();
    }
}

/// A filesystem that holds one thread still between its read of the ledger and its write.
///
/// # Why the seam is here and not in the store
///
/// Two writers that merely run at the same time reproduce a lost update by luck, and a test
/// that reproduces by luck is one that goes green on a slower machine while the defect is
/// still there. What is needed is the interleaving itself: one writer's read must be known
/// to have happened before the other writer's whole operation, and its write must be known
/// to happen after.
///
/// [`FileLedger`] already takes the filesystem it reads through, for a reason its own doc
/// comment gives — a caller reaching for `std::fs` would put the store beyond a test's
/// control. That injection point is enough, so nothing test-only is added to the store: this
/// is an ordinary [`FileSystem`] that happens to stop after handing back the bytes.
struct Interleaving
{
    ledger: PathBuf,
    /// The thread that gets held, registered by that thread itself.
    held: Mutex<Option<ThreadId>>,
    /// Opened when the held thread has read the document it is about to write back.
    read: Gate,
    /// Opened by the harness when the held thread may proceed to its write.
    resume: Gate,
    /// Whether the hold has already happened, so it happens once rather than per read.
    stopped: AtomicBool,
}

impl Interleaving
{
    fn Over(ledger: PathBuf) -> Self
    {
        return Self {
            ledger,
            held: Mutex::new(None),
            read: Gate::New(),
            resume: Gate::New(),
            stopped: AtomicBool::new(false),
        };
    }

    /// Registers the calling thread as the one to hold.
    fn Hold_This_Thread(&self)
    {
        let mut held = self.held.lock().expect("the harness never panics under this lock");
        *held = Some(std::thread::current().id());
    }

    fn Holds(&self, path: &Path) -> bool
    {
        if path != self.ledger
        {
            return false;
        }

        let held = *self.held.lock().expect("the harness never panics under this lock");

        return held == Some(std::thread::current().id());
    }

    /// Whether the hold ever happened. The harness asserts this: a run in which the seam
    /// never fired proves nothing about interleaving, however green it is.
    fn Stopped(&self) -> bool
    {
        return self.stopped.load(Ordering::SeqCst);
    }
}

impl FileSystem for &Interleaving
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        let text = StdFileSystem.Read_To_String(path);

        // After the read, never before it. The point of the hold is that this thread is
        // carrying a snapshot of the document that somebody else is about to change.
        if self.Holds(path) && !self.stopped.swap(true, Ordering::SeqCst)
        {
            self.read.Open();
            self.resume.Wait();
        }

        return text;
    }

    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>
    {
        return StdFileSystem.Replace_Atomically(path, contents);
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return StdFileSystem.Exists(path);
    }
}

type InterleavedLedger<'shared> =
    FileLedger<&'shared Interleaving, &'shared FixedClock, FileLock>;

fn Ledger_Over<'shared>(
    filesystem: &'shared Interleaving,
    directory: &Path,
    clock: &'shared FixedClock,
) -> InterleavedLedger<'shared>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        filesystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

/// Runs two writers against one ledger with `first` held between its read and its write.
///
/// The order is fixed rather than raced. `second` does not start until `first` has read, and
/// `first` does not write until `second` has either finished or been kept waiting for
/// [`SECOND_WRITER_LIMIT`]. Both outcomes are legitimate and they are what the two
/// arrangements look like from outside: without exclusion `second` completes inside the
/// window and `first` then writes over it; with exclusion `second` is still waiting for the
/// lock when the window closes, and it reads `first`'s write when it finally gets in.
fn Two_Writers(
    directory: &Path,
    clock: &FixedClock,
    first: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
    second: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
)
{
    let filesystem = Interleaving::Over(directory.join("ledger.json"));
    let shared = &filesystem;
    let finished = Gate::New();
    let done = &finished;

    std::thread::scope(|scope| {
        scope.spawn(move || {
            shared.Hold_This_Thread();
            let mut ledger = Ledger_Over(shared, directory, clock);
            first(&mut ledger);
        });

        shared.read.Wait();

        scope.spawn(move || {
            let mut ledger = Ledger_Over(shared, directory, clock);
            second(&mut ledger);
            done.Open();
        });

        finished.Opened_Within(SECOND_WRITER_LIMIT);
        shared.resume.Open();
    });

    assert!(
        filesystem.Stopped(),
        "the seam never fired, so nothing was interleaved and this run proves nothing"
    );
}

fn Holder_Of(document: &LedgerDocument, id: &str) -> Option<String>
{
    return document
        .items
        .iter()
        .find(|item| return item.id == ItemId::New(id))
        .and_then(|item| return item.claim.as_ref())
        .map(|claim| return claim.holder.clone());
}

/// The defect `P10-LOCK-BYPASS` is open for, at the verb that starts every piece of work.
///
/// `Claim` read the whole document, decided against it and wrote the whole document back,
/// with no lock held anywhere in between — while [`FileLedger::With_Lock`] sat beside it
/// saying every mutation went through it. Two sessions overlapping therefore lost one of the
/// two writes, and what was lost was not a field: it was every change the other session had
/// made to any item, because the document written back was a snapshot taken before that
/// session existed.
///
/// The two items here reserve disjoint territory, so exclusion has nothing to say about
/// them. Both claims are legitimate and both must survive. That is the point: this is not a
/// test about refusing a claim, it is a test about not losing one that was granted.
#[test]
fn Test_Two_Concurrent_Claims_Should_Both_Survive()
{
    let directory = Temp_Dir("concurrent-claims");
    let clock = FixedClock(NOW);

    Ledger_At(&directory, &clock)
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs"]),
            Item("T-2", &["src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    Two_Writers(
        &directory,
        &clock,
        |ledger| {
            ledger
                .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
                .expect("T-1 is uncontended");
        },
        |ledger| {
            ledger
                .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
                .expect("T-2 shares no territory with T-1");
        },
    );

    let after = Ledger_At(&directory, &clock).Load().expect("readable");

    assert_eq!(
        Holder_Of(&after, "T-1"),
        Some("agent-a".to_owned()),
        "the first writer's claim is not in the ledger it wrote"
    );
    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "the second writer was told its claim was granted and the ledger does not have it: \
         one writer wrote back a document it had read before the other one existed"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The same loss at the verb that ends a piece of work.
///
/// `Release` is the write `Finish` performs after its predicate passes, so a lost one is an
/// agent that ran its verification, was told the item was recorded as done, and left behind
/// a board that still calls the item claimed — or, as here, a board that has forgotten
/// somebody else's claim entirely.
#[test]
fn Test_A_Release_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let directory = Temp_Dir("concurrent-release");
    let clock = FixedClock(NOW);

    Ledger_At(&directory, &clock)
        .Save(&Document(vec![
            Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 3_600),
            Item("T-2", &["src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    Two_Writers(
        &directory,
        &clock,
        |ledger| {
            ledger
                .Release(
                    &ItemId::New("T-1"),
                    "agent-a",
                    ReleaseOutcome::Abandoned {
                        reason: REASON.to_owned(),
                    },
                )
                .expect("a holder may give up its own claim");
        },
        |ledger| {
            ledger
                .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
                .expect("T-2 shares no territory with T-1");
        },
    );

    let after = Ledger_At(&directory, &clock).Load().expect("readable");

    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "a release wrote back a document read before the other writer's claim, so the claim \
         it was granted is gone"
    );
    assert_eq!(
        after
            .items
            .iter()
            .find(|item| return item.id == ItemId::New("T-1"))
            .map(|item| return item.abandoned.len()),
        Some(1),
        "the abandonment the first writer was told had been recorded is not in the ledger"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// And at the verb an agent runs most often, which is the one that hides best.
///
/// A renewal that is lost does not look like a lost write. It looks like a lease that ran out
/// early, which reads as an agent that died — so the wrong thing gets investigated.
#[test]
fn Test_A_Renewal_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let directory = Temp_Dir("concurrent-renew");
    let clock = FixedClock(NOW);

    Ledger_At(&directory, &clock)
        .Save(&Document(vec![
            Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 10),
            Item("T-2", &["src/b.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    Two_Writers(
        &directory,
        &clock,
        |ledger| {
            ledger
                .Renew(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
                .expect("a holder may renew its own claim");
        },
        |ledger| {
            ledger
                .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
                .expect("T-2 shares no territory with T-1");
        },
    );

    let after = Ledger_At(&directory, &clock).Load().expect("readable");

    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "a renewal wrote back a document read before the other writer's claim, so the claim \
         it was granted is gone"
    );
    assert_eq!(
        after
            .items
            .iter()
            .find(|item| return item.id == ItemId::New("T-1"))
            .and_then(|item| return item.claim.as_ref())
            .map(|claim| return claim.lease_expires_at),
        Some(At(NOW + 3_600)),
        "the renewal the first writer was told had been recorded is not in the ledger"
    );

    let _ = std::fs::remove_dir_all(&directory);
}
