//! Why a claim, a takeover or a decline is refused, decided before anything is written.

// The questions about a lapsed lease, somebody else's live claim and an unfinished
// dependency, which every refusal below is built out of and none of which is a refusal
// this file's callers reach for by name.
#[path = "refusal/ground.rs"]
mod ground;

use nomos_platform::Timestamp;

use crate::ClaimRefusal;
use crate::LedgerItem;
use crate::ItemId;
use crate::ItemState;
use crate::LedgerDocument;
use crate::LedgerError;

/// What would refuse a claim on `item` as of `now`, if anything.
///
/// # Why this is a function rather than a check inside `Claim`
///
/// Two callers need this answer and they must not compute it twice. [`ExclusionLedger::Claim`]
/// asks it to decide whether to grant; a listing asks it to decide what to *call* an item.
/// When those were separate, `work list` read the `state` field alone and printed `ready`
/// for items nothing could take — on 2026-08-09 it said `ready` for eight items while a
/// single held claim refused all eight. An agent picking work off that column burns a round
/// trip per item and learns to distrust the column.
///
/// The fix is not a second guard. Two implementations of one rule is how they come to
/// disagree, and a listing that disagreed with claiming would be worse than one that says
/// too little. So this is the only implementation, and `Claim` is one of its callers.
///
/// The order of the checks is the order a claim refuses in, so the reason reported is the
/// first reason a claimant would actually hit.
#[must_use]
pub fn Claim_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };
    // The lapse check comes before the state check, because `Claimed` is the state a lapsed
    // item is in and reporting it as merely "not claimable" is what left the operator with
    // no next step: true of a `Done` item as well, and the two have opposite remedies.
    // `OD-LEDGER-012`.
    if let Some(refusal) = ground::Lapse_Refusal(target, now)
    {
        return Some(refusal);
    }
    if !target.state.Is_Claimable()
    {
        return Some(ClaimRefusal::NotClaimable {
            item: item.clone(),
            state: target.state.Describe(),
        });
    }

    return ground::Contested_By(document, target, now);
}

/// Every item [`Claim_Refusal`] would grant right now, in the order a session should offer
/// them.
///
/// `Claim_Refusal` already exists so a listing and a claim cannot come to disagree about
/// whether one item is takeable; this runs that same check over the whole board instead of
/// one item, so "which one, in what order" stops being a question answered by scanning a
/// snapshot and starts being a question the ledger answers the same way every time it is
/// asked. `OD-LEDGER-023`.
///
/// Ordered by [`ItemId`] — the one key every item already carries, and the only one two
/// sessions reading the same board are guaranteed to compare identically. `OD-LEDGER-017`
/// deferred a `priority` field rather than adding one that would satisfy the letter of
/// the corpus requirement while ageing worst of the three ranking keys it named; id order
/// is this function's tie-break for as long as that stands, not a placeholder standing in
/// for a ranking this function does not attempt.
#[must_use]
pub fn Eligible_Items(document: &LedgerDocument, now: Timestamp) -> Vec<&LedgerItem>
{
    let mut eligible: Vec<&LedgerItem> = document
        .items
        .iter()
        .filter(|item| return Claim_Refusal(document, &item.id, now).is_none())
        .collect();

    eligible.sort_by(|left, right| return left.id.cmp(&right.id));

    return eligible;
}

/// Why an item may not be declined, if it may not.
///
/// Only a `Ready` item can be ended, and the three refusals below are the three ways of not
/// being one. `OD-LEDGER-019` decision 4 states each; what matters here is the *order*, which
/// is the order a caller hits them so that the reason reported is the first one that actually
/// applies — the same discipline [`Claim_Refusal`] follows, and the lapse check is first for
/// the same reason it is first there.
///
/// [`ground::Contested_By`] is deliberately not consulted. Territory contention decides who
/// may *work* an item; it has nothing to say about whether the item is work at all, and a
/// decline refused because some unrelated peer holds an overlapping file would be a refusal
/// nobody could act on.
pub(super) fn Decline_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };
    // The lapse check is first, because `Claimed` is the state a lapsed item is in and the
    // remedies differ: a live holder is asked to release, and a dead one is taken over.
    // `OD-LEDGER-012`.
    if let Some(refusal) = ground::Lapse_Refusal(target, now)
    {
        return Some(refusal);
    }

    return Held_Or_Unready(target);
}

