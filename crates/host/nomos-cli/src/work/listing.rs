//! Printing an item: how it stands on the board, and everything recorded against it.

use nomos_ledger::{
    Claim_Refusal, ClaimRefusal, ItemState, LedgerDocument, LedgerItem, Territory,
    VerificationPredicate, VerificationRecord,
};
use nomos_platform::Timestamp;

use super::ListingScope;

/// What a listing prints of the board it was handed.
///
/// One value rather than two parameters, because both answer the same question -- which rows
/// -- from opposite ends, and a caller holding them apart would have to remember which one
/// wins. Carried together, the precedence is stated once, in [`Listed_As`], and every caller
/// gets it.
#[derive(Clone, Copy)]
pub(super) struct Bounds<'a>
{
    /// Only items whose label is this word, when a caller named one.
    pub state: Option<&'a str>,
    /// How much of the board to draw rows from when no word was named.
    pub scope: ListingScope,
}

/// The label this item lists under, or nothing when `bounds` leaves it out.
///
/// The label is computed first, out of the whole `document`, whether or not the row survives.
/// That ordering is the invariant this function carries: a bound decides which rows are
/// printed and never what a printed row is called, so no item's word can change because some
/// other item was withheld. `OD-LEDGER-023`.
///
/// A named `state` wins outright over `scope`. It is itself a bound, and the one the caller
/// actually asked for -- `--state done` wants terminal rows, and narrowing it further by scope
/// would answer that question with none of them.
pub(super) fn Listed_As(
    document: &LedgerDocument,
    item: &LedgerItem,
    bounds: Bounds<'_>,
    now: Timestamp,
) -> Option<&'static str>
{
    let label = Listing_Label(document, item, now);

    return match bounds.state
    {
        Some(wanted) => label.eq_ignore_ascii_case(wanted).then_some(label),
        None => Admitted_By(item, bounds.scope).then_some(label),
    };
}

/// Whether `scope` admits `item` onto an unfiltered listing.
///
/// Terminality is read off the item's own state rather than off its label, and the two agree:
/// `Claim_Refusal` answers `NotClaimable` for every `Done` and `Declined` item, so
/// [`Listing_Label`] falls through to the state word for exactly these. The state is what this
/// asks because it is what the bound means -- an item that has ended -- and
/// `nomos_ledger::ItemState::Is_Finished` is where that is already decided, for `decline` and
/// for the claim check both.
fn Admitted_By(item: &LedgerItem, scope: ListingScope) -> bool
{
    return match scope
    {
        ListingScope::Whole => true,
        ListingScope::Live => !item.state.Is_Finished(),
    };
}

