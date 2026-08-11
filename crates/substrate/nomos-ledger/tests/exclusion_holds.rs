//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.

use nomos_ledger::{
    Finishing,
    Abandonment, AddRefusal, Blocker, Claim, ClaimRefusal, Declination, ExclusionLedger, FileLedger, Finish,
    FinishRefusal, GateOutcome, ItemId, ItemState, LedgerDocument, LedgerError, LedgerItem,
    ReleaseOutcome, Reservation, SCHEMA_VERSION, Territory as ItemTerritory, Validate, VerificationPredicate,
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

impl Clock for FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

/// Offered by reference as well, so the two shared statics can be lent to many ledgers while
/// a test that needs its own moment hands one over by value.
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

/// A temporary repository that removes itself when the test holding it ends.
///
/// Every test here used to close with its own `remove_dir_all`, which is a line that only
/// runs when the test passes: a failed assertion unwinds straight past it. `Drop` runs on
/// the unwind too, so the tree is cleared exactly when the value goes out of scope and the
/// cleanup is no longer a step a test can forget or an assertion can skip.
struct Scratch(PathBuf);

impl Drop for Scratch
{
    fn drop(&mut self)
    {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch
{
    type Target = Path;

    fn deref(&self) -> &Path
    {
        return &self.0;
    }
}

fn Temp_Dir(name: &str) -> Scratch
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-ledger-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    Write_Gate(&path);

    return Scratch(path);
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

/// The clock every test that does not move time shares.
///
/// A `'static` clock is what lets [`Board_At`] hand back a ledger: the ledger borrows its
/// clock, so a local one could not outlive the call that built it. A test that needs time
/// to move builds its own later clock and a second ledger over the same directory.
static AT_NOW: FixedClock = FixedClock(NOW);

/// The one-hour lease every test here takes, said once.
const LEASE: Duration = Duration::from_secs(3_600);

/// One agent claims one item for the standard lease, and it is expected to succeed.
///
/// A refusal here is the fixture failing rather than the assertion under test, so it panics
/// with the refusal's own words instead of returning it.
fn Take<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .unwrap_or_else(|refusal| panic!("the fixture claim was refused: {}", refusal.Describe()));
}

/// An item whose territory carries a pattern, which nothing on the command line can build
/// any more — `OD-LEDGER-013` withdrew `--territory-pattern` — and which these two tests
/// still construct by hand, because the state stays reachable by editing the document.
fn Patterned(id: &str, files: &[&str], pattern: &str) -> LedgerItem
{
    let mut item = Item(id, files);
    item.territory = item.territory.With_Pattern(pattern);

    return item;
}

/// An item that is `Done` and carries the evidence that made it done.
///
/// A `Done` item without a verification record is not a valid ledger, so the two are built
/// together or not at all.
fn Finished(id: &str, files: &[&str]) -> LedgerItem
{
    let mut item = Item(id, files);
    item.state = ItemState::Done;
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
    });

    return item;
}

/// A holder releases its own claim as finished, carrying the evidence that made it so.
///
/// The record is the fixture rather than the subject — what these tests assert is what the
/// store does with it — so building it here keeps eleven lines of literal out of the test.
fn Release_As_Finished<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Release(
            &ItemId::New(item),
            holder,
            ReleaseOutcome::Finished(VerificationRecord {
                argv: vec!["cargo".to_owned(), "test".to_owned()],
                exit_code: 0,
                output_tail: "ok".to_owned(),
                verified_at: At(NOW),
                gate: None,
            }),
        )
        .expect("a release carrying evidence must be accepted");
}

/// Every abandonment the item kept, as who stopped and what they said, oldest first.
fn Abandonments(item: &LedgerItem) -> Vec<(&str, &str)>
{
    return item
        .abandoned
        .iter()
        .map(|entry| return (entry.holder.as_str(), entry.reason.as_str()))
        .collect();
}

/// A holder gives up its own claim, with the words it gave for stopping.
fn Abandon<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str, reason: &str)
{
    ledger
        .Release(
            &ItemId::New(item),
            holder,
            ReleaseOutcome::Abandoned {
                reason: reason.to_owned(),
            },
        )
        .expect("a holder may give up its own claim");
}

/// A claim that is expected to be refused, with the refusal handed back as the value the
/// test is about.
fn Refused<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str) -> ClaimRefusal
{
    return ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .expect_err("this claim is contended and must be refused");
}

/// Runs an item's own verification predicate through the ledger, in a named repository.
fn Finish_In(
    ledger: &mut FileLedger<StdFileSystem, &FixedClock, FileLock>,
    directory: &Path,
    item: &str,
    holder: &str,
) -> Result<VerificationRecord, FinishRefusal>
{
    return Finish(
        ledger,
        &StdProcessLauncher,
        &Finishing {
            item: &ItemId::New(item),
            holder,
        },
        Some(directory),
    );
}

/// The one item a single-item board carries, read back through the file.
///
/// Nearly every test below reaches the same pair of lines to get at the one value it asserts
/// on. Naming the pair keeps the load out of the assertion's way, and going through the file
/// rather than through the in-memory value is the point of asserting at all: what the next
/// session sees is what was written, not what this one still holds.
fn Only_Item<Clock: nomos_platform::Clock>(
    ledger: &FileLedger<StdFileSystem, Clock, FileLock>,
) -> LedgerItem
{
    return ledger
        .Load()
        .expect("the ledger is readable")
        .items
        .into_iter()
        .next()
        .expect("the item survives");
}

/// One named item on a board carrying several, read back through the file.
///
/// The multi-item boards assert about one of their items and use the others as the context
/// that makes the assertion mean something, so reaching the subject by name rather than by
/// position keeps the test honest when the board is reordered.
fn Named<Clock: nomos_platform::Clock>(
    id: &str,
    ledger: &FileLedger<StdFileSystem, Clock, FileLock>,
) -> LedgerItem
{
    return ledger
        .Load()
        .expect("the ledger is readable")
        .items
        .into_iter()
        .find(|item| return item.id == ItemId::New(id))
        .unwrap_or_else(|| panic!("{id} survives"));
}

/// Who an item's own claim names, or nothing when it carries none.
///
/// The tests compare holders, and `claim.as_ref().map(|claim| claim.holder.clone())` is four
/// tokens of plumbing in front of one word. This says the word.
fn Holder(item: &LedgerItem) -> Option<&str>
{
    return item.claim.as_ref().map(|claim| return claim.holder.as_str());
}

/// Who holds an item now and every holder a takeover displaced, as one value.
///
/// The takeover tests all turn on the *pair*: a takeover that installs the new holder while
/// dropping the old one passes an assertion on either field alone, and is exactly the outcome
/// `OD-LEDGER-012` exists to prevent. Comparing the pair is what makes that one failure.
#[derive(Debug, PartialEq, Eq)]
struct Standing<'a>
{
    held_by: Option<&'a str>,
    displaced: Vec<&'a str>,
}

