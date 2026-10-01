//! Who holds the ground an item needs, read off the refusal a claim would be given.
//!
//! # Why this reads a refusal rather than the board
//!
//! The answer a queued session needs — who holds the territory, and until when — is already
//! computed. [`Claim_Refusal`] produces [`ClaimRefusal::HeldBy`] carrying the blocking item,
//! its holder and its lease expiry, and until this module existed the only caller that ever
//! saw those three fields was a session that attempted a claim it expected to fail.
//! `super::Refusal_Label` reduced the whole refusal to the word `held` at the moment it named
//! the state, and both listing verbs printed that word and nothing else.
//!
//! So nothing here decides who is blocking whom. A listing that computed its own idea of
//! overlap would be a second implementation of exclusion, and the two would eventually
//! disagree about whether an agent may proceed — which is the failure the ad-hoc scripts this
//! replaces already had. `crates/host/nomos-cli/src/work/listing.rs`'s [`Listing_Label`] doc
//! makes the same argument about the state word, and it was a second guard once.
//!
//! # How more than one blocker is found without a second computation
//!
//! [`Claim_Refusal`] names one blocker: `Held_Ground` returns the first live claim in board
//! order whose territory overlaps, so a session that clears it can meet a second holder it
//! never saw. That happened, and it is why this does not stop at the first.
//!
//! [`Blocking_Claims`] asks the *same* function a counterfactual instead of writing a
//! different one: drop the named blocker's claim from a copy of the board, ask again, and
//! keep going while the answer is still `HeldBy`. Every blocker is therefore one the claim
//! check itself named, from the only predicate that decides exclusion, and the list's head is
//! literally the refusal a claim would be given right now. The copy is the board in memory and
//! is never written; `work/ledger.json` is untouched by anything here.

use nomos_ledger::{ClaimRefusal, Claim_Refusal, ItemId, LedgerDocument};
use nomos_platform::Timestamp;

/// Seconds in a minute, an hour and a day, for [`Span`]'s three thresholds.
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;

/// One live claim standing between a caller and the item it asked about.
pub(in crate::work) struct Blocker
{
    /// The item whose claim holds the overlapping ground. Not the item that was refused —
    /// `OD-LEDGER-014` measured what happens when the two are confused in a rendering.
    pub item: ItemId,
    /// Who holds it.
    pub holder: String,
    /// When their lease lapses.
    pub until: Timestamp,
}

/// Every live claim that would refuse a claim on `item`, in the order the board answers them.
///
/// Empty for an item nothing holds, and empty for one refused on any other ground: a
/// dependency that has not finished is not somebody to wait for, and reporting it here would
/// put a holder's name where there is none.
pub(in crate::work) fn Blocking_Claims(document: &LedgerDocument, item: &ItemId, now: Timestamp) -> Vec<Blocker>
{
    let mut found = Vec::new();
    // Asked against the borrowed board first, so an item nothing holds costs one refusal and
    // no copy. Every listed row runs this, and most rows are not blocked.
    let Some(first) = Held_By(Claim_Refusal(document, item, now))
    else
    {
        return found;
    };

    let mut board = document.clone();
    let mut next = Some(first);
    while let Some(blocker) = next
    {
        let released = Release(&mut board, &blocker.item);
        found.push(blocker);
        // A refusal naming an item the board does not carry cannot be released, and asking
        // again would return it forever. Reported as the last blocker rather than looped on.
        if !released
        {
            break;
        }
        next = Held_By(Claim_Refusal(&board, item, now));
    }

    return found;
}

/// The blocker a refusal names, and nothing at all for a refusal of any other kind.
fn Held_By(refusal: Option<ClaimRefusal>) -> Option<Blocker>
{
    let ClaimRefusal::HeldBy { holder, until, item } = refusal?
    else
    {
        return None;
    };

    return Some(Blocker { item, holder, until });
}

/// Drops `blocker`'s claim from `board`, reporting whether there was such an item to drop.
///
/// The copy only. This is how the next question gets asked and is not a release: `work
/// abandon` is the verb that ends a claim, and only its holder may run it (`OD-LEDGER-001`).
fn Release(board: &mut LedgerDocument, blocker: &ItemId) -> bool
{
    let Some(found) = board.items.iter_mut().find(|candidate| return &candidate.id == blocker)
    else
    {
        return false;
    };

    found.claim = None;

    return true;
}

