//! Printing an item: how it stands on the board, and everything recorded against it.

use nomos_ledger::{Claim_Refusal, ClaimRefusal, ItemState, LedgerDocument, LedgerItem, VerificationRecord};
use nomos_platform::Timestamp;

/// The label this item lists under, or nothing when the filter excludes it.
pub(super) fn Listed_As(
    document: &LedgerDocument,
    item: &LedgerItem,
    state: Option<&str>,
    now: Timestamp,
) -> Option<&'static str>
{
    let label = Listing_Label(document, item, now);
    if state.is_some_and(|wanted| return !label.eq_ignore_ascii_case(wanted))
    {
        return None;
    }

    return Some(label);
}

/// What to say when the listing printed nothing.
///
/// An empty result and a filter that matched nothing look identical otherwise, and the
/// user's next action differs.
pub(super) fn Nothing_Listed(state: Option<&str>, output: &mut impl std::io::Write)
{
    let _ = match state
    {
        Some(wanted) => writeln!(output, "no items are {wanted}"),
        None => writeln!(output, "the ledger has no items"),
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
    use nomos_ledger::{ItemId, ItemKind, ItemOrigin, Territory};

    /// The column an agent reads before claiming has to say the item is over.
    ///
    /// This is the whole of what the state buys at the surface. `P10-REQUIRABLE-DECLARED` was
    /// superseded twice and read `ready` both times, with the reason behind `work show` where
    /// nobody looks first — so the second session claimed it and spent its run establishing
    /// that the first one was right.
    #[test]
    fn Test_Listing_Label_Should_Report_Declined_For_A_Declined_Item()
    {
        let mut item = Item("T-1");
        item.Decline("superseded by T-2", "agent-a", Timestamp::From_Unix_Seconds(1));
        let document = Board_With(item);
        let found = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listing_Label(&document, found, Timestamp::From_Unix_Seconds(2)),
            "declined"
        );
    }

    #[test]
    fn Test_Listed_As_Should_Report_The_Items_Label_When_No_Filter_Is_Given()
    {
        let document = Board_With(Item("T-1"));
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(&document, item, None, Timestamp::From_Unix_Seconds(0)),
            Some("ready")
        );
    }

    #[test]
    fn Test_Listed_As_Should_Exclude_An_Item_Whose_Label_Does_Not_Match_The_Filter()
    {
        let document = Board_With(Item("T-1"));
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listed_As(&document, item, Some("claimed"), Timestamp::From_Unix_Seconds(0)),
            None,
            "a ready item filtered by `--state claimed` must not be listed"
        );
    }

    #[test]
    fn Test_Nothing_Listed_Should_Name_The_Filter_That_Matched_Nothing()
    {
        let mut output = Vec::new();

        Nothing_Listed(Some("blocked"), &mut output);

        assert_eq!(String::from_utf8(output).unwrap(), "no items are blocked\n");
    }

    #[test]
    fn Test_Nothing_Listed_Should_Say_The_Ledger_Is_Empty_When_No_Filter_Was_Given()
    {
        let mut output = Vec::new();

        Nothing_Listed(None, &mut output);

        assert_eq!(String::from_utf8(output).unwrap(), "the ledger has no items\n");
    }

    #[test]
    fn Test_Print_Listing_Should_Print_The_Items_Identifier_Label_And_Title()
    {
        let item = Item("T-1");
        let mut output = Vec::new();

        Print_Listing(&item, "ready", &mut output);

        let printed = String::from_utf8(output).unwrap();
        assert!(printed.contains("T-1"), "{printed}");
        assert!(printed.contains("ready"), "{printed}");
        assert!(printed.contains("item T-1"), "{printed}");
    }

    #[test]
    fn Test_Print_Listing_Should_Show_The_Holder_Of_A_Claimed_Item()
    {
        let mut item = Item("T-1");
        item.claim = Some(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(1),
            lease_expires_at: Timestamp::From_Unix_Seconds(1_000),
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
        let item = Item("T-1");
        let mut output = Vec::new();

        Print_Claim(&item, Timestamp::From_Unix_Seconds(0), &mut output);

        assert!(output.is_empty(), "an unclaimed item has no claim line to print");
    }

    #[test]
    fn Test_Print_Claim_Should_Mark_A_Lapsed_Claim()
    {
        let mut item = Item("T-1");
        item.claim = Some(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(10),
        });
        let mut output = Vec::new();

        Print_Claim(&item, Timestamp::From_Unix_Seconds(20), &mut output);

        let printed = String::from_utf8(output).unwrap();
        assert!(printed.contains("agent-a"), "{printed}");
        assert!(printed.contains("(lapsed)"), "{printed}");
    }

    #[test]
    fn Test_Print_History_Should_Print_Every_Displacement_Abandonment_And_Verification()
    {
        let mut item = Item("T-1");
        item.displaced.push(nomos_ledger::Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(0),
            lease_expires_at: Timestamp::From_Unix_Seconds(10),
        });
        item.abandoned.push(nomos_ledger::Abandonment {
            holder: "agent-b".to_owned(),
            reason: "superseded".to_owned(),
            abandoned_at: Timestamp::From_Unix_Seconds(20),
        });
        item.verified = Some(VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: String::new(),
            verified_at: Timestamp::From_Unix_Seconds(30),
            gate: None,
            revision: Some("abc123".to_owned()),
        });
        let mut output = Vec::new();

        Print_History(&item, Some("abc123"), &mut output);

        let printed = String::from_utf8(output).unwrap();
        assert!(printed.contains("taken over from agent-a"), "{printed}");
        assert!(printed.contains("abandoned by agent-b"), "{printed}");
        assert!(printed.contains("verified by `cargo test`"), "{printed}");
        assert!(printed.contains("still describes this tree"), "{printed}");
    }

    /// A minimal, ready item: enough to exercise the listing surface without a claim, a
    /// history or a territory that matters to any test here.
    fn Item(id: &str) -> LedgerItem
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
