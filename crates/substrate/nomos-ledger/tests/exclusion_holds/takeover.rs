//! Recovering an item whose holder is gone, and refusing the ones that are not.
//!
//! A takeover displaces a claim and records what it displaced, so the board can still say
//! who held the item before. An item nobody holds has nothing to take over, and that
//! refusal is a different one from the item that is merely unusable.

use crate::board::{
    At, AT_LATER_SECONDS, AT_NOW, After_The_Lapse, Board, Board_At, ClaimRefusal, Claimant, Document, Finished,
    Held_By, Item, ItemState, LEASE_SECONDS, Ledger_At, Ledger_When, NOW, Named, Only_Item, Path, PathBuf, Refused,
    SCHEMA_VERSION, Standing, Standing_Of, Take, Take_Over_In, Temporary_Directory, Validate_Document,
};

/// A moment a minute after the claim was taken, and so well inside its lease: the holder is
/// still working, and a takeover here would be taking an item away from somebody who has not
/// stopped.
const STILL_WITHIN_THE_LEASE: i64 = NOW + 60;

/// A moment later than the lease the first takeover installed — the first lapsed at
/// [`AT_LATER_SECONDS`], so the second has to be a whole lease beyond that moment again.
const AFTER_THE_SECOND_LEASE: i64 = AT_LATER_SECONDS + LEASE_SECONDS + LEASE_SECONDS;

/// A live claim is not a lapse, and `takeover` is not a way to steal work in progress.
///
/// Without this the verb is worse than the defect it fixes: an agent that is merely slow gets
/// its item taken by anybody who types the word. The lease is the whole of the protection and
/// renewing it is the whole of the remedy, which is why `HeldBy` here is retryable — waiting
/// is genuinely the right advice when somebody is working.
#[test]
fn Test_An_Item_With_A_Live_Claim_Should_Not_Be_Taken_Over()
{
    let Board { directory, mut ledger } = Board_At("takeover-refuses-live", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let mut during = Ledger_When(directory.As_Path(), STILL_WITHIN_THE_LEASE);

    let refusal = Take_Over_In(&mut during, "T-1", Claimant("agent-b"))
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
            widened: Vec::new(),
        },
        "a takeover displaced a holder who was still working"
    );
}

/// The wrong verb is refused rather than accommodated.
///
/// A `takeover` that quietly worked as a `claim` would mean two verbs doing one thing, and the
/// whole point of a separate verb is that it means something the other does not. Both ends of
/// the range are covered: an item nobody has ever held, and one that is finished.
/// The two items on the board a takeover must refuse for a reason other than a live claim:
/// one nobody has ever held, and one that is already finished.
///
/// A named provider rather than an inline literal, so a third wrong-verb case is a value
/// added here rather than a change to the loop that reads them.
const WRONG_VERB_CASES: usize = 2;

fn Items_A_Takeover_Must_Refuse() -> [(&'static str, &'static str); WRONG_VERB_CASES]
{
    return [("T-1", "an item nobody holds"), ("T-2", "a finished item")];
}

#[test]
fn Test_Taking_Over_An_Item_Nobody_Holds_Should_Be_Refused()
{
    let Board { directory: _directory, mut ledger } = Board_At("takeover-wrong-verb", vec![
        Item("T-1", &["src/a.rs"]),
        Finished("T-2", &["src/b.rs"]),
    ]);
    for (item, what) in Items_A_Takeover_Must_Refuse()
    {
        let refusal = Take_Over_In(&mut ledger, item, Claimant("agent-b"))
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
    let Board { directory, mut ledger } = Board_At("takeover-refuses-taken-ground", vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/a.rs"]),
    ]);
    Take(&mut ledger, "T-1", Claimant("dead-agent"));
    // The second claim succeeds precisely because T-1's lapsed claim no longer excludes. That
    // is the state the takeover then has to notice.
    let mut after = After_The_Lapse(directory.As_Path());
    Take(&mut after, "T-2", Claimant("agent-b"));

    let refusal = Take_Over_In(&mut after, "T-1", Claimant("agent-c"))
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
            widened: Vec::new(),
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
    let Board { directory, mut ledger } = Board_At("takeover-twice", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("dead-agent"));

    let mut takes = Ledger_When(directory.As_Path(), AT_LATER_SECONDS);
    Take_Over_In(&mut takes, "T-1", Claimant("agent-b")).expect("the first holder's lease ran out");

    // Past `agent-b`'s lease too: the takeover above installed it at `AT_LATER_SECONDS`, and a
    // lease runs a whole `LEASE` from the moment it is taken.
    let mut again = Ledger_When(directory.As_Path(), AFTER_THE_SECOND_LEASE);
    Take_Over_In(&mut again, "T-1", Claimant("agent-c"))
        .expect("the second holder's lease ran out as well");

    assert_eq!(
        Standing_Of(&Only_Item(&again)),
        Standing {
            held_by: Some("agent-c"),
            displaced: vec!["dead-agent", "agent-b"],
            widened: Vec::new(),
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
    let directory = Temporary_Directory("takeover-refuses-a-hole");
    let mut ledger = Ledger_At(directory.As_Path(), &AT_NOW);
    let path = Write_A_Claimed_Item_With_No_Claim(directory.As_Path());
    let before = std::fs::read(&path).expect("the hand-written ledger is on disk to compare against");

    let refusal = Take_Over_In(&mut ledger, "T-1", Claimant("agent-b"))
        .expect_err("an item whose predecessor record is already missing is not taken over");
    assert!(matches!(refusal, ClaimRefusal::NotClaimable { .. }), "{}", refusal.Describe());
    assert_eq!(
        std::fs::read(&path).expect("the file the refused takeover may have rewritten"),
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
             \"kind\":\"Correction\",\"origin\":\"Proposed\",\
             \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\
             \"patterns\":[]}},\
             \"state\":\"Claimed\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
             \"verification\":null,\"verified\":null,\"abandoned\":[],\"displaced\":[],\"widened\":[]}}\
             ]\n}}\n"
        ),
    )
    .expect("the hand-written ledger has to reach the disk");

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
    let Board { directory, mut ledger } = Board_At("takeover-round-trip", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("dead-agent"));

    let mut after = After_The_Lapse(directory.As_Path());
    Take_Over_In(&mut after, "T-1", Claimant("agent-b")).expect("a lapsed item is takeable");

    let first = after.Load().expect("a document carrying `displaced` must read");
    assert_eq!(first.schema_version, SCHEMA_VERSION);
    assert!(
        first
            .items
            .first()
            .is_some_and(|item| return item.displaced.len() == 1),
        "the displaced claim did not survive the write"
    );

    after.Save(&first).expect("the document with a displaced claim is one the board accepts");
    let second = after.Load().expect("the document the second save wrote reads back");

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

    let violations = Validate_Document(&Document(vec![item]), At(NOW));

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
        Validate_Document(&document, At(NOW)),
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
    let Board { directory, mut ledger } = Board_At("unusable-ledger", vec![Item("T-1", &["src/a.rs"])]);
    let missing = Refused(&mut ledger, "T-NOPE", Claimant("agent-a"));

    std::fs::write(directory.As_Path().join("ledger.json"), "{ not json")
        .expect("the corruption has to reach the disk");
    let mut broken = Ledger_At(directory.As_Path(), &AT_NOW);
    let unusable = Refused(&mut broken, "T-1", Claimant("agent-a"));
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