/// What a live claim or a state other than `Ready` does to a decline.
///
/// `Describe` rather than a bare state word, so a `Declined` item's refusal carries the
/// reason it already holds. Told only "it is declined", a caller cannot tell a duplicate
/// from a disagreement, and both need a person.
fn Held_Or_Unready(target: &LedgerItem) -> Option<ClaimRefusal>
{
    if let Some(claim) = target.claim.as_ref()
    {
        return Some(ClaimRefusal::StillHeld {
            item: target.id.clone(),
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
        });
    }
    if target.state != ItemState::Ready
    {
        return Some(ClaimRefusal::NotClaimable {
            item: target.id.clone(),
            state: target.state.Describe(),
        });
    }

    return None;
}

/// Why `holder` may not widen `item` by `adding` as of `now`, if they may not.
///
/// # Why the enlarged territory and not the added paths
///
/// The contention question is about the territory the item *would have*, so the check is run
/// over exactly that: the target is cloned, [`LedgerItem::Widen`] is applied to the clone, and
/// the result is handed to [`ground::Held_Ground`]. Asking instead whether each added path
/// collides would be a second notion of what a widening produces, free to disagree with the
/// one that applies it -- and the answer that matters is the one the write will make true.
///
/// [`ground::Held_Ground`] and not [`ground::Contested_By`]. The dependency half of that pair
/// asks whether an item may *start*, which a claimed item has already answered; re-asking it
/// here would refuse a widening because a dependency was declined after the work began, which
/// is a real problem and not this verb's to report.
///
/// # Why liveness and not the holder's name
///
/// The lapse check is first, as it is in [`Claim_Refusal`] and [`Decline_Refusal`], and here it
/// is more than message ordering. A lapsed claim stops excluding -- which is what
/// [`ground::Contested_By`] already records for takeovers -- so another item may since have
/// been claimed over exactly the files this one reserves. A holder whose lease has gone,
/// enlarging a reservation that currently excludes nobody, is that hazard reached through a
/// new verb, and their name still matching is precisely what makes it look permissible. They
/// are told to take the item over first.
pub(super) fn Widen_Refusal(
    document: &LedgerDocument,
    requested: &Enlargement<'_>,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let item = requested.item;
    let holder = requested.holder;

    let Some(target) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };

    if let Some(refusal) = ground::Lapse_Refusal(target, now)
    {
        return Some(refusal);
    }

    if let Some(refusal) = Held_Or_Unclaimed(target, holder)
    {
        return Some(refusal);
    }

    return Refused_By_The_Enlargement(document, target, requested, now);
}

/// The widening being asked for -- grouped so this stays under the workspace's own
/// parameter-count ceiling, the same reason [`Claimant`] is grouped in `validation.rs`.
pub(super) struct Enlargement<'a>
{
    /// Which item is to be enlarged.
    pub(super) item: &'a ItemId,
    /// Who is asking, which must be the live holder.
    pub(super) holder: &'a str,
    /// The paths to add. Never a replacement territory.
    pub(super) adding: &'a [String],
}