/// A lease expiry as something to act on: how long is left, or how long ago it ran out.
///
/// # Why the epoch stays beside it rather than being replaced
///
/// Every lease this CLI prints is a whole-second unix epoch, which is the ledger's own
/// storage form and is what `git diff` over `work/ledger.json` shows. A reader deciding
/// whether to wait converts it by hand every time; a reader reconciling a printed line
/// against the committed board needs the stored number. So the callers print both, and this
/// renders the half that was missing.
///
/// # Why `Refusal::Describe` is deliberately left saying `until unix N`
///
/// It cannot say anything else. `Describe` takes `&self` and no instant, so the library has
/// no clock to measure against, and giving it one would move a composition-root decision into
/// `nomos-ledger` — the division `OD-HOST-002` draws, and the reason `super::Ended` carries a
/// `now` the report is handed rather than reading its own. A relative window is a rendering
/// for a caller that knows what time it is, which is this module.
pub(in crate::work) fn Lease_Window(until: Timestamp, now: Timestamp) -> String
{
    let remaining = until.Since(now).as_secs();
    if remaining > 0
    {
        return format!("{} from now", Span(remaining));
    }

    let lapsed = now.Since(until).as_secs();
    if lapsed == 0
    {
        return "now".to_owned();
    }

    return format!("{} ago", Span(lapsed));
}

