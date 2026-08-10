//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.

use nomos_ledger::{
    Abandonment, Blocker, Claim, ClaimRefusal, ExclusionLedger, FileLedger, Finish, FinishRefusal,
    GateOutcome, ItemId, ItemState, LedgerDocument, LedgerError, LedgerItem, ReleaseOutcome,
    SCHEMA_VERSION, Territory as ItemTerritory, Validate, VerificationPredicate,
    VerificationRecord,
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
        displaced: Vec::new(),
        declined: None,
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

/// `SCHEMA_VERSION` rather than a literal `1`.
///
/// `Save` stamps what it writes with the version this build understands, so a fixture holding a
/// literal would stop equalling its own reload the moment the constant moves — and
/// `Test_The_Ledger_Should_Round_Trip_Losslessly` would then fail for a reason that has nothing
/// to do with round-tripping.
fn Document(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: SCHEMA_VERSION,
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

/// What an unexpanded pattern does to the board, pinned as measured rather than as argued.
///
/// `P10-PATTERN-BRICK` found this by reading `territory.rs` rather than from an incident: a
/// single entry in `patterns` short-circuits `Territory::Intersect` to `Unknown` before a
/// single path is compared, and `Unknown` refuses non-retryably.
///
/// # The item's own description of this was one clause too strong, and the correction matters
///
/// `P10-PATTERN-BRICK` says the item "can never be claimed by anyone, its own holder
/// included". Measured here, that is not what happens, because `Conflicts` compares only
/// against items holding an **active claim**. So a pattern item on a quiet board claims
/// perfectly normally — asserted below, because it is the step that makes the rest possible.
///
/// The real shape is worse than an item nobody can take, and this is the finding:
///
/// 1. the pattern item is claimable exactly when the board is quiet, so nothing warns the
///    agent who takes it;
/// 2. from that moment every other claim is refused against it, including territory sharing
///    no path with it at all;
/// 3. and the refusal is the non-retryable one, which by `README.md`'s exit-code contract
///    tells each refused agent to stop and fetch a person rather than pick up another item.
///
/// So one agent quietly acquires the power to stop every other session, and learns nothing
/// about having done so. `OD-LEDGER-013` withdrew `--territory-pattern` on the strength of
/// this. The state stays reachable by hand-editing the document, which is why this test can
/// still construct it, and why the `Unknown` in `Territory::Intersect` is kept rather than
/// relaxed: withdrawing the flag removes the way in, not the guard.
#[test]
fn Test_A_Held_Pattern_Should_Refuse_Every_Other_Claim_On_The_Board()
{
    let directory = Temp_Dir("pattern-brick");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let mut vague = Item("T-1", &["src/a.rs"]);
    vague.territory = vague.territory.With_Pattern("crates/spec/**");

    ledger
        .Save(&Document(vec![
            vague,
            Item("T-2", &["docs/unrelated.md"]),
            Item("T-3", &["tests/also-unrelated.rs"]),
        ]))
        .expect("a document carrying a pattern is well-formed, which is the problem");

    // 1. It claims without complaint. Nothing is held yet, so nothing is compared, so the
    //    pattern is never consulted. This is the step the item's description missed.
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("on a quiet board a pattern item claims like any other, which is the trap");

    // 2. And now the board is shut. `docs/unrelated.md` shares nothing with `src/a.rs` or
    //    with `crates/spec/**`, and is refused anyway — the short-circuit runs before any
    //    path is looked at, so being unrelated is no defence.
    for (item, holder) in [("T-2", "agent-b"), ("T-3", "agent-c")]
    {
        let collateral = ledger
            .Claim(&ItemId::New(item), holder, Duration::from_secs(3_600))
            .expect_err("every other claim is refused against the held pattern");

        assert!(
            matches!(collateral, ClaimRefusal::UnknownIndependence { .. }),
            "{item}: {collateral:?}"
        );
        assert!(
            !collateral.Is_Retryable(),
            "{item}: this is the sharp end — a non-retryable refusal tells the agent to stop \
             and fetch a person, so one held pattern reads to every session as a broken ledger"
        );
    }

    let _ = std::fs::remove_dir_all(&directory);
}

/// And it shuts in the other direction too, once anything at all is held.
///
/// The complement of the test above, and together they are why the state has no safe
/// ordering: claim the pattern first and it stops everyone else; claim anything else first
/// and the pattern item can never be taken. There is no sequence in which the board both
/// carries a pattern and keeps working.
#[test]
fn Test_A_Pattern_Item_Should_Be_Unclaimable_Once_Anything_Is_Held()
{
    let directory = Temp_Dir("pattern-brick-reverse");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let mut vague = Item("T-2", &["src/b.rs"]);
    vague.territory = vague.territory.With_Pattern("crates/spec/**");

    ledger
        .Save(&Document(vec![Item("T-1", &["docs/unrelated.md"]), vague]))
        .expect("a document carrying a pattern is well-formed");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("an ordinary item on a quiet board");

    let refused = ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect_err("the pattern item cannot be claimed while anything is held");

    assert!(matches!(refused, ClaimRefusal::UnknownIndependence { .. }), "{refused:?}");
    assert!(
        !refused.Is_Retryable(),
        "and waiting will not help: `docs/unrelated.md` is disjoint from `src/b.rs`, so the \
         refusal is not contention and no lease expiring resolves it"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// A refusal printed under the refused item's own identifier must not read as a statement
/// about the blocker.
///
/// `P10-AUDIT-STATE` measured the failure: `work audit` prints one line per blocked item, the
/// identifier first, and the held arm of `Describe` used to open with the *blocker's* name.
/// Forty-four lines read `P1-MODEL: P9-AUTHORING overlaps territory held by …`, in which the
/// only thing a reader can be sure of is that one of the two names was refused, and nothing
/// says which. `OD-LEDGER-014` moved the phrasing into the library.
///
/// The assertion is positional rather than a substring search, because a substring search is
/// what a wrong sentence also passes: both spellings contain both identifiers, and only the
/// order distinguishes them.
#[test]
fn Test_A_Refusal_Should_Not_Open_With_The_Blockers_Name()
{
    let directory = Temp_Dir("refusal-subject");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-BLOCKER", &["src/shared.rs"]),
            Item("T-REFUSED", &["src/shared.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-BLOCKER"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = ledger
        .Claim(&ItemId::New("T-REFUSED"), "agent-b", Duration::from_secs(3_600))
        .expect_err("overlapping territory must be refused");

    let sentence = refusal.Describe();

    // The whole defect in one assertion: the blocker's name must not be the first thing the
    // sentence says. Restoring `{item} overlaps territory held by {holder} …` makes this red
    // and leaves every other assertion in this file green, which is what makes it the control
    // for this arm rather than a restatement of the ones above.
    assert!(
        !sentence.starts_with("T-BLOCKER"),
        "the refusal opens with the blocker's name, so printed under the refused item's own \
         identifier it says the reverse of what happened: {sentence}"
    );

    // And it still has to say who is in the way, or the fix would have been to delete the
    // information rather than to place it.
    assert!(
        sentence.contains("T-BLOCKER"),
        "the refusal must still name the blocker: {sentence}"
    );
    assert!(
        sentence.contains("agent-a"),
        "and who holds it: {sentence}"
    );

    // The composed line `work audit` actually prints. Read as English, the subject is
    // `T-REFUSED` and `T-BLOCKER` is what its territory runs into.
    let line = format!("{:<13} {:<9} {sentence}", "T-REFUSED", "held");
    assert!(
        line.starts_with("T-REFUSED"),
        "the caller names the subject and the description follows it: {line}"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// One rendering, not two.
///
/// `work audit` carried its own copy of the held arm while the library's was wrong. Both are
/// now the library's, and this pins the sentence the CLI composes so the workaround cannot
/// quietly come back as a local `format!` that drifts from this one.
#[test]
fn Test_The_Held_Arm_Should_Have_Exactly_One_Rendering()
{
    let refusal = ClaimRefusal::HeldBy {
        holder: "agent-a".to_owned(),
        until: At(NOW + 3_600),
        item: ItemId::New("T-BLOCKER"),
    };

    assert_eq!(
        refusal.Describe(),
        format!("territory overlaps T-BLOCKER, held by agent-a until unix {}", NOW + 3_600)
    );
}

/// The negative control for the test above: the same two items, minus the pattern.
///
/// Without this, `Test_An_Unexpanded_Pattern_Should_Brick_Every_Claim_On_The_Board` would
/// pass just as happily if claiming were broken for some entirely unrelated reason, and the
/// record would be citing a measurement of nothing. One line differs between the two.
#[test]
fn Test_The_Same_Board_Without_The_Pattern_Should_Claim_Freely()
{
    let directory = Temp_Dir("pattern-brick-control");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs"]),
            Item("T-2", &["docs/unrelated.md"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("the pattern was the only thing stopping this");
    ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect("and the only thing stopping this");

    let _ = std::fs::remove_dir_all(&directory);
}

/// A directory reserves what is beneath it, which is what the withdrawn flag was for.
///
/// This is the load-bearing half of `OD-LEDGER-013`: withdrawing `--territory-pattern` is
/// only a narrowing if it removed something an author could otherwise say. It did not.
/// "Everything under `crates/spec`" is an ordinary territory entry, it is decided from the
/// text with no filesystem access, and — unlike the pattern — it answers.
#[test]
fn Test_A_Directory_Should_Reserve_Its_Subtree_Without_A_Pattern()
{
    let directory = Temp_Dir("subtree-without-pattern");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["crates/spec"]),
            Item("T-2", &["crates/spec/nomos-spec-model/src/lib.rs"]),
            Item("T-3", &["crates/host/nomos-cli/src/work.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect_err("a file inside a reserved directory is reserved");

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the subtree is *held*, not unanswerable — the distinction is the whole record: \
         {refusal:?}"
    );
    assert!(
        refusal.Is_Retryable(),
        "and it is a queue rather than a wall, which the pattern never was"
    );

    // While genuinely unrelated territory is still free, so the directory entry reserves a
    // subtree rather than the repository.
    ledger
        .Claim(&ItemId::New("T-3"), "agent-c", Duration::from_secs(3_600))
        .expect("a directory reserves its subtree, not the board");

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

// ---------------------------------------------------------------------------
// Durability — a build that cannot account for the document does not write it.
//
// P10-STALE-WRITER, decided in `OD-LEDGER-008`. The test above this comment is the reason
// this section exists: round-tripping a document built from `LedgerItem` only ever contains
// keys `LedgerItem` declares, so it asserted that the *declared* shape survives, which was
// never in question. Everything below writes JSON to disk by hand.
// ---------------------------------------------------------------------------

/// One item as JSON text, with `extra` spliced in as further keys on it.
///
/// Written out rather than built from [`LedgerItem`] on purpose. A fixture built from the type
/// cannot carry a key the type does not declare, and the key it does not declare is exactly what
/// an older binary was dropping.
fn Raw_Ledger(schema_version: u32, extra: &str) -> String
{
    return format!(
        "{{\n  \"schema_version\": {schema_version},\n  \"items\": [\
         {{\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]}},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
         \"verification\":null,\"verified\":null,\"abandoned\":[]{extra}}}\
         ]\n}}\n"
    );
}

/// The subject. A key no declared type recognises must refuse the read.
///
/// Serde's default is to ignore it, and ignoring it is what cost an abandonment reason on
/// 2026-08-09: the key was dropped on the way in and therefore absent on the way out, so a
/// lossy write and a clean one were the same event at the surface reporting them.
#[test]
fn Test_A_Ledger_Carrying_An_Undeclared_Key_Should_Not_Load()
{
    let directory = Temp_Dir("undeclared-key");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    std::fs::write(
        directory.join("ledger.json"),
        Raw_Ledger(
            SCHEMA_VERSION,
            ",\"a_field_this_build_does_not_know\":{\"holder\":\"agent-a\"}",
        ),
    )
    .expect("write");

    let error = ledger
        .Load()
        .expect_err("a document carrying a key this build cannot account for must not load");

    assert!(
        format!("{error}").contains("a_field_this_build_does_not_know"),
        "the refusal must name the key it could not account for: {error}"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Every object in the document refuses, not a list of types somebody maintained.
///
/// A hand-written list of the types carrying the attribute is only as complete as the hand —
/// `OD-COMPLETENESS-001` — and the attribute is invisible to the public-surface snapshot, so
/// nothing else in this workspace can see it removed from one nested type. The universe is
/// therefore derived from a fully populated document: serialize it, walk it, and probe every
/// object node that walk finds.
///
/// This is what covers a type that does not exist yet. A field added to [`LedgerItem`] whose
/// own container forgot the attribute fails here without anybody extending this test.
#[test]
fn Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key()
{
    let mut item = Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 3_600);
    item.state = ItemState::Blocked;
    item.blocked = Some(Blocker::Dependency {
        items: vec![ItemId::New("T-0")],
    });
    item.depends_on = vec![ItemId::New("T-0")];
    item.territory = ItemTerritory::Of_Files(["src/a.rs"]).With_Pattern("src/**");
    item.verification = Some(VerificationPredicate::New(vec!["cargo".to_owned()]));
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: Some(GateOutcome {
            argv: vec!["cargo".to_owned(), "--version".to_owned()],
            exit_code: 0,
        }),
    });
    item.abandoned = vec![Abandonment {
        holder: "agent-b".to_owned(),
        reason: "went to look at something else".to_owned(),
        abandoned_at: At(NOW),
    }];
    // The nested type `OD-LEDGER-012` added. Populated here rather than left empty because an
    // empty list serializes as `[]` and contributes no object node, so the walk below would
    // never reach a `Claim` inside `displaced` and the guard would be silent about it.
    item.displaced = vec![Claim {
        holder: "dead-agent".to_owned(),
        acquired_at: At(NOW),
        lease_expires_at: At(NOW + 60),
    }];

    let whole = serde_json::to_value(Document(vec![item])).expect("the document serializes");

    let mut pointers = Vec::new();
    Object_Pointers(&whole, "", &mut pointers);

    for pointer in &pointers
    {
        let mut probed = whole.clone();
        let node = probed
            .pointer_mut(pointer)
            .expect("every pointer was collected from this same value");
        let object = node
            .as_object_mut()
            .expect("only object nodes were collected");
        object.insert("nomos_probe".to_owned(), serde_json::Value::Null);

        assert!(
            serde_json::from_value::<LedgerDocument>(probed).is_err(),
            "an undeclared key was accepted at `{pointer}`, so a build that predates a field \
             there would drop it and write the document back"
        );
    }

    assert!(
        pointers.len() >= 11,
        "only {} object(s) were probed, so the fixture above has stopped being fully \
         populated — the guard did not shrink, the universe did",
        pointers.len()
    );
}

/// Every object node in a value, as JSON pointers.
///
/// The document's shape rather than a list of types, which is the point of the test above.
fn Object_Pointers(value: &serde_json::Value, at: &str, found: &mut Vec<String>)
{
    match value
    {
        serde_json::Value::Object(fields) =>
        {
            found.push(at.to_owned());
            for (key, nested) in fields
            {
                Object_Pointers(nested, &format!("{at}/{key}"), found);
            }
        }
        serde_json::Value::Array(elements) =>
        {
            for (index, nested) in elements.iter().enumerate()
            {
                Object_Pointers(nested, &format!("{at}/{index}"), found);
            }
        }
        _ =>
        {}
    }
}

/// A file newer than this build says so, rather than being called damaged.
///
/// The two failures reach `Load` by the same route and the operator's next action is opposite:
/// rebuild the reader, or repair the file. Reporting the first as the second sends somebody to
/// edit a file that is correct.
#[test]
fn Test_A_Ledger_Newer_Than_This_Build_Should_Say_So_Rather_Than_Malformed()
{
    let directory = Temp_Dir("newer-than-build");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    // The spliced key was `"displaced":[]` when this test was written, chosen as a field a
    // later build might add. `OD-LEDGER-012` added it, so it became declared and the parse it
    // was here to make fail started succeeding. Named for what it is instead, which is the
    // same lesson `Raw_Ledger`'s own fixture learned: a probe key must not be one the schema
    // can catch up with.
    std::fs::write(
        directory.join("ledger.json"),
        Raw_Ledger(9_999, ",\"a_field_this_build_does_not_know\":[]"),
    )
    .expect("write");

    let error = ledger.Load().expect_err("a newer document must not load");

    let LedgerError::Unrecognized {
        understood, found, ..
    } = &error
    else
    {
        panic!("a file newer than this build must not be reported as damaged: {error}");
    };

    assert_eq!(*understood, SCHEMA_VERSION);
    assert_eq!(*found, 9_999);

    let said = format!("{error}");
    assert!(said.contains("9999"), "{said}");
    assert!(said.contains(&format!("{SCHEMA_VERSION}")), "{said}");
    assert!(
        said.to_lowercase().contains("rebuild"),
        "the message must name the remedy: {said}"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The control that keeps the distinction honest.
///
/// Without it, `Unrecognized` could swallow every parse failure and an operator with a genuinely
/// damaged file would be told to rebuild a binary that is fine. Asserted on the variant rather
/// than only on `is_err`, which `Test_A_Corrupt_Ledger_Should_Be_An_Error_Not_An_Empty_One`
/// already covers from the other side.
#[test]
fn Test_A_Ledger_That_Is_Merely_Broken_Should_Still_Be_Malformed()
{
    let directory = Temp_Dir("merely-broken");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a broken document must not load");

    assert!(
        matches!(error, LedgerError::Malformed { .. }),
        "a damaged file must not be reported as one this build is too old for: {error}"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// What is written says what this build understands, not what the file it read said.
///
/// The one state the schema version cannot explain is a document carrying keys a build invented
/// while claiming the older number it loaded: an older reader then fails the strict parse, probes
/// the version, finds nothing newer than itself, and reports the file damaged — sending somebody
/// to repair a file that is correct. Stamping on the way out is what forecloses that, and the
/// version is the only field in the document that `Save` does not take from its argument.
///
/// Asserted with a version this build could not have produced, because a fixture already holding
/// `SCHEMA_VERSION` cannot tell a stamp from an echo while the constant sits at one.
#[test]
fn Test_Saving_Should_Stamp_The_Version_This_Build_Understands()
{
    let directory = Temp_Dir("stamps-version");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&LedgerDocument {
            schema_version: 9_999,
            items: vec![Item("T-1", &["src/a.rs"])],
        })
        .expect("valid");

    let written = std::fs::read_to_string(directory.join("ledger.json")).expect("readable");

    assert!(
        written.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")),
        "the write echoed the version it was handed rather than stamping this build's:\n{written}"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The other direction, and the control `done_when` asks for by name.
///
/// A guard that refused every write would stop the board rather than protect it, and the
/// asymmetry is the whole trade: a new build still reads an old file, because every added field
/// carries `serde(default)`, and an old build no longer reads a new one. Every item in this
/// repository's own committed ledger was written without `abandoned` at some point, and this is
/// that case.
#[test]
fn Test_A_Document_Written_Before_A_Field_Existed_Should_Still_Load()
{
    let directory = Temp_Dir("older-than-build");
    let clock = FixedClock(NOW);
    let ledger = Ledger_At(&directory, &clock);

    std::fs::write(
        directory.join("ledger.json"),
        "{\n  \"schema_version\": 1,\n  \"items\": [\
         {\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
         \"verification\":null,\"verified\":null}\
         ]\n}\n",
    )
    .expect("write");

    let document = ledger
        .Load()
        .expect("a document written before a field existed must still read");

    assert_eq!(document.items.len(), 1);
    assert!(
        document
            .items
            .first()
            .is_some_and(|item| item.abandoned.is_empty() && item.displaced.is_empty()),
        "a missing field must read as absent rather than refusing the file"
    );

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
/// It stays `Claimed` and a plain `claim` on it is refused. `OD-LEDGER-009` states the
/// grounds: `Claim` overwrites `claim`, and `claim` is the only thing recording that the work
/// was ever started — which `OD-LEDGER-006` decided must survive, having refused to synthesize
/// an `Abandonment` for a lapse because `Abandonment::reason` is the words the holder gave
/// and a lapse has none. Taking a lapsed item over is a different operation from claiming a
/// free one, and `OD-LEDGER-012` is where it became one.
///
/// Asserted rather than left implicit, because the refusal is now deliberate. What must not
/// happen is that it becomes claimable by accident and quietly erases who was working on it.
///
/// # What changed here, and why it is this test working rather than a regression
///
/// This test was `…Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible`, and "refuse a new
/// holder" became false once `takeover` existed: a new holder is exactly what a takeover
/// installs. Its subject — `claim` is refused — is **not** reversed, and the refusal is
/// stronger than it was, because `Lapsed` names the holder and the remedy where `NotClaimable`
/// said only that the item was `Claimed`. The assertion that the lapsed claim was not replaced
/// is kept word for word: nothing in `OD-LEDGER-012` erases a claim, and the test was written
/// to stop it being erased *by accident*.
///
/// This test's own previous doc comment said it "stops short of the claimability, because an
/// assertion either way would pin the behaviour before that decision is made". The decision is
/// made, in `OD-LEDGER-012`. `Test_A_Lapsed_Item_Should_Be_Taken_Over_And_Still_Name_Its_\
/// Previous_Holder` covers the case this one could not, because the operation did not exist.
#[test]
fn Test_A_Lapsed_Item_Should_Refuse_A_Plain_Claim_And_Name_The_Takeover()
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
        matches!(refusal, ClaimRefusal::Lapsed { .. }),
        "the refusal must say the item is not in a claimable state rather than blame \
         territory or the identifier: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("takeover"),
        "a refusal whose remedy is a verb has to name the verb: {}",
        refusal.Describe()
    );
    assert!(
        !refusal.Is_Retryable(),
        "waiting does not revive a dead holder; somebody has to decide to take the work"
    );

    let held = after.Load().expect("readable");
    let item = held.items.first().expect("the item survives");
    assert_eq!(
        item.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("dead-agent".to_owned()),
        "the lapsed claim was replaced, so nothing says who walked away from this"
    );
    assert!(
        item.displaced.is_empty(),
        "a refused claim recorded a displacement, so `claim` has quietly become `takeover`"
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

// ---------------------------------------------------------------------------
// A lapse is taken over, and the claim it replaces is kept. P10-LAPSE-TAKEOVER,
// decided in `OD-LEDGER-012`.
//
// The two tests above this comment are the boundary the decision was made against: a lapse
// must not brick the board, and a plain `claim` must not quietly erase a dead agent's claim.
// Everything below is the operation that does take the item, and the assertion running through
// all of it is that nothing it does is silent.
// ---------------------------------------------------------------------------

/// The subject, and exactly the sequence `done_when` names.
///
/// Claim it, force the lease into the past, take it over as somebody else, and assert both
/// halves: the takeover succeeded, and the previous holder is still named on the item. The
/// second half is the whole point — *"a new claim overwriting the old one silently is the
/// outcome this must not have."*
#[test]
fn Test_A_Lapsed_Item_Should_Be_Taken_Over_And_Still_Name_Its_Previous_Holder()
{
    let directory = Temp_Dir("takeover-keeps-predecessor");
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

    let reservation = after
        .Take_Over(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| {
            panic!(
                "an item whose holder died must return to the pool without a person editing \
                 the file: {}",
                refusal.Describe()
            )
        });

    assert_eq!(reservation.holder, "agent-b");

    let held = after.Load().expect("readable");
    let item = held.items.first().expect("the item survives");

    assert_eq!(
        item.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("agent-b".to_owned()),
        "the takeover did not install the new holder"
    );
    assert!(
        item.Has_Active_Claim(At(NOW + 7_200)),
        "a takeover that leaves the lease in the past has taken nothing"
    );
    assert_eq!(
        item.state,
        ItemState::Claimed,
        "a takeover does not move the state; the item was claimed and still is"
    );

    // The half this item exists for. Who held it, when they took it, and when the lease ran
    // out — all three survive, and they survive as the claim itself rather than as a summary.
    assert_eq!(
        item.displaced.len(),
        1,
        "the takeover kept {} displaced claim(s) rather than exactly the one it replaced, so \
         either nothing records who walked away from this or something records it twice",
        item.displaced.len()
    );
    let displaced = item
        .displaced
        .first()
        .expect("the claim the takeover replaced is kept");
    assert_eq!(
        displaced.holder, "dead-agent",
        "the takeover dropped the previous holder, which is the outcome this must not have"
    );
    assert_eq!(displaced.acquired_at, At(NOW), "when they took it");
    assert_eq!(
        displaced.lease_expires_at,
        At(NOW + 3_600),
        "when the lease ran out"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// A live claim is not a lapse, and `takeover` is not a way to steal work in progress.
///
/// Without this the verb is worse than the defect it fixes: an agent that is merely slow gets
/// its item taken by anybody who types the word. The lease is the whole of the protection and
/// renewing it is the whole of the remedy, which is why `HeldBy` here is retryable — waiting
/// is genuinely the right advice when somebody is working.
#[test]
fn Test_An_Item_With_A_Live_Claim_Should_Not_Be_Taken_Over()
{
    let directory = Temp_Dir("takeover-refuses-live");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let soon = FixedClock(NOW + 60);
    let mut during = Ledger_At(&directory, &soon);

    let refusal = during
        .Take_Over(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err("a live claim must not be displaced by a takeover");

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "{}",
        refusal.Describe()
    );
    assert!(
        refusal.Is_Retryable(),
        "the lease running out is what resolves this, so waiting is the honest advice"
    );

    let held = during.Load().expect("readable");
    let item = held.items.first().expect("the item survives");
    assert_eq!(
        item.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("agent-a".to_owned()),
        "a takeover displaced a holder who was still working"
    );
    assert!(item.displaced.is_empty(), "{:?}", item.displaced);

    let _ = std::fs::remove_dir_all(&directory);
}

/// The wrong verb is refused rather than accommodated.
///
/// A `takeover` that quietly worked as a `claim` would mean two verbs doing one thing, and the
/// whole point of a separate verb is that it means something the other does not. Both ends of
/// the range are covered: an item nobody has ever held, and one that is finished.
#[test]
fn Test_Taking_Over_An_Item_Nobody_Holds_Should_Be_Refused()
{
    let directory = Temp_Dir("takeover-wrong-verb");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let mut finished = Item("T-2", &["src/b.rs"]);
    finished.state = ItemState::Done;
    finished.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
    });

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"]), finished]))
        .expect("a fresh ledger is valid");

    for (item, what) in [("T-1", "an item nobody holds"), ("T-2", "a finished item")]
    {
        let refusal = ledger
            .Take_Over(&ItemId::New(item), "agent-b", Duration::from_secs(3_600))
            .expect_err("a takeover answers a lapse and nothing else");

        assert!(
            matches!(refusal, ClaimRefusal::NotClaimable { .. }),
            "{what} must read as the wrong verb rather than as a queue: {}",
            refusal.Describe()
        );
        assert!(
            !refusal.Is_Retryable(),
            "{what} will not become takeable by waiting"
        );
    }

    let held = ledger.Load().expect("readable");
    assert!(
        held.items
            .iter()
            .all(|item| return item.claim.is_none() && item.displaced.is_empty()),
        "a refused takeover wrote to the item anyway"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Independence is re-established, not assumed — the case that is easy to get wrong and
/// impossible to notice afterwards.
///
/// A lapsed claim stops excluding, so the ground it reserved may already have been taken by
/// somebody else. A takeover that skipped the territory check would hand two live agents the
/// same files and call it recovery, which is the one thing this ledger exists to prevent.
///
/// It passes because `Take_Over` calls `Contested_By`, the function `Claim_Refusal` calls. If it
/// ever passes because the loop was copied instead, the two copies will drift and this test
/// will not see it.
#[test]
fn Test_A_Takeover_Should_Refuse_Territory_Somebody_Has_Since_Claimed()
{
    let directory = Temp_Dir("takeover-refuses-taken-ground");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    // Two items over the same file. Concurrently claimable only while one of them is not.
    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/a.rs"]),
            Item("T-2", &["src/a.rs"]),
        ]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "dead-agent", Duration::from_secs(3_600))
        .expect("uncontended");

    let later = FixedClock(NOW + 7_200);
    let mut after = Ledger_At(&directory, &later);

    // Succeeds precisely because T-1's lapsed claim no longer excludes. This is the state the
    // takeover then has to notice.
    after
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect("a lapsed claim stops excluding, so this ground is free");

    let refusal = after
        .Take_Over(&ItemId::New("T-1"), "agent-c", Duration::from_secs(3_600))
        .expect_err("the ground T-1 reserves is held by a live claim on T-2");

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "{}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("agent-b"),
        "the refusal must name who holds the ground now: {}",
        refusal.Describe()
    );

    let held = after.Load().expect("readable");
    let taken = held
        .items
        .iter()
        .find(|item| return item.id == ItemId::New("T-1"))
        .expect("T-1 survives");
    assert_eq!(
        taken.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("dead-agent".to_owned()),
        "a refused takeover replaced the claim anyway"
    );
    assert!(taken.displaced.is_empty(), "{:?}", taken.displaced);

    let _ = std::fs::remove_dir_all(&directory);
}

/// A list, not a field.
///
/// An item taken over twice was taken over twice, and keeping only the most recent would
/// discard the earlier holder — `OD-LEDGER-006`'s reason for `abandoned` being a list, restated
/// one field across. This is the test that a single-slot implementation passes B1 and fails.
#[test]
fn Test_An_Item_Taken_Over_Twice_Should_Name_Both_Predecessors()
{
    let directory = Temp_Dir("takeover-twice");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "dead-agent", Duration::from_secs(3_600))
        .expect("uncontended");

    let second = FixedClock(NOW + 7_200);
    let mut takes = Ledger_At(&directory, &second);
    takes
        .Take_Over(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect("the first holder's lease ran out");

    // Past `agent-b`'s lease too: NOW + 7_200 + 3_600.
    let third = FixedClock(NOW + 14_400);
    let mut again = Ledger_At(&directory, &third);
    again
        .Take_Over(&ItemId::New("T-1"), "agent-c", Duration::from_secs(3_600))
        .expect("the second holder's lease ran out as well");

    let held = again.Load().expect("readable");
    let item = held.items.first().expect("the item survives");

    assert_eq!(
        item.claim.as_ref().map(|claim| return claim.holder.clone()),
        Some("agent-c".to_owned())
    );
    assert_eq!(
        item.displaced
            .iter()
            .map(|claim| return claim.holder.clone())
            .collect::<Vec<String>>(),
        vec!["dead-agent".to_owned(), "agent-b".to_owned()],
        "oldest first, and both of them: keeping only the most recent is this same loss one \
         scale down"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Absence is not permission.
///
/// `Claimed` with no claim recorded is the one corruption `Validate` still refuses, so this
/// document is written by hand — `Save` will not persist it and `Load` does not validate. A
/// takeover must refuse it rather than fill the hole: writing a claim over a missing record
/// would destroy the fact that it was missing, which is `OD-LEDGER-008`'s defect one scale down.
#[test]
fn Test_An_Item_Claimed_With_No_Claim_Recorded_Should_Not_Be_Taken_Over()
{
    let directory = Temp_Dir("takeover-refuses-a-hole");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    let path = directory.join("ledger.json");
    std::fs::write(
        &path,
        format!(
            "{{\n  \"schema_version\": {SCHEMA_VERSION},\n  \"items\": [\
             {{\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
             \"done_when\":\"the tests pass\",\
             \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\
             \"patterns\":[]}},\
             \"state\":\"Claimed\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
             \"verification\":null,\"verified\":null,\"abandoned\":[],\"displaced\":[]}}\
             ]\n}}\n"
        ),
    )
    .expect("write");
    let before = std::fs::read(&path).expect("readable");

    let refusal = ledger
        .Take_Over(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err("an item whose predecessor record is already missing is not taken over");

    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "{}",
        refusal.Describe()
    );
    assert_eq!(
        std::fs::read(&path).expect("readable"),
        before,
        "a refused takeover rewrote the file"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The new field through the strict deserializer, which is the pairing the two items exist to
/// make safe.
///
/// `OD-LEDGER-008` made `Load` refuse a key it cannot account for and `Save` stamp the version
/// rather than echo it. A field added afterwards has to survive both, and the second `Save` here
/// is what shows the stamp is idempotent rather than incrementing on every write — a version
/// that climbed on its own would make every file unreadable by the build that wrote it.
#[test]
fn Test_An_Item_With_A_Displaced_Claim_Should_Round_Trip_Losslessly()
{
    let directory = Temp_Dir("takeover-round-trip");
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
    after
        .Take_Over(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect("a lapsed item is takeable");

    let first = after.Load().expect("a document carrying `displaced` must read");
    assert_eq!(first.schema_version, SCHEMA_VERSION);
    assert!(
        first
            .items
            .first()
            .is_some_and(|item| return item.displaced.len() == 1),
        "the displaced claim did not survive the write"
    );

    after.Save(&first).expect("valid");
    let second = after.Load().expect("readable");

    assert_eq!(second, first, "a displaced claim changed meaning on rewrite");

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