/// What a claim held by somebody else, or no live claim at all, does to a widening.
///
/// `Claimed` and a live claim are two facts and both are required. The state alone would admit
/// an item whose claim record is missing, which `Validate` calls a corruption rather than a
/// thing to widen; the claim alone would admit one that has been finished or declined out from
/// under a holder who never released it. They are two arms because their remedies differ: a
/// wrong holder is somebody to wait for, and a state that is not `Claimed` is not this verb's
/// to fix at all.
fn Held_Or_Unclaimed(target: &LedgerItem, holder: &str) -> Option<ClaimRefusal>
{
    let Some(claim) = target.claim.as_ref().filter(|_| return target.state == ItemState::Claimed)
    else
    {
        return Some(ClaimRefusal::NotClaimable {
            item: target.id.clone(),
            state: target.state.Describe(),
        });
    };

    if claim.holder != holder
    {
        return Some(ClaimRefusal::StillHeld {
            item: target.id.clone(),
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
        });
    }

    return None;
}

/// The item as the widening would leave it, judged against the ground everybody else holds.
///
/// [`LedgerItem::Widen`] and not two statements at the call site, for the reason the verb
/// itself calls it: a call site that grew the territory itself would be free to grow it and
/// not record what it grew by. What `Widen` reports is discarded deliberately -- it answers
/// what changed, and the question here is what the item would *hold*.
fn Refused_By_The_Enlargement(
    document: &LedgerDocument,
    target: &LedgerItem,
    requested: &Enlargement<'_>,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let mut enlarged = target.clone();
    let added = enlarged.Widen(requested.adding, requested.holder, now);
    drop(added);

    return ground::Held_Ground(document, &enlarged, now);
}

/// What refuses a takeover of `item` as of `now`, if anything.
///
/// The mirror of [`Claim_Refusal`], differing in exactly one clause: a claim needs the item to
/// be free and this needs it to be lapsed. Everything after that question is the same code,
/// which is the point — [`ground::Contested_By`] is called and not copied, so a takeover
/// cannot come to disagree with a claim about whether two territories are independent.
pub(super) fn Takeover_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document
        .items
        .iter()
        .find(|candidate| &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };

    // A takeover answers a lapse and nothing else. An item with a live claim is a queue, and
    // everything else is the caller reaching for the wrong verb.
    if ground::Lapse_Refusal(target, now).is_none()
    {
        return Some(Wrong_Verb(target, now));
    }

    return ground::Contested_By(document, target, now);
}

/// What to say to a caller that used `takeover` on an item that has not lapsed.
///
/// A live claim is [`ClaimRefusal::HeldBy`] — retryable, because the lease running out is what
/// resolves it, and telling an agent to wait is the honest answer when somebody is working.
/// Everything else is the state word, so that `takeover` against a `Ready` or `Done` item reads
/// as the wrong verb rather than as a queue that will never clear.
fn Wrong_Verb(target: &LedgerItem, now: Timestamp) -> ClaimRefusal
{
    if let Some(claim) = &target.claim
        && !claim.Has_Lapsed(now)
    {
        return ClaimRefusal::HeldBy {
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
            item: target.id.clone(),
        };
    }

    return ClaimRefusal::NotClaimable {
        item: target.id.clone(),
        state: target.state.Describe(),
    };
}

/// A store failure, reported as itself rather than as a missing item.
///
/// Every load and save in the three operations below used to discard its error and return
/// [`ClaimRefusal::NoSuchItem`], so an unreadable file, a parse error and an invalid
/// document all told the operator that their identifier matched nothing. That is a reason
/// destroyed by the failure it explains, and it is why `P10-LAPSE-BRICKS` needed an
/// experiment to diagnose rather than a glance: the surface was reporting a spelling
/// mistake while the ledger was refusing to load.
///
/// A conversion rather than a free function since [`FileLedger::Decide_Under_Lock`] became
/// generic over what its caller refuses with: this is the one thing every such vocabulary
/// has to be able to say, so it is the bound rather than a step each verb performs.
impl From<&LedgerError> for ClaimRefusal
{
    fn from(error: &LedgerError) -> Self
    {
        return Self::LedgerUnusable {
            cause: error.to_string(),
        };
    }
}

