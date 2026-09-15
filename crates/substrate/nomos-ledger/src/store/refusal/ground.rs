//! What a lapsed lease, an unfinished dependency and somebody else's live claim do to an
//! item: the questions every refusal in `refusal.rs` is built out of.
//!
//! Split out of `refusal.rs`, which keeps the four refusals a verb can be handed and their
//! documentation. Nothing here is part of the crate's public surface: every function is
//! reached either from within this file or through the two entry points `refusal.rs` names,
//! and neither is a rule of its own.

use nomos_platform::Timestamp;

use crate::ClaimRefusal;
use crate::ItemState;
use crate::LedgerDocument;
use crate::LedgerItem;

/// The refusal a lapsed item earns, if it is one.
///
/// One implementation, two callers, for the reason [`super::Claim_Refusal`] itself is a
/// function: this predicate now decides both what `claim` refuses with *and* whether
/// [`crate::FileLedger::Take_Over`] will succeed, and a second copy of it would eventually
/// let a listing say `lapsed` about an item the takeover then declined.
///
/// `now` comes from the caller for the same reason every other judgment here takes it: a
/// second clock read would decide half of one answer against a different instant.
pub(super) fn Lapse_Refusal(target: &LedgerItem, now: Timestamp) -> Option<ClaimRefusal>
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

/// Everything that refuses an item for a reason outside the item's own state: an unfinished
/// dependency, or territory somebody else is actively holding.
///
/// Lifted out of [`super::Claim_Refusal`] unchanged so that [`crate::FileLedger::Take_Over`]
/// re-establishes independence by the same code rather than by a second copy of it. A
/// takeover that skipped this would grant overlapping ground, and the case is not
/// hypothetical: a lapsed claim stops excluding, so another item may since have been claimed
/// over exactly the files this one reserves.
///
/// Reached only through the two entry points that ask it. The rule is not a third one.
pub(super) fn Contested_By(
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
///
/// The territory half of [`Contested_By`] on its own, for the one caller that must not ask
/// the dependency question: a widening of an item already under way cannot be refused
/// because a dependency was declined after the work began.
pub(super) fn Held_Ground(
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Claim;
    use crate::ItemId;
    use crate::ItemKind;
    use crate::ItemOrigin;
    use crate::Territory;

    /// When the claim every fixture below was acquired.
    const CLAIM_ACQUIRED_AT_SECONDS: i64 = 1_000;

    /// When a live claim fixture below expires.
    const CLAIM_EXPIRES_AT_SECONDS: i64 = 9_000;

    /// The instant every test here asks its question at, which is after
    /// `CLAIM_ACQUIRED_AT_SECONDS` and before `CLAIM_EXPIRES_AT_SECONDS`.
    const ASKED_AT_SECONDS: i64 = 2_000;

    /// A lease that had already run out by `ASKED_AT_SECONDS`, so the claim carrying it is
    /// one nobody has taken over.
    const ALREADY_LAPSED_LEASE_SECONDS: i64 = 900;

    #[test]
    fn Test_Lapse_Refusal_Should_Report_A_Claim_Whose_Lease_Has_Run_Out()
    {
        let target = Claimed_Item("agent-a", ALREADY_LAPSED_LEASE_SECONDS);

        assert!(matches!(
            Lapse_Refusal(&target, Timestamp_At_Seconds(ASKED_AT_SECONDS)),
            Some(ClaimRefusal::Lapsed { .. })
        ));
    }

    /// One item, `P1-HELD`, claimed by `holder` until `expires` over `a/shared.rs`.
    ///
    /// Both tests below ask a different question about the same arrangement, so they share
    /// it rather than each rebuilding it -- a change to what "held" means here then changes
    /// for both at once instead of drifting between two hand-written copies.
    fn Claimed_Item(holder: &str, expires: i64) -> LedgerItem
    {
        let mut item = Item_Over("P1-HELD", Territory::Of_Files(["a/shared.rs"]));
        item.state = ItemState::Claimed;
        item.claim = Some(Claim {
            holder: holder.to_owned(),
            acquired_at: Timestamp_At_Seconds(CLAIM_ACQUIRED_AT_SECONDS),
            lease_expires_at: Timestamp_At_Seconds(expires),
        });

        return item;
    }

    fn Timestamp_At_Seconds(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }

    #[test]
    fn Test_Contested_By_Should_Refuse_An_Item_A_Live_Claim_Already_Covers()
    {
        let board = Board_Of(vec![Claimed_Item("agent-a", CLAIM_EXPIRES_AT_SECONDS)]);
        let target = Item_Over("P2-LATER", Territory::Of_Files(["a/shared.rs"]));

        assert!(matches!(
            Contested_By(&board, &target, Timestamp_At_Seconds(ASKED_AT_SECONDS)),
            Some(ClaimRefusal::HeldBy { .. })
        ));
    }

    #[test]
    fn Test_Held_Ground_Should_Refuse_An_Item_A_Live_Claim_Already_Covers()
    {
        let board = Board_Of(vec![Claimed_Item("agent-a", CLAIM_EXPIRES_AT_SECONDS)]);
        let target = Item_Over("P2-LATER", Territory::Of_Files(["a/shared.rs"]));

        assert!(matches!(
            Held_Ground(&board, &target, Timestamp_At_Seconds(ASKED_AT_SECONDS)),
            Some(ClaimRefusal::HeldBy { .. })
        ));
    }

    fn Board_Of(items: Vec<LedgerItem>) -> LedgerDocument
    {
        return LedgerDocument {
            schema_version: crate::SCHEMA_VERSION,
            items,
        };
    }

    /// A `Ready` item, `id`, reserving `territory`.
    fn Item_Over(id: &str, territory: Territory) -> LedgerItem
    {
        let mut item = Item_Named(id);
        item.territory = territory;

        return item;
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
}
