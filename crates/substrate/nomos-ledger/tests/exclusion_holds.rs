//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.

use nomos_ledger::{
    Claim, ClaimRefusal, ExclusionLedger, FileLedger, ItemId, ItemState, LedgerDocument,
    LedgerError, LedgerItem, Validate, VerificationPredicate, VerificationRecord,
};
use nomos_model::{Content_Digest, SetResolution, SubjectSet};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use nomos_contracts::SubjectId;
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

fn Subject(name: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(name.as_bytes()));
}

fn Territory(files: &[&str]) -> SubjectSet
{
    return SubjectSet::Of(SetResolution::File, files.iter().map(|name| Subject(name)));
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
    return path;
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
    let mut second = Item("T-2", &[]);
    second.territory = SubjectSet::Of(SetResolution::Symbol, [Subject("src/b.rs::foo")]);

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
