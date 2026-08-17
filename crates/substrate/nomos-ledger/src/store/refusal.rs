//! Why a claim, a takeover or a decline is refused, decided before anything is written.

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
    if let Some(refusal) = Lapse_Refusal(target, now)
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

    return Contested_By(document, target, now);
}

/// The refusal a lapsed item earns, if it is one.
///
/// One implementation, two callers, for the reason [`Claim_Refusal`] itself is a function:
/// this predicate now decides both what `claim` refuses with *and* whether
/// [`FileLedger::Take_Over`] will succeed, and a second copy of it would eventually let a
/// listing say `lapsed` about an item the takeover then declined.
///
/// `now` comes from the caller for the same reason every other judgment here takes it: a
/// second clock read would decide half of one answer against a different instant.
fn Lapse_Refusal(target: &LedgerItem, now: Timestamp) -> Option<ClaimRefusal>
{
    if target.state != ItemState::Claimed
    {
        return None;
    }

    let claim = target.claim.as_ref()?;
    if !claim.Has_Lapsed(now)
    {
        return None;
    }

    return Some(ClaimRefusal::Lapsed {
        item: target.id.clone(),
        holder: claim.holder.clone(),
        since: claim.lease_expires_at,
    });
}

/// Why an item may not be declined, if it may not.
///
/// Only a `Ready` item can be ended, and the three refusals below are the three ways of not
/// being one. `OD-LEDGER-019` decision 4 states each; what matters here is the *order*, which
/// is the order a caller hits them so that the reason reported is the first one that actually
/// applies — the same discipline [`Claim_Refusal`] follows, and the lapse check is first for
/// the same reason it is first there.
///
/// [`Contested_By`] is deliberately not consulted. Territory contention decides who may *work*
/// an item; it has nothing to say about whether the item is work at all, and a decline refused
/// because some unrelated peer holds an overlapping file would be a refusal nobody could act
/// on.
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
    if let Some(refusal) = Lapse_Refusal(target, now)
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

/// Everything that refuses an item for a reason outside the item's own state: an unfinished
/// dependency, or territory somebody else is actively holding.
///
/// Lifted out of [`Claim_Refusal`] unchanged so that [`FileLedger::Take_Over`] re-establishes
/// independence by the same code rather than by a second copy of it. A takeover that skipped
/// this would grant overlapping ground, and the case is not hypothetical: a lapsed claim stops
/// excluding, so another item may since have been claimed over exactly the files this one
/// reserves.
///
/// Private. Both entry points are public and the rule is not a third one.
fn Contested_By(
    document: &LedgerDocument,
    target: &LedgerItem,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    if let Some(refusal) = Unmet_Dependency(document, target)
    {
        return Some(refusal);
    }

    return Held_Ground(document, target, now);
}

/// The first dependency of `target` that is not done, as a refusal.
///
/// A dependency that is `Declined` is reported through its own arm rather than
/// [`ClaimRefusal::DependencyUnmet`] — `OD-LEDGER-020` is why: it will never become `Done`, so
/// the two must not share a refusal whose retryability tells the caller to wait.
fn Unmet_Dependency(document: &LedgerDocument, target: &LedgerItem) -> Option<ClaimRefusal>
{
    for dependency in &target.depends_on
    {
        let Some(found) = document.items.iter().find(|candidate| return &candidate.id == dependency)
        else
        {
            return Some(ClaimRefusal::DependencyUnmet {
                item: target.id.clone(),
                dependency: dependency.clone(),
                state: "not in the ledger".to_owned(),
            });
        };

        if let ItemState::Declined { .. } = &found.state
        {
            return Some(ClaimRefusal::DependencyDeclined {
                item: target.id.clone(),
                dependency: dependency.clone(),
                state: found.state.Describe(),
            });
        }

        if found.state != ItemState::Done
        {
            return Some(ClaimRefusal::DependencyUnmet {
                item: target.id.clone(),
                dependency: dependency.clone(),
                state: found.state.Describe(),
            });
        }
    }

    return None;
}

/// The first active claim whose territory `target` cannot be shown independent of.
fn Held_Ground(
    document: &LedgerDocument,
    target: &LedgerItem,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    for other in &document.items
    {
        if other.id == target.id || !other.Has_Active_Claim(now)
        {
            continue;
        }

        let refusal = Refused_By(target, other);
        if refusal.is_some()
        {
            return refusal;
        }
    }

    return None;
}

/// What `other`'s live claim does to `target`, if anything.
fn Refused_By(target: &LedgerItem, other: &LedgerItem) -> Option<ClaimRefusal>
{
    use crate::exclusion::Refusal_From;

    let claim = other.claim.as_ref()?;
    let overlap = target.territory.Intersect(&other.territory);

    return Refusal_From(&overlap, &other.id, &claim.holder, claim.lease_expires_at);
}

/// What refuses a takeover of `item` as of `now`, if anything.
///
/// The mirror of [`Claim_Refusal`], differing in exactly one clause: a claim needs the item to
/// be free and this needs it to be lapsed. Everything after that question is the same code,
/// which is the point — [`Contested_By`] is called and not copied, so a takeover cannot come to
/// disagree with a claim about whether two territories are independent.
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
    if Lapse_Refusal(target, now).is_none()
    {
        return Some(Wrong_Verb(target, now));
    }

    return Contested_By(document, target, now);
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