/// What to say when the listing printed nothing.
///
/// Three cases rather than two, and the third is the one the bound added. An empty board, a
/// filter that matched nothing, and a board whose every item has ended look identical
/// otherwise, and the reader's next action differs in each: nothing to do, ask for another
/// bucket, or ask for the whole board. The third names the flag, because a caller who has just
/// been shown nothing is exactly the one who cannot tell a bounded answer from an empty one.
pub(super) fn Nothing_Listed(bounds: Bounds<'_>, output: &mut impl std::io::Write)
{
    let _ = match (bounds.state, bounds.scope)
    {
        (Some(wanted), _) => writeln!(output, "no items are {wanted}"),
        (None, ListingScope::Live) => writeln!(
            output,
            "nothing on the board is live; `nomos work list --all` prints every item, \
             including the ones that have ended"
        ),
        (None, ListingScope::Whole) => writeln!(output, "the ledger has no items"),
    };
}

/// One item's line: its identifier, what it may be called now, its kind and origin, its
/// holder, and its title.
///
/// `kind` and `origin` sit right after the claimability label and before the holder and
/// title, the same two fixed-width columns `work show`'s own `kind: {:?}  origin: {:?}`
/// prints for the field it reads them from -- `OD-LEDGER-024`'s follow-on, closed here.
pub(super) fn Print_Listing(item: &LedgerItem, label: &str, output: &mut impl std::io::Write)
{
    let holder = item
        .claim
        .as_ref()
        .map_or_else(String::new, |claim| return format!("  [{}]", claim.holder));

    let _ = writeln!(
        output,
        "{:<13} {:<9}{:<11?} {:<9?}{holder}  {}",
        item.id, label, item.kind, item.origin, item.title
    );
}

/// The live claim, if there is one.
///
/// A lapsed claim is still shown, and still says it lapsed. It stops excluding without being
/// removed, so a reader who is not told would take it for a live one.
pub(super) fn Print_Claim(found: &LedgerItem, now: Timestamp, output: &mut impl std::io::Write)
{
    let Some(claim) = &found.claim
    else
    {
        return;
    };

    let lapsed = if claim.Has_Lapsed(now)
    {
        " (lapsed)"
    }
    else
    {
        ""
    };

    let _ = writeln!(
        output,
        "held by {} since unix {} until unix {}{lapsed}",
        claim.holder,
        claim.acquired_at.Unix_Seconds(),
        claim.lease_expires_at.Unix_Seconds()
    );
}

/// What has happened to the item: takeovers, abandonments, and the verification that ended
/// it.
///
/// Reported, not merely stored. `OD-LEDGER-006`'s rule and `OD-LEDGER-012`'s reason for
/// obeying it here: a record no surface reports is one only somebody willing to read the
/// JSON can find, which is most of the way back to not keeping it. Each displacement line
/// names the holder a takeover displaced and the window they held — who displaced them is
/// the next line's holder, or the live claim above.
///
/// `current_revision` is this tree's revision *right now*, read by the caller the same way
/// `nomos_ledger::Finish_Item` read it when it stamped the record — `OD-LEDGER-027`'s staleness
/// half. `None` when it could not be read, which `Print_Verification` reports as its own
/// case rather than silently treating as agreement.
pub(super) fn Print_History(found: &LedgerItem, current_revision: Option<&str>, output: &mut impl std::io::Write)
{
    Print_Displacements(found, output);
    Print_Abandonments(found, output);
    Print_Widenings(found, output);
    Print_Verification(found, current_revision, output);
}

/// Every holder a takeover displaced, and the window they held.
fn Print_Displacements(found: &LedgerItem, output: &mut impl std::io::Write)
{
    for displaced in &found.displaced
    {
        let _ = writeln!(
            output,
            "taken over from {}, who held it from unix {} until unix {}",
            displaced.holder,
            displaced.acquired_at.Unix_Seconds(),
            displaced.lease_expires_at.Unix_Seconds()
        );
    }
}

/// Every claim that was given up without finishing, and the reason given.
fn Print_Abandonments(found: &LedgerItem, output: &mut impl std::io::Write)
{
    for abandonment in &found.abandoned
    {
        let _ = writeln!(
            output,
            "abandoned by {} at unix {}: {}",
            abandonment.holder,
            abandonment.abandoned_at.Unix_Seconds(),
            abandonment.reason
        );
    }
}

/// Every enlargement of the territory, and what each one added.
///
/// The added paths and not the territory as it stands, which the item already carries. What is
/// not otherwise recoverable is which of those paths were not predicted when the item was
/// authored, and that is the whole of what `OD-LEDGER-039` keeps these rows for.
fn Print_Widenings(found: &LedgerItem, output: &mut impl std::io::Write)
{
    for widening in &found.widened
    {
        let _ = writeln!(
            output,
            "widened by {} at unix {}, adding {}: {}",
            widening.holder,
            widening.widened_at.Unix_Seconds(),
            widening.added.len(),
            widening.added.join(" ")
        );
    }
}

/// The predicate that ended the item, if one has.
fn Print_Verification(found: &LedgerItem, current_revision: Option<&str>, output: &mut impl std::io::Write)
{
    let Some(record) = &found.verified
    else
    {
        return;
    };

    let _ = writeln!(
        output,
        "verified by `{}` at unix {} with exit {}",
        record.argv.join(" "),
        record.verified_at.Unix_Seconds(),
        record.exit_code
    );

    Print_Staleness(record, current_revision, output);
}

/// Whether the tree this record was verified against is the tree being read right now.
///
/// `OD-LEDGER-027`'s rule that staleness is reported, not merely storable: `work show` says
/// something different for each of the four cases below rather than only exposing the
/// field for somebody willing to read the JSON to compare by hand.
fn Print_Staleness(
    record: &VerificationRecord,
    current_revision: Option<&str>,
    output: &mut impl std::io::Write,
)
{
    let _ = match (&record.revision, current_revision)
    {
        // Written before `OD-LEDGER-027`, or `HEAD` could not be resolved when it ran.
        // Never backfilled -- see `VerificationRecord::revision`.
        (None, _) => writeln!(output, "no revision recorded for this verification"),
        (Some(recorded), Some(current)) if recorded == current =>
        {
            writeln!(output, "still describes this tree, at revision {recorded}")
        }
        (Some(recorded), Some(current)) => writeln!(
            output,
            "STALE: verified against revision {recorded}, this tree is now at {current}"
        ),
        (Some(recorded), None) => writeln!(
            output,
            "cannot tell: verified against revision {recorded}, this tree's current revision \
             could not be read"
        ),
    };
}

/// The item's own terms: why it is worth doing, what finishing means, the ground it reserves,
/// and the predicate that will judge it.
///
/// `OD-LEDGER-006`'s rule — a record no surface reports is one only somebody willing to read
/// the JSON can find — applied to the four fields it had never reached. `AGENTS.md`'s loop and
/// the `nomos-task` skill both send a session to read an item's full `why`, `done_when`,
/// territory and predicate before it edits anything, and until this existed the verb they
/// route to printed none of them. The failure was quiet rather than loud: a session that did
/// not notice proceeded from whatever paraphrase its instructions carried, which is the thing
/// an unamendable `done_when` exists to prevent.
///
/// Printed after the claim and the history rather than before them. Those are short lines that
/// readers, and at least one script, find by position near the top; a `done_when` runs to
/// several hundred words and would push every one of them off the screen.
pub(super) fn Print_Contract(found: &LedgerItem, output: &mut impl std::io::Write)
{
    Print_Prose("why", &found.why, output);
    Print_Prose("done_when", &found.done_when, output);
    Print_Territory(&found.territory, output);
    Print_Predicate(found.verification.as_ref(), output);
}

/// One prose field, under a label of its own, exactly as the board holds it.
///
/// Nothing here truncates, elides, re-wraps or indents the text. A reader comparing what was
/// printed against what is stored sees the same characters, and a clipped contract is worse
/// than no contract at all, because nothing in a clipped one says which clause is missing.
///
/// The label is a line by itself for the same reason the text is not indented: `why:` followed
/// by four hundred words is a label that has scrolled away by the time the reader wants it,
/// and a label on its own line is one a reader can scan a screen for.
///
/// An empty field says so on the label line. Printing a label over a blank line reads as
/// output that broke, rather than as a field nobody filled in.
fn Print_Prose(label: &str, text: &str, output: &mut impl std::io::Write)
{
    let _ = writeln!(output);

    if text.trim().is_empty()
    {
        let _ = writeln!(output, "{label}: (empty)");

        return;
    }

    let _ = writeln!(output, "{label}:");
    let _ = writeln!(output, "{text}");
}

/// The ground the item reserves, counted and then listed.
///
/// One path per line, because a territory is a list rather than prose and the reader is
/// scanning for whether one file is in it. The count comes first and in the spelling `add`
/// already answers with, so the two surfaces report a reservation the same way.
///
/// Patterns are printed when there are any. Nothing has authored one since `OD-LEDGER-013`
/// withdrew the flag, so the usual answer is none — but a hand-edited document can still
/// carry one, every comparison involving it answers `Unknown`, and a reader wondering why
/// their claim was refused needs to see it rather than have it silently left out.
fn Print_Territory(territory: &Territory, output: &mut impl std::io::Write)
{
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "territory: {} path(s) at {:?} resolution",
        territory.paths.len(),
        territory.resolution
    );

    for path in &territory.paths
    {
        let _ = writeln!(output, "  {path}");
    }
    for pattern in &territory.patterns
    {
        let _ = writeln!(output, "  pattern {pattern}");
    }
}

