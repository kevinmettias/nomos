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
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher};
use std::path::{Path, PathBuf};
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