/// A duration in the coarsest unit that still says something: days and hours, hours and
/// minutes, minutes, or seconds.
///
/// Days exist for the lapsed direction only. A live lease is bounded by `MAXIMUM_LEASE`, so a
/// window measured forward never reaches one; a claim whose holder stopped paying attention is
/// bounded by nothing, and a lease read as `250h 34m ago` asks the reader for the arithmetic
/// this function exists to do. Measured on a real board: `P111` had lapsed ten days earlier.
fn Span(seconds: u64) -> String
{
    if seconds >= SECONDS_PER_DAY
    {
        let hours = (seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR;

        return format!("{}d {hours}h", seconds / SECONDS_PER_DAY);
    }
    if seconds >= SECONDS_PER_HOUR
    {
        let minutes = (seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;

        return format!("{}h {minutes}m", seconds / SECONDS_PER_HOUR);
    }
    if seconds >= SECONDS_PER_MINUTE
    {
        return format!("{}m", seconds / SECONDS_PER_MINUTE);
    }

    return format!("{seconds}s");
}

/// The blocked fact for one listing row: who to wait for, for how long, and how many more are
/// behind them.
///
/// Empty when nothing holds the item, which is what keeps an unblocked row reading exactly as
/// it did before this existed.
///
/// The count is what stops this from lying by omission. One name alone reads as the only
/// blocker, and a session that cleared the one name a refusal gave it reformed its plan around
/// that and then met a second holder; `+N more` is the difference between a first answer and a
/// complete one, and [`Blocked_Lines`] is where the rest are named.
pub(in crate::work) fn Blocked_Note(blockers: &[Blocker], now: Timestamp) -> String
{
    let Some(first) = blockers.first()
    else
    {
        return String::new();
    };

    let others = blockers.len().saturating_sub(1);
    let more = if others == 0
    {
        String::new()
    }
    else
    {
        format!(" +{others} more")
    };

    return format!("  [{}{more}, {}]", first.holder, Lease_Window(first.until, now));
}

/// The blocked fact in full, one line per blocker: the item holding the ground, who holds it,
/// and when their lease lapses.
///
/// `show` prints an item whole and unabridged, so it names every blocker rather than the first
/// and a count. Empty for an item nothing holds, which is why an unblocked `show` says nothing
/// new.
pub(in crate::work) fn Blocked_Lines(blockers: &[Blocker], now: Timestamp) -> Vec<String>
{
    return blockers
        .iter()
        .map(|blocker| {
            return format!(
                "blocked by {}, held by {} until unix {} ({})",
                blocker.item,
                blocker.holder,
                blocker.until.Unix_Seconds(),
                Lease_Window(blocker.until, now)
            );
        })
        .collect();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_ledger::{Claim, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};

    /// The instant every case below asks its question at.
    const ASKED_AT: i64 = 1_000;
    /// A lease still running at [`ASKED_AT`], and one that ran out before it.
    const LIVE_LEASE_ENDS_AT: i64 = 5_000;
    const LAPSED_LEASE_ENDED_AT: i64 = 500;
    /// The ground two items both reserve, and ground nobody contests.
    const CONTESTED: &str = "src/contested.rs";
    const UNCONTESTED: &str = "src/elsewhere.rs";

    /// The moment the cases read, spelled once.
    fn Now() -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(ASKED_AT);
    }

    /// An unclaimed item reserving one file.
    fn Item(id: &str, path: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: format!("item {id}"),
            why: "because".to_owned(),
            done_when: "it prints".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files(vec![path.to_owned()]),
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

    /// The same item, held by `holder` under a lease that has not run out.
    fn Held(id: &str, path: &str, holder: &str) -> LedgerItem
    {
        let mut item = Item(id, path);
        item.state = ItemState::Claimed;
        item.claim = Some(Claim {
            holder: holder.to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(LIVE_LEASE_ENDS_AT),
        });

        return item;
    }

    fn Board(items: Vec<LedgerItem>) -> LedgerDocument
    {
        return LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items,
        };
    }

    /// **The guard this module exists to satisfy.** The holder reported to a reader is the
    /// holder [`Claim_Refusal`] names, compared rather than asserted against a literal.
    ///
    /// Both sides are derived here on purpose. A case asserting `"agent-b"` would pass over a
    /// rendering that had picked some other live claim off the board, which is exactly what an
    /// ad-hoc script over `work/ledger.json` does when it is wrong: it produces a blocker that
    /// is plausible. This fails the moment the two disagree, whichever of them moved.
    #[test]
    fn Test_Blocking_Claims_Should_Name_The_Holder_The_Claim_Check_Names()
    {
        let board = Board(vec![Item("T-1", CONTESTED), Held("T-2", CONTESTED, "agent-b")]);
        let asked = ItemId::New("T-1");

        let blockers = Blocking_Claims(&board, &asked, Now());

        let Some(ClaimRefusal::HeldBy { holder, until, item }) = Claim_Refusal(&board, &asked, Now())
        else
        {
            panic!("the fixture holds T-1's ground, so the claim check must refuse it as held");
        };
        let first = blockers.first().expect("a held item has at least one blocker");
        assert_eq!(first.holder, holder, "the reported holder is not the one the claim check names");
        assert_eq!(first.item, item, "the reported blocking item is not the one the claim check names");
        assert_eq!(first.until, until, "the reported lease is not the one the claim check names");
    }

    /// Every blocker, not the first one and a silence about the rest.
    ///
    /// Measured on the live board while this was written: `P127` was held by two claims at
    /// once. A session that clears the one name a refusal gives it and finds a second holder
    /// has been told something true and incomplete, and the incompleteness is what cost a
    /// session its plan.
    #[test]
    fn Test_Blocking_Claims_Should_Name_Every_Blocker_Rather_Than_Only_The_First()
    {
        let board = Board(vec![
            Item("T-1", CONTESTED),
            Held("T-2", CONTESTED, "agent-b"),
            Held("T-3", CONTESTED, "agent-c"),
        ]);

        let blockers = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        let holders: Vec<&str> = blockers.iter().map(|blocker| return blocker.holder.as_str()).collect();
        assert_eq!(holders, vec!["agent-b", "agent-c"], "both live claims contest this ground");
    }

    /// An item nothing holds gets no blocker, which is what keeps an unblocked row unchanged.
    #[test]
    fn Test_Blocking_Claims_Should_Be_Empty_For_An_Item_Nothing_Contests()
    {
        let board = Board(vec![Item("T-1", UNCONTESTED), Held("T-2", CONTESTED, "agent-b")]);

        let blockers = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        assert!(blockers.is_empty(), "disjoint territory is not a blocker");
    }

    /// A lapsed claim stops excluding, so its holder is not somebody to wait for.
    ///
    /// The refusal for this item is `Lapsed`, not `HeldBy`, and putting a name here would tell
    /// a reader to wait for a holder who is gone when the remedy is `nomos work takeover`.
    #[test]
    fn Test_Blocking_Claims_Should_Be_Empty_When_The_Contesting_Lease_Has_Run_Out()
    {
        let mut stale = Held("T-2", CONTESTED, "agent-b");
        stale.claim = Some(Claim {
            holder: "agent-b".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(LAPSED_LEASE_ENDED_AT),
        });
        let board = Board(vec![Item("T-1", CONTESTED), stale]);

        let blockers = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        assert!(blockers.is_empty(), "a lapsed claim excludes nobody, so it blocks nobody");
    }

    /// Asking the counterfactual must not write the board it was asked about.
    #[test]
    fn Test_Blocking_Claims_Should_Leave_The_Board_It_Was_Given_Untouched()
    {
        let board = Board(vec![Item("T-1", CONTESTED), Held("T-2", CONTESTED, "agent-b")]);
        let before = board.clone();

        let _ = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        assert_eq!(board, before, "the released claim was dropped from a copy, never from this");
    }

    #[test]
    fn Test_Lease_Window_Should_Say_How_Long_Is_Left_And_How_Long_Ago_It_Ran_Out()
    {
        let ahead = Timestamp::From_Unix_Seconds(ASKED_AT + 7_260);
        let behind = Timestamp::From_Unix_Seconds(ASKED_AT - 120);

        assert_eq!(Lease_Window(ahead, Now()), "2h 1m from now");
        assert_eq!(Lease_Window(behind, Now()), "2m ago");
        assert_eq!(Lease_Window(Now(), Now()), "now");
    }

    /// The three units, at each boundary rather than at one comfortable value inside it.
    #[test]
    fn Test_Span_Should_Report_Days_Then_Hours_Then_Minutes_Then_Seconds()
    {
        assert_eq!(Span(SECONDS_PER_DAY), "1d 0h");
        assert_eq!(Span(SECONDS_PER_DAY - 1), "23h 59m");
        assert_eq!(Span(SECONDS_PER_HOUR), "1h 0m");
        assert_eq!(Span(SECONDS_PER_HOUR - 1), "59m");
        assert_eq!(Span(SECONDS_PER_MINUTE), "1m");
        assert_eq!(Span(SECONDS_PER_MINUTE - 1), "59s");
        assert_eq!(Span(0), "0s");
    }

    /// The note names one holder and counts the rest, and says nothing when nothing holds.
    #[test]
    fn Test_Blocked_Note_Should_Count_The_Blockers_It_Does_Not_Name()
    {
        let board = Board(vec![
            Item("T-1", CONTESTED),
            Held("T-2", CONTESTED, "agent-b"),
            Held("T-3", CONTESTED, "agent-c"),
        ]);
        let blockers = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        let note = Blocked_Note(&blockers, Now());

        assert!(note.contains("agent-b"), "{note}");
        assert!(note.contains("+1 more"), "one name alone reads as the only blocker: {note}");
        assert_eq!(Blocked_Note(&[], Now()), "", "an unblocked row gains nothing");
    }

    #[test]
    fn Test_Blocked_Lines_Should_Name_The_Item_The_Holder_And_The_Lease_Per_Blocker()
    {
        let board = Board(vec![Item("T-1", CONTESTED), Held("T-2", CONTESTED, "agent-b")]);
        let blockers = Blocking_Claims(&board, &ItemId::New("T-1"), Now());

        let lines = Blocked_Lines(&blockers, Now());

        assert_eq!(lines.len(), 1, "one blocker, one line: {lines:?}");
        let said = lines.first().expect("asserted one line above");
        assert!(said.contains("T-2"), "{said}");
        assert!(said.contains("agent-b"), "{said}");
        assert!(said.contains(&format!("unix {LIVE_LEASE_ENDS_AT}")), "the stored epoch: {said}");
        assert!(said.contains("from now"), "and the window a reader acts on: {said}");
        assert!(Blocked_Lines(&[], Now()).is_empty(), "an unblocked show says nothing new");
    }

    /// [`Held_By`] answers for the held arm only, so no other refusal can produce a holder.
    #[test]
    fn Test_Held_By_Should_Answer_For_The_Held_Arm_And_No_Other()
    {
        let held = ClaimRefusal::HeldBy {
            holder: "agent-b".to_owned(),
            until: Timestamp::From_Unix_Seconds(LIVE_LEASE_ENDS_AT),
            item: ItemId::New("T-2"),
        };
        let unmet = ClaimRefusal::DependencyUnmet {
            item: ItemId::New("T-1"),
            dependency: ItemId::New("T-9"),
            state: "Ready".to_owned(),
        };

        assert!(Held_By(Some(held)).is_some());
        assert!(Held_By(Some(unmet)).is_none(), "a dependency is not a holder to wait for");
        assert!(Held_By(None).is_none());
    }

    /// [`Release`] reports a blocker the board does not carry, which is what bounds the loop.
    #[test]
    fn Test_Release_Should_Report_A_Blocker_The_Board_Does_Not_Carry()
    {
        let mut board = Board(vec![Held("T-2", CONTESTED, "agent-b")]);

        assert!(Release(&mut board, &ItemId::New("T-2")));
        assert!(board.items.first().expect("the fixture has an item").claim.is_none());
        assert!(!Release(&mut board, &ItemId::New("T-404")), "no such item is not a release");
    }
}
