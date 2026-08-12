//! Printing an item: how it stands on the board, and everything recorded against it.

use nomos_ledger::{Claim_Refusal, ClaimRefusal, ItemState, LedgerDocument, LedgerItem};
use nomos_platform::Timestamp;

use super::report::Refusal_Label;

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

/// One item's line: its identifier, what it may be called now, its holder, and its title.
pub(super) fn Print_Listing(item: &LedgerItem, label: &str, output: &mut impl std::io::Write)
{
    let holder = item
        .claim
        .as_ref()
        .map_or_else(String::new, |claim| return format!("  [{}]", claim.holder));

    let _ = writeln!(output, "{:<13} {:<9}{holder}  {}", item.id, label, item.title);
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
pub(super) fn Print_History(found: &LedgerItem, output: &mut impl std::io::Write)
{
    Print_Displacements(found, output);
    Print_Abandonments(found, output);
    Print_Verification(found, output);
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
fn Print_Verification(found: &LedgerItem, output: &mut impl std::io::Write)
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