fn Standing_Of(item: &LedgerItem) -> Standing<'_>
{
    return Standing {
        held_by: Holder(item),
        displaced: item
            .displaced
            .iter()
            .map(|claim| return claim.holder.as_str())
            .collect(),
    };
}

/// Two hours after [`AT_NOW`], by which time the one-hour lease these tests take has lapsed.
static AT_LATER: FixedClock = FixedClock(NOW + 7_200);

/// The same board read again once its lease has lapsed.
fn After_The_Lapse(
    directory: &Path,
) -> FileLedger<StdFileSystem, &'static FixedClock, FileLock>
{
    return Ledger_At(directory, &AT_LATER);
}

/// One agent takes a lapsed item over for the standard lease, with the verdict handed back.
///
/// Unlike [`Take`] this returns rather than panics, because both outcomes are subjects here:
/// half these tests are about the takeover succeeding and half about it being refused.
fn Take_Over_In<Clock: nomos_platform::Clock>(
    ledger: &mut FileLedger<StdFileSystem, Clock, FileLock>,
    item: &str,
    holder: &str,
) -> Result<Reservation, ClaimRefusal>
{
    return ledger.Take_Over(&ItemId::New(item), holder, LEASE);
}

/// The same board read again by a ledger standing at a named moment.
///
/// A ledger that owns its clock is what makes a moment one argument. Borrowing one forces
/// every test that moves time to bind the clock first and keep it alive by hand, which is two
/// lines of scaffolding in front of the one number the test is actually varying.
fn Ledger_When(directory: &Path, seconds: i64) -> FileLedger<StdFileSystem, FixedClock, FileLock>
{
    return Ledger_At(directory, FixedClock(seconds));
}