/// The predicate declared to judge the item, and the bound it will run under.
///
/// What the item *carries*, which is not what [`Print_Verification`] reports. That one prints
/// the record of a run, and an item that has never been verified has a declared predicate and
/// no record of it — so the two lines answer different questions and are spelled differently
/// (`verification:` against `verified by`) so that a reader can tell which they are reading.
fn Print_Predicate(predicate: Option<&VerificationPredicate>, output: &mut impl std::io::Write)
{
    let Some(declared) = predicate
    else
    {
        let _ = writeln!(output, "verification: (none declared)");

        return;
    };

    let _ = writeln!(
        output,
        "verification: `{}` within {} seconds",
        declared.argv.join(" "),
        declared.timeout_seconds
    );
}

/// What to call an item in a listing.
///
/// For everything except a `Ready` item this is just the state. `Ready` is the word that
/// was lying: it means "nobody has taken this", and a reader takes it to mean "I can take
/// this". Those came apart whenever a dependency was unfinished or somebody held
/// overlapping ground — on 2026-08-09 the column said `ready` for eight items that a single
/// held claim refused, three separate times.
///
/// `claimed` is the second word that lied, for the same reason `ready` was the first. An item
/// whose lease ran out four hours ago reads as work in progress, and it is work whose holder is
/// gone. It excludes nobody — `OD-LEDGER-009` — and since `OD-LEDGER-012` it is takeable, by
/// `nomos work takeover` rather than by `claim`. So `lapsed` now says an operation is available
/// rather than that an editor is.
///
/// The answer comes from [`Claim_Refusal`], the function `claim` itself refuses with. That is
/// the point rather than an implementation detail: a listing computing its own idea of
/// claimability would be a second guard for one rule, and the two would eventually disagree
/// about whether an agent may proceed.
///
/// This function *was* that second guard. It decided `lapsed` itself, in a branch above the
/// refusal it now reads, and after `OD-LEDGER-012` that branch was a second implementation of
/// the predicate deciding whether `takeover` succeeds — the exact arrangement the paragraph
/// above forbids, in the function whose doc comment forbids it. Every label is unchanged for
/// every input; what changed is that one function decides.
pub(super) fn Listing_Label(document: &LedgerDocument, item: &LedgerItem, now: Timestamp) -> &'static str
{
    use super::report::Refusal_Label;

    return match Claim_Refusal(document, &item.id, now)
    {
        // Nothing refuses it, or nothing is meant to: the state word is the honest answer in
        // both cases, and for a `Ready` item that word is `ready`. `NotClaimable` is the arm
        // every `Blocked`, `Done` and `Declined` item arrives on, and its own word is better
        // than any refusal's — nobody is queued behind finished work.
        None | Some(ClaimRefusal::NotClaimable { .. }) => State_Label(&item.state),
        Some(refusal) => Refusal_Label(&refusal),
    };
}