#[cfg(test)]
#[path = "refusal/tests.rs"]
mod tests;

/// Narrow, file-local proofs for each of this file's own visible functions, addressed by
/// name.
///
/// [`tests`] above is `refusal/tests.rs`, a separate physical file whose behavioural suite
/// this does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's
/// companion unit off the literal file it is textually written in, so a test living in that
/// separate file can never address a function declared here, however it is named — this
/// module gives each function here the one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use crate::Claim;
    use crate::ItemKind;
    use crate::ItemOrigin;
    use crate::Territory;

    /// When the held board's claim was acquired.
    const CLAIM_ACQUIRED_AT_SECONDS: i64 = 1_000;

    /// When the held board's claim expires.
    const CLAIM_EXPIRES_AT_SECONDS: i64 = 9_000;

    /// The instant every test here asks its question at, which is after
    /// `CLAIM_ACQUIRED_AT_SECONDS` and before `CLAIM_EXPIRES_AT_SECONDS`.
    const ASKED_AT_SECONDS: i64 = 2_000;

    #[test]
    fn Test_Claim_Refusal_Should_Report_No_Such_Item_When_Absent()
    {
        let empty = LedgerDocument {
            schema_version: crate::SCHEMA_VERSION,
            items: Vec::new(),
        };

        let refusal = Claim_Refusal(&empty, &ItemId::New("GHOST"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

        assert_eq!(refusal, Some(ClaimRefusal::NoSuchItem { item: ItemId::New("GHOST") }));
    }

    #[test]
    fn Test_Eligible_Items_Should_Exclude_An_Item_A_Live_Claim_Holds()
    {
        let document = Held_Board();

        assert!(
            Eligible_Items(&document, Timestamp_At_Seconds(ASKED_AT_SECONDS)).is_empty(),
            "P1-HELD is claimed, so Claim_Refusal refuses it and the listing must agree"
        );
    }

    #[test]
    fn Test_Decline_Refusal_Should_Refuse_An_Item_Someone_Else_Is_Holding()
    {
        let document = Held_Board();

        let refusal = Decline_Refusal(&document, &ItemId::New("P1-HELD"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

        assert!(
            matches!(refusal, Some(ClaimRefusal::StillHeld { .. })),
            "got {refusal:?}"
        );
    }

    #[test]
    fn Test_Takeover_Refusal_Should_Refuse_A_Verb_Used_On_A_Live_Claim()
    {
        let document = Held_Board();

        // The claim below expires at 9_000; asking at 2_000 means it has not lapsed, so a
        // takeover is the wrong verb rather than a valid recovery.
        let refusal = Takeover_Refusal(&document, &ItemId::New("P1-HELD"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

        assert!(
            matches!(refusal, Some(ClaimRefusal::HeldBy { .. })),
            "a live claim asked about with the recovery verb must read as held, not takeable: {refusal:?}"
        );
    }

    /// A board holding exactly one item, `P1-HELD`, live-claimed by `agent-a` over
    /// `a/shared.rs`.
    ///
    /// Three of the tests above ask a different verb about the same claimed item, so they
    /// share this one arrangement rather than each rebuilding it -- a change to what "held"
    /// means here then changes for all three at once instead of drifting between copies.
    fn Held_Board() -> LedgerDocument
    {
        let mut held = Item_Named("P1-HELD");
        held.territory = Territory::Of_Files(["a/shared.rs"]);
        held.state = ItemState::Claimed;
        held.claim = Some(Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp_At_Seconds(CLAIM_ACQUIRED_AT_SECONDS),
            lease_expires_at: Timestamp_At_Seconds(CLAIM_EXPIRES_AT_SECONDS),
        });

        return LedgerDocument {
            schema_version: crate::SCHEMA_VERSION,
            items: vec![held],
        };
    }

    fn Item_Named(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Empty(),
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

    fn Timestamp_At_Seconds(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }
}