/// A ledger on a fresh temporary directory, already holding the board it starts from.
fn Board_At(
    name: &str,
    items: Vec<LedgerItem>,
) -> (Scratch, FileLedger<StdFileSystem, &'static FixedClock, FileLock>)
{
    let directory = Temp_Dir(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&Document(items)).expect("a fresh ledger is valid");

    return (directory, ledger);
}

fn Ledger_At<Clock: nomos_platform::Clock>(
    directory: &Path,
    clock: Clock,
) -> FileLedger<StdFileSystem, Clock, FileLock>
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
    let (_directory, mut ledger) = Board_At("claim-overlap", vec![
        Item("T-1", &["src/a.rs", "src/shared.rs"]),
        Item("T-2", &["src/shared.rs", "src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Refused(&mut ledger, "T-2", "agent-b");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
    assert!(refusal.Is_Retryable(), "a held item is a queue, not a wall");
    assert!(refusal.Describe().contains("agent-a"));
}

/// The negative control: disjoint territory claims concurrently, which is the entire
/// point of doing any of this.
#[test]
fn Test_Claiming_Disjoint_Territory_Should_Succeed_Concurrently()
{
    let (_directory, mut ledger) = Board_At("claim-disjoint", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");

    ledger.Validate_Current().expect("both claims are legitimate");
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
    let (_directory, mut ledger) = Board_At("pattern-brick", vec![
        Patterned("T-1", &["src/a.rs"], "crates/spec/**"),
        Item("T-2", &["docs/unrelated.md"]),
        Item("T-3", &["tests/also-unrelated.rs"]),
    ]);
    // 1. It claims without complaint. Nothing is held yet, so nothing is compared, so the
    //    pattern is never consulted. This is the step the item's description missed.
    Take(&mut ledger, "T-1", "agent-a");

    // 2. And now the board is shut. `docs/unrelated.md` shares nothing with `src/a.rs` or
    //    with `crates/spec/**`, and is refused anyway — the short-circuit runs before any
    //    path is looked at, so being unrelated is no defence.
    for (item, holder) in [("T-2", "agent-b"), ("T-3", "agent-c")]
    {
        let collateral = Refused(&mut ledger, item, holder);
        Is_Collateral_Damage(item, &collateral);
    }
}

/// A claim refused for no reason of its own: unanswerable rather than contended, and
/// non-retryable, which is what tells the agent to stop and fetch a person. One held pattern
/// therefore reads to every other session as a broken ledger.
fn Is_Collateral_Damage(item: &str, refusal: &ClaimRefusal)
{
    assert!(
        matches!(refusal, ClaimRefusal::UnknownIndependence { .. }),
        "{item}: {refusal:?}"
    );
    assert!(!refusal.Is_Retryable(), "{item}: {}", refusal.Describe());
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
    let (_directory, mut ledger) = Board_At("pattern-brick-reverse", vec![
        Item("T-1", &["docs/unrelated.md"]),
        Patterned("T-2", &["src/b.rs"], "crates/spec/**"),
    ]);
    Take(&mut ledger, "T-1", "agent-a");

    let refused = Refused(&mut ledger, "T-2", "agent-b");

    assert!(matches!(refused, ClaimRefusal::UnknownIndependence { .. }), "{refused:?}");
    assert!(
        !refused.Is_Retryable(),
        "and waiting will not help: `docs/unrelated.md` is disjoint from `src/b.rs`, so the \
         refusal is not contention and no lease expiring resolves it"
    );
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
    let (_directory, mut ledger) = Board_At("refusal-subject", vec![
        Item("T-BLOCKER", &["src/shared.rs"]),
        Item("T-REFUSED", &["src/shared.rs"]),
    ]);

    Take(&mut ledger, "T-BLOCKER", "agent-a");

    Reads_As_A_Statement_About_The_Refused_Item(
        &Refused(&mut ledger, "T-REFUSED", "agent-b").Describe(),
    );
}

/// The three things the sentence has to do, and the line `work audit` composes from it.
fn Reads_As_A_Statement_About_The_Refused_Item(sentence: &str)
{
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
    // information rather than to place it. The composed line is what `work audit` prints:
    // read as English its subject is `T-REFUSED`, and `T-BLOCKER` is what the territory runs
    // into.
    let line = format!("{:<13} {:<9} {sentence}", "T-REFUSED", "held");
    assert!(sentence.contains("T-BLOCKER"), "must still name the blocker: {sentence}");
    assert!(sentence.contains("agent-a"), "and who holds it: {sentence}");
    assert!(
        line.starts_with("T-REFUSED"),
        "the caller names the subject and the description follows it: {line}"
    );
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
    let (_directory, mut ledger) = Board_At("pattern-brick-control", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["docs/unrelated.md"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");
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
    let (_directory, mut ledger) = Board_At("subtree-without-pattern", vec![
        Item("T-1", &["crates/spec"]),
        Item("T-2", &["crates/spec/nomos-spec-model/src/lib.rs"]),
        Item("T-3", &["crates/host/nomos-cli/src/work.rs"]),
    ]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Refused(&mut ledger, "T-2", "agent-b");
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
    Take(&mut ledger, "T-3", "agent-c");
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
    let (directory, mut ledger) = Board_At("finish-fails", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        Exits_With(1),
    )]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Finish_In(&mut ledger, &directory, "T-1", "agent-a")
    .expect_err("a predicate that exits non-zero must refuse the completion");

    assert!(matches!(refusal, FinishRefusal::PredicateFailed { .. }));
    assert!(refusal.Judged_The_Work());

    let after = ledger.Load().expect("readable");
    assert_eq!(
        after.items.first().map(|item| &item.state),
        Some(&ItemState::Claimed),
        "a refused completion must leave the item claimed, not done"
    );
}

/// The negative control. Without it, a `Finish` that refused everything unconditionally
/// would pass the test above.
#[test]
fn Test_Finishing_Should_Succeed_When_The_Predicate_Passes()
{
    let (directory, mut ledger) = Board_At("finish-passes", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        Exits_With(0),
    )]);
    Take(&mut ledger, "T-1", "agent-a");

    let record = Finish_In(&mut ledger, &directory, "T-1", "agent-a")
    .expect("a passing predicate must finish the item");

    assert_eq!(record.exit_code, 0);
    assert_eq!(record.verified_at, At(NOW));

    let finished = Only_Item(&ledger);
    assert_eq!(finished.state, ItemState::Done);
    assert!(
        finished.verified.is_some(),
        "a done item carries the evidence that made it done"
    );
    ledger
        .Validate_Current()
        .expect("a verified done item is a valid ledger");
}

/// An item with nothing to run cannot be shown to be finished. "There was nothing to
/// check" must not read the same as "everything checked out".
#[test]
fn Test_Finishing_Should_Be_Refused_Without_A_Predicate()
{
    let (directory, mut ledger) = Board_At("finish-no-predicate", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Finish_In(&mut ledger, &directory, "T-1", "agent-a")
    .expect_err("an item with no predicate cannot be finished");

    assert!(matches!(refusal, FinishRefusal::NoPredicate { .. }));
    assert!(
        !refusal.Judged_The_Work(),
        "nothing was learned about the work"
    );
}

/// A predicate that cannot be started says nothing about the work. Reporting it as a
/// failed check would tell an author their code is wrong when their tooling is missing.
#[test]
fn Test_An_Unstartable_Predicate_Should_Not_Judge_The_Work()
{
    let (directory, mut ledger) = Board_At("finish-unstartable", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        vec!["nomos-no-such-program-exists".to_owned()],
    )]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Finish_In(&mut ledger, &directory, "T-1", "agent-a")
    .expect_err("a missing program is not a verdict");

    assert!(matches!(refusal, FinishRefusal::CouldNotRun { .. }));
    assert!(!refusal.Judged_The_Work());
}

/// The state transition to `Done` carries its own evidence, so an item cannot arrive
/// there by any route that skipped verification.
#[test]
fn Test_Releasing_As_Finished_Should_Record_The_Verification()
{
    let (_directory, mut ledger) = Board_At("finish-records", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    Release_As_Finished(&mut ledger, "T-1", "agent-a");

    let finished = Only_Item(&ledger);
    assert_eq!(finished.state, ItemState::Done);
    assert_eq!(
        finished.verified.as_ref().map(|record| record.exit_code),
        Some(0)
    );
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
    let (_directory, mut ledger) = Board_At("abandon-records", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    Abandon(&mut ledger, "T-1", "agent-a", REASON);

    let item = Only_Item(&ledger);
    let abandonment = item
        .abandoned
        .first()
        .expect("the abandonment must survive the release that produced it");

    assert_eq!(abandonment.reason, REASON, "the reason the holder gave was not kept");
    assert_eq!(abandonment.holder, "agent-a", "the record does not say who stopped");
    assert_eq!(abandonment.abandoned_at, At(NOW), "the record does not say when");
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
    let (_directory, mut ledger) = Board_At("abandon-releases", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");
    Abandon(&mut ledger, "T-1", "agent-a", REASON);

    let item = Only_Item(&ledger);
    assert_eq!(item.state, ItemState::Ready);
    assert!(item.claim.is_none(), "a claim that survives an abandonment goes on excluding");

    Take(&mut ledger, "T-1", "agent-b");

    let again = Only_Item(&ledger);
    assert_eq!(
        again.abandoned.len(),
        1,
        "the next claim erased the record of the last one"
    );
}

/// Why the field is a list and not the most recent one.
///
/// An item abandoned twice was abandoned twice. Keeping only the latest would discard the
/// earlier reason, which is the loss this whole item is about, one scale down.
#[test]
fn Test_An_Item_Abandoned_Twice_Should_Keep_Both_Reasons()
{
    let (_directory, mut ledger) = Board_At("abandon-twice", vec![Item("T-1", &["src/a.rs"])]);

    for (holder, reason) in [("agent-a", "ran out of lease"), ("agent-b", REASON)]
    {
        ledger
            .Claim(&ItemId::New("T-1"), holder, Duration::from_secs(3_600))
            .expect("an abandoned item is claimable again");
        Abandon(&mut ledger, "T-1", holder, reason);
    }

    let item = Only_Item(&ledger);
    assert_eq!(
        Abandonments(&item),
        vec![("agent-a", "ran out of lease"), ("agent-b", REASON)],
        "both abandonments must survive, oldest first"
    );
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
    let (directory, mut ledger) = Board_At("abandon-lapse", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let lapsed = After_The_Lapse(&directory);
    let item = Only_Item(&lapsed);

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
}

#[test]
fn Test_A_Lease_Beyond_The_Ceiling_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("claim-lease", vec![Item("T-1", &["src/a.rs"])]);

    let refusal = ledger
        .Claim(
            &ItemId::New("T-1"),
            "agent-a",
            nomos_ledger::MAXIMUM_LEASE + Duration::from_secs(1),
        )
        .expect_err("an unbounded lease defeats the ledger");

    assert!(matches!(refusal, ClaimRefusal::LeaseTooLong { .. }));
}

#[test]
fn Test_Renewing_Someone_Elses_Claim_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("renew-foreign", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = ledger
        .Renew(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err("renewing another holder's claim must be refused");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
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
    let ledger = Ledger_At(&directory, &AT_NOW);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a corrupt ledger must not read as empty");

    assert!(matches!(error, LedgerError::Malformed { .. }));
}

#[test]
fn Test_A_Missing_Ledger_Should_Read_As_Empty()
{
    let directory = Temp_Dir("missing");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let document = ledger.Load().expect("a missing ledger is not an error");

    assert!(document.items.is_empty());
}

/// An invalid document must be refused *before* it is written. A ledger that is written
/// and then found invalid is one somebody has to repair by hand, and until they do
/// every agent is reading something the system itself says is wrong.
#[test]
fn Test_Saving_An_Invalid_Ledger_Should_Be_Refused_Before_The_Write()
{
    let directory = Temp_Dir("refuse-invalid");
    let ledger = Ledger_At(&directory, &AT_NOW);

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
}

/// Round-tripping must be lossless. If it is not, an agent's claim silently changes
/// meaning the next time somebody else writes the file.
#[test]
fn Test_The_Ledger_Should_Round_Trip_Losslessly()
{
    let directory = Temp_Dir("round-trip");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let original = Document(vec![
        Held_By(Item("T-1", &["src/a.rs", "src/b.rs"]), "agent-a", NOW + 3_600),
        Item("T-2", &["src/c.rs"]),
    ]);

    ledger.Save(&original).expect("valid");
    let reloaded = ledger.Load().expect("readable");

    assert_eq!(reloaded, original);
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
    let ledger = Ledger_At(&directory, &AT_NOW);

    let raw = Raw_Ledger(
        SCHEMA_VERSION,
        ",\"a_field_this_build_does_not_know\":{\"holder\":\"agent-a\"}",
    );
    std::fs::write(directory.join("ledger.json"), raw).expect("write");

    let error = ledger
        .Load()
        .expect_err("a document carrying a key this build cannot account for must not load");

    assert!(
        format!("{error}").contains("a_field_this_build_does_not_know"),
        "the refusal must name the key it could not account for: {error}"
    );
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
    let whole =
        serde_json::to_value(Document(vec![Fully_Populated()])).expect("the document serializes");

    let mut pointers = Vec::new();
    Object_Pointers(&whole, "", &mut pointers);
    for pointer in &pointers
    {
        assert!(
            Probed(&whole, pointer).is_err(),
            "an undeclared key was accepted at `{pointer}`, so a build that predates a field \
             there would drop it and write the document back"
        );
    }

    assert!(
        pointers.len() >= 11,
        "only {} object(s) were probed, so the fixture below has stopped being fully \
         populated — the guard did not shrink, the universe did",
        pointers.len()
    );
}

/// The same document with one undeclared key inserted at `pointer`, read back strictly.
fn Probed(whole: &serde_json::Value, pointer: &str) -> Result<LedgerDocument, serde_json::Error>
{
    let mut probed = whole.clone();
    probed
        .pointer_mut(pointer)
        .expect("every pointer was collected from this same value")
        .as_object_mut()
        .expect("only object nodes were collected")
        .insert("nomos_probe".to_owned(), serde_json::Value::Null);

    return serde_json::from_value::<LedgerDocument>(probed);
}

/// An item with every optional field set and every list non-empty.
///
/// The walk above reaches an object only if the document actually contains one, so an empty
/// list serializes as `[]`, contributes no node, and leaves the guard silent about whatever
/// type lives inside it. Populating everything is what makes the count at the end mean
/// something.
fn Fully_Populated() -> LedgerItem
{
    let held = Item("T-1", &["src/a.rs"]);
    let mut item = Held_By(held, "agent-a", NOW + 3_600);
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
    // The nested type `OD-LEDGER-012` added.
    item.displaced = vec![Claim {
        holder: "dead-agent".to_owned(),
        acquired_at: At(NOW),
        lease_expires_at: At(NOW + 60),
    }];

    return item;
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
    // The spliced key was `"displaced":[]` when this test was written, chosen as a field a
    // later build might add. `OD-LEDGER-012` added it, so it became declared and the parse it
    // was here to make fail started succeeding. Named for what it is instead, which is the
    // same lesson `Raw_Ledger`'s own fixture learned: a probe key must not be one the schema
    // can catch up with.
    let raw = Raw_Ledger(9_999, ",\"a_field_this_build_does_not_know\":[]");
    let error = Load_Failure("newer-than-build", &raw);

    let LedgerError::Unrecognized {
        understood, found, ..
    } = &error
    else
    {
        panic!("a file newer than this build must not be reported as damaged: {error}");
    };
    assert_eq!(*understood, SCHEMA_VERSION);
    assert_eq!(*found, 9_999);
    Names_The_Versions_And_The_Remedy(&format!("{error}"));
}

/// Both numbers and the action, because a message naming only one of them leaves the reader
/// with nothing to compare and no next step.
fn Names_The_Versions_And_The_Remedy(said: &str)
{
    assert!(said.contains("9999"), "{said}");
    assert!(said.contains(&format!("{SCHEMA_VERSION}")), "{said}");
    assert!(
        said.to_lowercase().contains("rebuild"),
        "the message must name the remedy: {said}"
    );
}

/// What `Load` says about a document written by hand.
///
/// These files are the ones `Save` refuses to produce, so writing the bytes directly is the
/// only way to reach the arm under test — and the tree goes away with the value returned.
fn Load_Failure(name: &str, raw: &str) -> LedgerError
{
    let directory = Temp_Dir(name);
    std::fs::write(directory.join("ledger.json"), raw).expect("write");

    return Ledger_At(&directory, &AT_NOW)
        .Load()
        .expect_err("this document must not load");
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
    let ledger = Ledger_At(&directory, &AT_NOW);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a broken document must not load");

    assert!(
        matches!(error, LedgerError::Malformed { .. }),
        "a damaged file must not be reported as one this build is too old for: {error}"
    );
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
    let ledger = Ledger_At(&directory, &AT_NOW);

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

    let document = Ledger_At(&directory, &AT_NOW)
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
}

/// A dependency edge that only `validate` reads is a comment. Claiming has to refuse an
/// item whose prerequisite is unfinished, or the ordering is advice.
#[test]
fn Test_Claiming_An_Item_With_An_Unfinished_Dependency_Should_Be_Refused()
{
    let directory = Temp_Dir("claim-dependency");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"]), dependent]))
        .expect("a fresh ledger is valid");

    let refusal = Refused(&mut ledger, "T-2", "agent-a");

    assert!(matches!(refusal, ClaimRefusal::DependencyUnmet { .. }), "{}", refusal.Describe());
    assert!(refusal.Is_Retryable(), "finishing T-1 is what resolves this");
    assert!(refusal.Describe().contains("T-1"), "{}", refusal.Describe());
}

/// The negative control. A satisfied dependency must not stand in the way.
#[test]
fn Test_A_Finished_Dependency_Should_Not_Block_A_Claim()
{
    let directory = Temp_Dir("claim-dependency-met");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let finished = Finished("T-1", &["src/a.rs"]);
    let mut dependent = Item("T-2", &["src/b.rs"]);
    dependent.depends_on = vec![ItemId::New("T-1")];

    ledger
        .Save(&Document(vec![finished, dependent]))
        .expect("a fresh ledger is valid");

    Take(&mut ledger, "T-2", "agent-a");
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
    let (directory, mut ledger) = Board_At("lapse-bricks", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ]);
    Take(&mut ledger, "T-1", "dead-agent");

    let mut after = After_The_Lapse(&directory);

    // One: the document is not called invalid because time passed.
    assert_eq!(
        Validate(&after.Load().expect("the file is still readable"), At(NOW + 7_200)),
        Vec::<String>::new(),
        "a lapsed lease made the whole document invalid, so nothing can be written to it"
    );
    after
        .Validate_Current()
        .expect("validate must not call a board with a lapsed lease broken");
    // Two: an unrelated item is still claimable. `src/b.rs` shares nothing with `src/a.rs`,
    // so a refusal here is not exclusion — it is the board refusing to be written at all.
    Take(&mut after, "T-2", "agent-b");
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
    let (directory, mut ledger) = Board_At("lapse-takeover", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "dead-agent");

    let mut after = After_The_Lapse(&directory);

    let refusal = Refused(&mut after, "T-1", "agent-b");
    Says_The_Item_Is_Lapsed_And_Names_The_Verb(&refusal);

    assert_eq!(
        Standing_Of(&Only_Item(&after)),
        Standing {
            held_by: Some("dead-agent"),
            displaced: Vec::new(),
        },
        "the lapsed claim was replaced, or a refused claim recorded a displacement — either \
         way `claim` has quietly become `takeover`"
    );
}

/// The three things the refusal has to say, and each is a different way of getting it wrong:
/// blaming territory, giving no remedy, or advising a wait that will never end.
fn Says_The_Item_Is_Lapsed_And_Names_The_Verb(refusal: &ClaimRefusal)
{
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
    let (directory, mut ledger) = Board_At("lapse-recover", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let mut after = After_The_Lapse(&directory);

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
    let (directory, mut ledger) = Board_At("takeover-keeps-predecessor", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "dead-agent");

    let mut after = After_The_Lapse(&directory);

    let reservation = Take_Over_In(&mut after, "T-1", "agent-b").unwrap_or_else(|refusal| {
        panic!(
            "an item whose holder died must return to the pool without a person editing the \
             file: {}",
            refusal.Describe()
        )
    });
    assert_eq!(reservation.holder, "agent-b");

    let item = Only_Item(&after);
    Installed_The_Taker(&item, At(NOW + 7_200));
    Kept_The_Claim_It_Replaced(&item);
}

/// The takeover installed its new holder, on a live lease, without moving the state.
///
/// Three assertions rather than one because each names a different way the verb could be
/// half-done, and a takeover that leaves the lease in the past has taken nothing.
fn Installed_The_Taker(item: &LedgerItem, now: Timestamp)
{
    assert_eq!(
        Holder(item),
        Some("agent-b"),
        "the takeover did not install the new holder"
    );
    assert!(
        item.Has_Active_Claim(now),
        "a takeover that leaves the lease in the past has taken nothing"
    );
    assert_eq!(
        item.state,
        ItemState::Claimed,
        "a takeover does not move the state; the item was claimed and still is"
    );
}

/// The half `P10-LAPSE-TAKEOVER` exists for: who held it, when they took it, and when the
/// lease ran out all survive, and they survive as the claim itself rather than as a summary.
fn Kept_The_Claim_It_Replaced(item: &LedgerItem)
{
    let displaced = item.displaced.first().unwrap_or_else(|| {
        panic!(
            "the takeover kept {} displaced claim(s) rather than exactly the one it replaced, \
             so nothing records who walked away from this",
            item.displaced.len()
        )
    });

    assert_eq!(item.displaced.len(), 1, "something records it twice");
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
    let (directory, mut ledger) = Board_At("takeover-refuses-live", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let mut during = Ledger_When(&directory, NOW + 60);

    let refusal = Take_Over_In(&mut during, "T-1", "agent-b")
        .expect_err("a live claim must not be displaced by a takeover");
    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }), "{}", refusal.Describe());
    assert!(
        refusal.Is_Retryable(),
        "the lease running out is what resolves this, so waiting is the honest advice"
    );

    assert_eq!(
        Standing_Of(&Only_Item(&during)),
        Standing {
            held_by: Some("agent-a"),
            displaced: Vec::new(),
        },
        "a takeover displaced a holder who was still working"
    );
}

/// The wrong verb is refused rather than accommodated.
///
/// A `takeover` that quietly worked as a `claim` would mean two verbs doing one thing, and the
/// whole point of a separate verb is that it means something the other does not. Both ends of
/// the range are covered: an item nobody has ever held, and one that is finished.
#[test]
fn Test_Taking_Over_An_Item_Nobody_Holds_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("takeover-wrong-verb", vec![
        Item("T-1", &["src/a.rs"]),
        Finished("T-2", &["src/b.rs"]),
    ]);
    for (item, what) in [("T-1", "an item nobody holds"), ("T-2", "a finished item")]
    {
        let refusal = Take_Over_In(&mut ledger, item, "agent-b")
            .expect_err("a takeover answers a lapse and nothing else");

        assert!(
            matches!(refusal, ClaimRefusal::NotClaimable { .. }),
            "{what} must read as the wrong verb rather than as a queue: {}",
            refusal.Describe()
        );
        assert!(!refusal.Is_Retryable(), "{what} will not become takeable by waiting");
    }

    assert!(
        ledger
            .Load()
            .expect("readable")
            .items
            .iter()
            .all(|item| return item.claim.is_none() && item.displaced.is_empty()),
        "a refused takeover wrote to the item anyway"
    );
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
    // Two items over the same file. Concurrently claimable only while one of them is not.
    let (directory, mut ledger) = Board_At("takeover-refuses-taken-ground", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/a.rs"]),
    ]);
    Take(&mut ledger, "T-1", "dead-agent");
    // The second claim succeeds precisely because T-1's lapsed claim no longer excludes. That
    // is the state the takeover then has to notice.
    let mut after = After_The_Lapse(&directory);
    Take(&mut after, "T-2", "agent-b");

    let refusal = Take_Over_In(&mut after, "T-1", "agent-c")
        .expect_err("the ground T-1 reserves is held by a live claim on T-2");
    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }), "{}", refusal.Describe());
    assert!(
        refusal.Describe().contains("agent-b"),
        "the refusal must name who holds the ground now: {}",
        refusal.Describe()
    );
    assert_eq!(
        Standing_Of(&Named("T-1", &after)),
        Standing {
            held_by: Some("dead-agent"),
            displaced: Vec::new(),
        },
        "a refused takeover wrote to the item anyway"
    );
}

/// A list, not a field.
///
/// An item taken over twice was taken over twice, and keeping only the most recent would
/// discard the earlier holder — `OD-LEDGER-006`'s reason for `abandoned` being a list, restated
/// one field across. This is the test that a single-slot implementation passes B1 and fails.
#[test]
fn Test_An_Item_Taken_Over_Twice_Should_Name_Both_Predecessors()
{
    let (directory, mut ledger) = Board_At("takeover-twice", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "dead-agent");

    let mut takes = Ledger_When(&directory, NOW + 7_200);
    Take_Over_In(&mut takes, "T-1", "agent-b").expect("the first holder's lease ran out");

    // Past `agent-b`'s lease too: NOW + 7_200 + 3_600.
    let mut again = Ledger_When(&directory, NOW + 14_400);
    Take_Over_In(&mut again, "T-1", "agent-c").expect("the second holder's lease ran out as well");

    assert_eq!(
        Standing_Of(&Only_Item(&again)),
        Standing {
            held_by: Some("agent-c"),
            displaced: vec!["dead-agent", "agent-b"],
        },
        "oldest first, and both of them: keeping only the most recent is this same loss one \
         scale down"
    );
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
    let mut ledger = Ledger_At(&directory, &AT_NOW);
    let path = Write_A_Claimed_Item_With_No_Claim(&directory);
    let before = std::fs::read(&path).expect("readable");

    let refusal = Take_Over_In(&mut ledger, "T-1", "agent-b")
        .expect_err("an item whose predecessor record is already missing is not taken over");
    assert!(matches!(refusal, ClaimRefusal::NotClaimable { .. }), "{}", refusal.Describe());
    assert_eq!(
        std::fs::read(&path).expect("readable"),
        before,
        "a refused takeover rewrote the file"
    );
}

/// The one corruption `Validate` still refuses, written by hand because nothing else can
/// produce it: `Save` will not persist it, and `Load` does not validate what it reads.
fn Write_A_Claimed_Item_With_No_Claim(directory: &Path) -> PathBuf
{
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

    return path;
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
    let (directory, mut ledger) = Board_At("takeover-round-trip", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "dead-agent");

    let mut after = After_The_Lapse(&directory);
    Take_Over_In(&mut after, "T-1", "agent-b").expect("a lapsed item is takeable");

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
    // A genuinely missing item over a ledger that is fine, and then the same call over a file
    // that is not a ledger at all.
    let (directory, mut sound) = Board_At("unusable-ledger", vec![Item("T-1", &["src/a.rs"])]);
    let missing = Refused(&mut sound, "T-NOPE", "agent-a");

    std::fs::write(directory.join("ledger.json"), "{ not json").expect("writes the corruption");
    let mut broken = Ledger_At(&directory, &AT_NOW);
    let unusable = Refused(&mut broken, "T-1", "agent-a");
    Told_Apart(&missing, &unusable);
}

/// The two causes must not read the same, which is the whole defect: every load and save
/// failure used to discard its error and answer `NoSuchItem`, sending an operator whose file
/// was damaged off to check a spelling.
fn Told_Apart(missing: &ClaimRefusal, unusable: &ClaimRefusal)
{
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
    assert_ne!(missing.Describe(), unusable.Describe(), "the two causes read the same");
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
    name: &str,
    items: Vec<LedgerItem>,
    first: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
    second: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
) -> LedgerDocument
{
    let directory = Contended(name, items);
    let filesystem = Interleaving::Over(directory.join("ledger.json"));

    let over = Harness {
        shared: &filesystem,
        finished: &Gate::New(),
        directory: &directory,
    };

    Interleaved(over, first, second);
    assert!(
        filesystem.Stopped(),
        "the seam never fired, so nothing was interleaved and this run proves nothing"
    );

    return Written(&directory);
}

/// The apparatus both writers share: the filesystem carrying the seam, the gate the second
/// writer opens when it is through, and the tree they are both writing.
#[derive(Clone, Copy)]
struct Harness<'a>
{
    shared: &'a Interleaving,
    finished: &'a Gate,
    directory: &'a Path,
}

/// The scope itself: `second` starts once `first` has read, and `first` writes once `second`
/// has either finished or been kept waiting for [`SECOND_WRITER_LIMIT`].
fn Interleaved(
    over: Harness<'_>,
    first: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
    second: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
)
{
    std::thread::scope(|scope| {
        scope.spawn(move || {
            over.shared.Hold_This_Thread();
            let mut ledger = Ledger_Over(over.shared, over.directory, &AT_NOW);
            first(&mut ledger);
        });

        over.shared.read.Wait();

        scope.spawn(move || {
            let mut ledger = Ledger_Over(over.shared, over.directory, &AT_NOW);
            second(&mut ledger);
            over.finished.Open();
        });

        over.finished.Opened_Within(SECOND_WRITER_LIMIT);
        over.shared.resume.Open();
    });
}

/// A board for the interleaving tests, saved once before either writer starts.
fn Contended(name: &str, items: Vec<LedgerItem>) -> Scratch
{
    let directory = Temp_Dir(name);
    Ledger_At(&directory, &AT_NOW)
        .Save(&Document(items))
        .expect("a fresh ledger is valid");

    return directory;
}

/// What is actually on disk once both writers have finished — the only thing that settles
/// whether a write was lost, since each writer was told its own succeeded.
fn Written(directory: &Path) -> LedgerDocument
{
    return Ledger_At(directory, &AT_NOW).Load().expect("readable");
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
    let after = Two_Writers(
        "concurrent-claims",
        vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            Take(ledger, "T-1", "agent-a");
        },
        |ledger| {
            Take(ledger, "T-2", "agent-b");
        },
    );

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
    let after = Two_Writers(
        "concurrent-release",
        vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 3_600),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            Abandon(ledger, "T-1", "agent-a", REASON);
        },
        |ledger| {
            Take(ledger, "T-2", "agent-b");
        },
    );

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
}