fn State_Label(state: &ItemState) -> &'static str
{
    return match state
    {
        ItemState::Ready => "ready",
        ItemState::Claimed => "claimed",
        ItemState::Blocked => "blocked",
        ItemState::Done => "done",
        ItemState::Declined { .. } => "declined",
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use super::super::ListingScope;
    use nomos_ledger::{ItemId, ItemKind, ItemOrigin, Territory};

    /// The moment a listing is read, one second after the item it names was declined. Named
    /// so the test reads as "after the decline" rather than as an unexplained second.
    const AFTER_THE_DECLINE: i64 = 2;

    /// A lease far enough ahead that a claim is still held when the listing is printed.
    const HELD_LEASE_EXPIRES_AT: i64 = 1_000;

    /// A lease that has run out by the moment the claim is printed, and that later moment.
    const LAPSED_LEASE_EXPIRES_AT: i64 = 10;
    const AFTER_THE_LEASE_LAPSES: i64 = 20;

    /// The three moments the full-history fixture is stamped with, in the order its rows were
    /// appended: a claim whose lease ran out, then the abandonment, then the verification.
    const HISTORY_LEASE_EXPIRED_AT: i64 = 10;
    const HISTORY_ABANDONED_AT: i64 = 20;
    const HISTORY_VERIFIED_AT: i64 = 30;

    /// The column an agent reads before claiming has to say the item is over.
    ///
    /// This is the whole of what the state buys at the surface. `P10-REQUIRABLE-DECLARED` was
    /// superseded twice and read `ready` both times, with the reason behind `work show` where
    /// nobody looks first — so the second session claimed it and spent its run establishing
    /// that the first one was right.
    #[test]
    fn Test_Listing_Label_Should_Report_Declined_For_A_Declined_Item()
    {
        let mut item = Item_For_Id("T-1");
        item.Decline("superseded by T-2", "agent-a", Timestamp::From_Unix_Seconds(1));
        let document = Board_With(item);
        let found = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listing_Label(&document, found, Timestamp::From_Unix_Seconds(AFTER_THE_DECLINE)),
            "declined"
        );
    }

    /// The bounds an unfiltered call carries under each scope, named so a case below reads as
    /// the question it asks rather than as a struct literal.
    const LIVE: Bounds<'static> = Bounds { state: None, scope: ListingScope::Live };
    const WHOLE: Bounds<'static> = Bounds { state: None, scope: ListingScope::Whole };

    #[test]
    fn Test_Listed_As_Should_Report_The_Items_Label_When_No_Filter_Is_Given()
    {
        let document = Board_With(Item_For_Id("T-1"));
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(&document, item, WHOLE, Timestamp::From_Unix_Seconds(0)),
            Some("ready")
        );
    }

    #[test]
    fn Test_Listed_As_Should_Exclude_An_Item_Whose_Label_Does_Not_Match_The_Filter()
    {
        let document = Board_With(Item_For_Id("T-1"));
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(
                &document,
                item,
                Bounds { state: Some("claimed"), scope: ListingScope::Whole },
                Timestamp::From_Unix_Seconds(0)
            ),
            None,
            "a ready item filtered by `--state claimed` must not be listed"
        );
    }

    /// The default's bound, at the one function that applies it.
    #[test]
    fn Test_Listed_As_Should_Exclude_An_Item_That_Has_Ended_From_The_Live_Scope()
    {
        let mut ended = Item_For_Id("T-1");
        ended.state = ItemState::Done;
        let document = Board_With(ended);
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(&document, item, LIVE, Timestamp::From_Unix_Seconds(0)),
            None,
            "a Done item is not on the live board"
        );
        assert_eq!(
            Listed_As(&document, item, WHOLE, Timestamp::From_Unix_Seconds(0)),
            Some("done"),
            "and the whole board is the scope that reaches it, or the bound is a hole"
        );
    }

    /// A named state answers with its own rows, whatever the scope beside it says.
    ///
    /// Without this, the default bound would quietly make `--state done` and `--state declined`
    /// answer nothing -- two of the ten words the usage text promises a row can carry.
    #[test]
    fn Test_Listed_As_Should_Answer_A_Terminal_Filter_Under_The_Live_Scope()
    {
        let mut ended = Item_For_Id("T-1");
        ended.state = ItemState::Done;
        let document = Board_With(ended);
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(
                &document,
                item,
                Bounds { state: Some("done"), scope: ListingScope::Live },
                Timestamp::From_Unix_Seconds(0)
            ),
            Some("done"),
            "`--state done` asks for terminal rows and must not be narrowed by the default scope"
        );
    }

    #[test]
    fn Test_Nothing_Listed_Should_Name_The_Filter_That_Matched_Nothing()
    {
        let mut output = Vec::new();

        Nothing_Listed(Bounds { state: Some("blocked"), scope: ListingScope::Live }, &mut output);

        assert_eq!(String::from_utf8(output).unwrap(), "no items are blocked\n");
    }

    #[test]
    fn Test_Nothing_Listed_Should_Say_The_Ledger_Is_Empty_When_The_Whole_Board_Was_Asked_For()
    {
        let mut output = Vec::new();

        Nothing_Listed(WHOLE, &mut output);

        assert_eq!(String::from_utf8(output).unwrap(), "the ledger has no items\n");
    }

    /// An empty live board is not an empty ledger, and saying so would send a reader away from
    /// work that is on the board behind a flag nothing had told them about.
    #[test]
    fn Test_Nothing_Listed_Should_Name_The_Flag_When_The_Live_Board_Is_Empty()
    {
        let mut output = Vec::new();

        Nothing_Listed(LIVE, &mut output);

        let said = String::from_utf8(output).expect("Nothing_Listed writes only str into the buffer");
        assert!(said.contains("live"), "{said}");
        assert!(said.contains("--all"), "{said}");
        assert!(!said.contains("the ledger has no items"), "{said}");
    }

    #[test]
    fn Test_Print_Listing_Should_Print_The_Items_Identifier_Label_And_Title()
    {
        let item = Item_For_Id("T-1");
        let mut output = Vec::new();

        Print_Listing(&item, "ready", &mut output);

        let printed = String::from_utf8(output).expect("Print_Listing writes only str into the buffer");
        assert!(printed.contains("T-1"), "{printed}");
        assert!(printed.contains("ready"), "{printed}");
        assert!(printed.contains("item T-1"), "{printed}");
    }

    #[test]
    fn Test_Print_Listing_Should_Show_The_Holder_Of_A_Claimed_Item()
    {
        let mut item = Item_For_Id("T-1");
        item.claim = Some(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(1),
            lease_expires_at: Timestamp::From_Unix_Seconds(HELD_LEASE_EXPIRES_AT),
        });
        let mut output = Vec::new();

        Print_Listing(&item, "claimed", &mut output);

        assert!(
            String::from_utf8(output).unwrap().contains("[agent-a]"),
            "a claimed item's line must name its holder"
        );
    }

    #[test]
    fn Test_Print_Claim_Should_Print_Nothing_For_An_Item_With_No_Claim()
    {
        let item = Item_For_Id("T-1");
        let mut output = Vec::new();

        Print_Claim(&item, Timestamp::From_Unix_Seconds(0), &mut output);

        assert!(output.is_empty(), "an unclaimed item has no claim line to print");
    }

    #[test]
    fn Test_Print_Claim_Should_Mark_A_Lapsed_Claim()
    {
        let mut item = Item_For_Id("T-1");
        item.claim = Some(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(LAPSED_LEASE_EXPIRES_AT),
        });
        let mut output = Vec::new();

        Print_Claim(&item, Timestamp::From_Unix_Seconds(AFTER_THE_LEASE_LAPSES), &mut output);

        let printed = String::from_utf8(output).expect("Print_Claim writes only str into the buffer");
        assert!(printed.contains("agent-a"), "{printed}");
        assert!(printed.contains("(lapsed)"), "{printed}");
    }

    #[test]
    fn Test_Print_History_Should_Print_Every_Displacement_Abandonment_And_Verification()
    {
        let item = An_Item_With_A_Full_History();
        let mut output = Vec::new();

        Print_History(&item, Some("abc123"), &mut output);

        let printed = String::from_utf8(output).expect("Print_History writes only str into the buffer");
        assert!(printed.contains("taken over from agent-a"), "{printed}");
        assert!(printed.contains("abandoned by agent-b"), "{printed}");
        assert!(printed.contains("verified by `cargo test`"), "{printed}");
        assert!(printed.contains("still describes this tree"), "{printed}");
    }

    /// One item carrying each of the three history rows [`Print_History`] prints, and a
    /// verification stamped against the revision the test reads back.
    fn An_Item_With_A_Full_History() -> LedgerItem
    {
        let mut item = Item_For_Id("T-1");
        item.displaced.push(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(HISTORY_LEASE_EXPIRED_AT),
        });
        item.abandoned.push(nomos_ledger::Abandonment {
            holder: "agent-b".to_owned(),
            reason: "superseded".to_owned(),
            abandoned_at: Timestamp::From_Unix_Seconds(HISTORY_ABANDONED_AT),
        });
        item.verified = Some(VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: String::new(),
            verified_at: Timestamp::From_Unix_Seconds(HISTORY_VERIFIED_AT),
            gate: None,
            revision: Some("abc123".to_owned()),
        });

        return item;
    }

    /// A minimal, ready item: enough to exercise the listing surface without a claim, a
    /// history or a territory that matters to any test here.
    fn Item_For_Id(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: format!("item {id}"),
            why: "because".to_owned(),
            done_when: "it prints".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files(vec!["src/a.rs".to_owned()]),
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

    fn Board_With(item: LedgerItem) -> LedgerDocument
    {
        return LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items: vec![item],
        };
    }
}