/// And at the verb an agent runs most often, which is the one that hides best.
///
/// A renewal that is lost does not look like a lost write. It looks like a lease that ran out
/// early, which reads as an agent that died — so the wrong thing gets investigated.
#[test]
fn Test_A_Renewal_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let after = Two_Writers(
        "concurrent-renew",
        vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", NOW + 10),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            ledger
                .Renew(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
                .expect("a holder may renew its own claim");
        },
        |ledger| {
            Take(ledger, "T-2", "agent-b");
        },
    );

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
}

/// The same loss at the verb that puts work on the board, which `OD-LEDGER-015` left behind.
///
/// `Claim`, `Renew` and `Release` were moved behind the lock and `add` was not, because the
/// three were named as "the verbs that change the board" and adding an item was not counted
/// as changing it. It is: the document `add` writes back is the whole board, so an add that
/// read before somebody else's claim erases that claim exactly as a stale `Claim` would.
///
/// Observed on the real ledger rather than reasoned about. Two adds ran back to back, both
/// printed their success line and both exited 0, and only the first was ever on the board —
/// a peer's locked verb had read the file between them and written its snapshot back over
/// the second. `OD-LEDGER-021`.
#[test]
fn Test_An_Add_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let after = Two_Writers(
        "concurrent-add",
        vec![Item("T-2", &["src/b.rs"])],
        |ledger| {
            let item = Item("T-1", &["src/a.rs"]);
            ledger
                .Add(&item, "agent-a", &ItemTerritory::Empty(), &ItemTerritory::Empty())
                .expect("T-1 is not on the board yet");
        },
        |ledger| {
            Take(ledger, "T-2", "agent-b");
        },
    );

    assert!(
        after
            .items
            .iter()
            .any(|item| return item.id == ItemId::New("T-1")),
        "the add was told the item was recorded and the item is not on the board: an add \
         wrote back a document it had read before the other writer existed, or was written \
         over by one"
    );
    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "an add wrote back a document read before the other writer's claim, so the claim it \
         was granted is gone"
    );
}

/// The duplicate check has to travel inside the lock with the write it guards.
///
/// Deciding it outside is the same defect one level down, and it is worse than a lost item:
/// two sessions adding one identifier both read a board without it, both are told it is
/// theirs, and the document that results has the identifier twice. [`Validate`] calls that
/// invalid and every operation loads before it does anything, so the next agent to touch the
/// board — any agent, on any item — is refused by a ledger that will not load. Two callers
/// were each told they succeeded and the board is unusable.
///
/// The interleaving is the one the harness always builds, and what it proves is different
/// here: the second writer is not merely made to wait, it is made to *see* the first writer's
/// item and refuse on it.
#[test]
fn Test_Two_Concurrent_Adds_Of_One_Identifier_Should_Not_Both_Be_Accepted()
{
    let second_outcome = Mutex::new(None);
    let recorded = &second_outcome;
    let after = Two_Writers(
        "concurrent-duplicate-add",
        Vec::new(),
        |ledger| {
            Adds_T_1(ledger, "src/a.rs", "agent-a").expect("the board is empty, so T-1 is free");
        },
        |ledger| {
            let outcome = Adds_T_1(ledger, "src/b.rs", "agent-b");
            *recorded.lock().expect("the harness never panics under this lock") = Some(outcome);
        },
    );

    assert_eq!(
        second_outcome
            .into_inner()
            .expect("the harness never panics under this lock")
            .expect("the second writer ran"),
        Err(AddRefusal::AlreadyPresent {
            item: ItemId::New("T-1")
        }),
        "the second add read the board before the first one's write and was told an \
         identifier that was already taken was free"
    );
    assert_eq!(
        after.items.iter().filter(|item| return item.id == ItemId::New("T-1")).count(),
        1,
        "one identifier is on the board twice, so the board no longer loads for anybody"
    );
    assert!(
        Validate(&after, At(NOW)).is_empty(),
        "the board two accepted adds left behind is one the ledger itself calls invalid"
    );
}

/// Both writers add `T-1`; only the territory and the holder differ, which is what makes the
/// second one's refusal a statement about the identifier rather than about the files.
fn Adds_T_1(
    ledger: &mut InterleavedLedger<'_>,
    file: &str,
    holder: &str,
) -> Result<(), AddRefusal>
{
    let item = Item("T-1", &[file]);

    return ledger.Add(&item, holder, &ItemTerritory::Empty(), &ItemTerritory::Empty());
}

/// An item the board cannot hold is the caller's to correct, not a broken store.
///
/// The two are different exit codes and opposite next actions — fix your item, or stop and
/// fetch a person — so routing `add` through the store had to keep them apart. Carrying
/// [`LedgerError::Invalid`] out as [`AddRefusal::LedgerUnusable`] with everything else would
/// have turned "your territory is empty" into "the ledger is unusable", which is
/// `OD-LEDGER-009`'s conflation arriving by a new route.
///
/// An empty territory is the instance that actually happens: `AGENTS.md` states it as a rule
/// of the board, so it is the refusal an author hits by writing a plausible item.
#[test]
fn Test_An_Item_That_Would_Not_Validate_Should_Refuse_As_The_Authors_Mistake()
{
    let (_directory, mut ledger) = Board_At("add-would-not-validate", Vec::new());

    let item = Item("T-1", &[]);
    let refused =
        ledger.Add(&item, "agent-a", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    assert!(
        matches!(refused, Err(AddRefusal::WouldBeInvalid { .. })),
        "an item that reserves nothing is a violation of the board's own rules, and \
         reporting it as an unusable store sends its author to the wrong remedy: {refused:?}"
    );
    assert!(
        ledger
            .Load()
            .expect("readable")
            .items
            .is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

// ---------------------------------------------------------------------------
// A record identifier is allocated once, and `add` is where that is enforced.
// ---------------------------------------------------------------------------

/// An item reserving `docs/records/<ID>` alongside whatever else it touches.
fn Reserving_Record(id: &str, identifier: &str) -> LedgerItem
{
    return Item(id, &["crates/a/src/lib.rs", identifier]);
}

/// A published record, spelled as the file it actually is rather than as its identifier.
fn Published(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().map(|file| return (*file).to_owned()));
}

/// A published identifier reserved without declaring an amendment is refused, by its file.
///
/// The half nothing could have caught. A published identifier excludes nobody, so
/// `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` — which reddens on two *open*
/// items sharing one — is blind to it by construction. What happened instead is that the
/// author found the identifier taken mid-claim, with no `work edit` to move it.
///
/// What this asserts narrowed when the declaration arrived, and the assertion did not have to
/// change: the item here declares nothing, so it is allocating, and an allocation onto a spent
/// number is still the defect this was written for. The case it no longer covers is the one
/// directly below.
#[test]
fn Test_A_Record_Identifier_Already_Published_Should_Be_Refused_By_Its_File()
{
    let (_directory, mut ledger) = Board_At("add-record-published", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &ItemTerritory::Empty(),
    );

    let Err(AddRefusal::RecordPublished { identifier, file }) = refused
    else
    {
        panic!("a spent identifier must be refused as spent: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-006");
    assert_eq!(file, "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md");
    assert!(
        ledger.Load().expect("readable").items.is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

/// The negative control. An unspent identifier is still accepted, beside published ones.
///
/// Without it every assertion above is satisfied by an `add` that refuses everything, and
/// the guard would be indistinguishable from a broken one on the day it mattered.
#[test]
fn Test_An_Unspent_Record_Identifier_Should_Still_Be_Accepted()
{
    let (_directory, mut ledger) = Board_At("add-record-unspent", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-007");
    let added = ledger.Add(
        &item,
        "agent-a",
        &Published(&[
            "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md",
            // The ordinal is compared as a whole component, so this must not make `007`
            // look taken. `0071` is not `007`, and a prefix rule would say it is.
            "docs/records/OD-LEDGER-0071-something-else.md",
        ]),
        &ItemTerritory::Empty(),
    );

    assert_eq!(added, Ok(()), "the next free identifier is free");
    assert_eq!(ledger.Load().expect("readable").items.len(), 1);
}

/// What the item says it is editing rather than allocating.
///
/// The same shape as [`Published`] and deliberately a different name at the call site: the
/// two arguments are both record files and mean opposite things, and a reader who sees
/// `Published` twice has to work out which one is the claim and which is the repository.
fn Amending(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().map(|file| return (*file).to_owned()));
}

/// A published record reserved for amendment is accepted, in either spelling.
///
/// The clause this item exists for. `ARC-ECOSYSTEM-001` is at version 2 and
/// `OD-CAPABILITY-001` at version 2, so amending a record is ordinary work here, and an
/// amendment must reserve the file it edits because territory is the only thing keeping two
/// writers off one file. Both spellings are driven, because `OD-LEDGER-016` makes the
/// identifier and its file one subject and an author who had to guess which one `--amends`
/// wanted would be following a convention rather than a rule.
#[test]
fn Test_A_Published_Record_Declared_As_An_Amendment_Should_Be_Accepted()
{
    const FILE: &str = "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md";

    for (described, spelled) in [
        ("the bare identifier", "docs/records/OD-LEDGER-006"),
        ("the published filename", FILE),
    ]
    {
        let (_directory, mut ledger) = Board_At("add-record-amended", Vec::new());
        let item = Reserving_Record("T-1", spelled);

        assert_eq!(
            ledger.Add(&item, "agent-a", &Published(&[FILE]), &Amending(&[spelled])),
            Ok(()),
            "an amendment declared by {described} was refused as an allocation"
        );
        assert_eq!(ledger.Load().expect("readable").items.len(), 1);
    }
}

/// Declaring an amendment does not exempt the record from the open-item comparison.
///
/// The half a reader is most likely to expect the other way round. Two items amending one
/// record are two writers on one file, which is exactly what territory serializes, so the
/// declaration answers *which act this is* and never *whether somebody else is already doing
/// it*. Asserted separately from the acceptance above rather than folded into it, because one
/// assertion covering both is satisfied by an `add` that ignores the declaration entirely.
#[test]
fn Test_An_Amendment_Should_Not_Exempt_A_Record_Another_Open_Item_Reserves()
{
    let (_directory, mut ledger) = Board_At("add-record-amend-contended", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-006",
    )]);

    let item = Reserving_Record("T-2", "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-b",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Amending(&["docs/records/OD-LEDGER-006"]),
    );

    let Err(AddRefusal::RecordReserved { identifier, item }) = refused
    else
    {
        panic!("a second amender must still be refused by the first: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-006");
    assert_eq!(item, ItemId::New("T-1"));
}

/// An amendment declared against a record nobody has published is refused.
///
/// Without this the declaration would be the cheapest way to defeat the guard the whole
/// check exists to be: declare every reservation an amendment and no identifier is ever
/// spent again. It is also the ordinary mistake — an author who mistyped the number is told
/// so here rather than getting an item that allocates while saying it amends.
#[test]
fn Test_A_Declared_Amendment_Of_An_Unpublished_Record_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("add-record-amend-absent", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-099");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Amending(&["docs/records/OD-LEDGER-099"]),
    );

    let Err(AddRefusal::AmendmentNotPublished { identifier }) = refused
    else
    {
        panic!("an amendment of nothing must be refused as such: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-099");
    assert!(
        ledger.Load().expect("readable").items.is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

/// A record identifier another open item reserves is refused, and that item is named.
///
/// Five collisions bought this. Three open items reserved `OD-LEDGER-020` and two reserved
/// `OD-LEDGER-021`, each authored by a session taking the next free number; clearing them
/// meant declining and re-authoring four items, because there is no `work edit`.
#[test]
fn Test_A_Record_Identifier_Another_Open_Item_Reserves_Should_Be_Refused_By_Its_Item()
{
    let (_directory, mut ledger) = Board_At("add-record-reserved", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-020",
    )]);

    // The second author writes the identifier's file spelling rather than its bare form.
    // `OD-LEDGER-016` makes those one subject, so this must still be refused — an author
    // who reserved the file they were about to write has taken the identifier.
    let item = Reserving_Record("T-2", "docs/records/OD-LEDGER-020-the-same-number.md");
    let refused =
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    let Err(AddRefusal::RecordReserved { identifier, item }) = refused
    else
    {
        panic!("an identifier another open item holds must be refused as held: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-020");
    assert_eq!(item, ItemId::New("T-1"));
}

/// The two refusals are different values, because the remedies are different.
///
/// Choosing another identifier fixes one; the other may resolve itself when the item
/// holding it is retired. An author told only "taken" picks the wrong remedy half the time,
/// which is the mis-subject `OD-LEDGER-014` measured one verb over.
#[test]
fn Test_A_Published_Identifier_And_A_Reserved_One_Should_Be_Different_Refusals()
{
    let (_directory, mut ledger) = Board_At("add-record-distinct", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-020",
    )]);

    let holding = Reserving_Record("T-2", "docs/records/OD-LEDGER-020");
    let publishing = Reserving_Record("T-3", "docs/records/OD-LEDGER-006");
    let reserved =
        ledger.Add(&holding, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());
    let published = ledger.Add(
        &publishing,
        "agent-b",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &ItemTerritory::Empty(),
    );

    assert_ne!(reserved, published);
    assert_ne!(
        reserved.as_ref().err().map(AddRefusal::Describe),
        published.as_ref().err().map(AddRefusal::Describe),
        "two refusals with one sentence send both authors to one remedy"
    );
}

/// A closed item's territory is history, not a reservation.
///
/// The rule that keeps this guard from refusing the whole board: almost every item ever
/// finished reserved a record, so counting closed items would make every allocated number a
/// permanent claim and the next author could allocate nothing at all.
#[test]
fn Test_A_Closed_Items_Record_Reservation_Should_Not_Reserve_Anything()
{
    let directory = Temp_Dir("add-record-closed");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let closed_states = [
        ItemState::Done,
        ItemState::Declined {
            reason: "it turned out not to be work".to_owned(),
        },
    ];

    for state in closed_states
    {
        Allocates_Over(&mut ledger, state);
    }
}

/// The board holds one closed item reserving a record, and the next author allocates it.
fn Allocates_Over<Clock: nomos_platform::Clock>(
    ledger: &mut FileLedger<StdFileSystem, Clock, FileLock>,
    state: ItemState,
)
{
    const RECORD: &str = "docs/records/OD-LEDGER-020";

    let described = format!("{state:?}");
    ledger
        .Save(&Document(vec![Closed_Reserving(RECORD, state)]))
        .expect("valid");

    let item = Reserving_Record("T-2", RECORD);
    assert_eq!(
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty()),
        Ok(()),
        "a {described} item's reservation outlived it, so the number is claimed forever"
    );
}

/// An item in a closed state, carrying whatever that state's own invariants require.
///
/// A `Done` item with no verification and a `Declined` one with no declination are both
/// refused by the board before this test's subject is ever reached, so each arm has to be
/// completed here — but the completing is scaffolding, not what the test is about.
fn Closed_Reserving(record: &str, state: ItemState) -> LedgerItem
{
    let mut closed = Reserving_Record("T-1", record);
    if matches!(state, ItemState::Declined { .. })
    {
        closed.declined = Some(Declination {
            holder: "agent-a".to_owned(),
            declined_at: At(NOW),
        });
    }
    else
    {
        closed.verified = Some(VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: "test result: ok".to_owned(),
            verified_at: At(NOW),
            gate: None,
        });
    }
    closed.state = state;

    return closed;
}

/// Ordinary overlapping territory is still accepted, and that is deliberate.
///
/// `add` does not refuse shared territory in general and must not start: items overlap
/// constantly and claims are what serialize them. `P10-ADD-PROMISE` narrowed the doc comment
/// that once promised otherwise. What is guarded is only the reservation an author cannot
/// recover from mid-claim.
#[test]
fn Test_Ordinary_Shared_Territory_Should_Still_Be_Accepted()
{
    let (_directory, mut ledger) = Board_At("add-shared-territory", vec![Item("T-1", &["crates/a/src/lib.rs"])]);

    let item = Item("T-2", &["crates/a/src/lib.rs"]);
    let added =
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    assert_eq!(
        added,
        Ok(()),
        "two items may reserve one path; a claim is what decides who holds it"
    );
}
