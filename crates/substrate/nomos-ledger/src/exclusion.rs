//! Taking and holding territory.

use crate::release_outcome::ReleaseOutcome;
use crate::claim_refusal::ClaimRefusal;
use crate::reservation::Reservation;
use crate::item_id::ItemId;
use crate::item::MAXIMUM_LEASE;
use nomos_model::Intersection;
use nomos_platform::Timestamp;
use std::time::Duration;

/// Territory-based mutual exclusion.
///
/// One trait, several instances with different lifetimes and authority: the durable
/// build ledger committed to the repository, the run-scoped reservations a correction
/// scheduler uses for parallel waves, and the session-scoped leases held by delegated
/// agents. They differ in where they persist and who may write them; the exclusion
/// question is identical, and answering it three ways is three chances to disagree
/// about whether two writers may proceed.
pub trait ExclusionLedger
{
    /// Takes territory for `holder`.
    ///
    /// # Errors
    ///
    /// Returns a [`ClaimRefusal`] naming what stopped it.
    fn Claim(
        &mut self,
        item: &ItemId,
        holder: &str,
        lease: Duration,
    ) -> Result<Reservation, ClaimRefusal>;

    /// Extends an existing claim.
    ///
    /// Deliberately cheaper than re-claiming: a holder that is still working should not
    /// have to re-establish independence it already established, and making renewal
    /// expensive is how leases end up set too long.
    ///
    /// # Errors
    ///
    /// Returns a [`ClaimRefusal`] if the claim is no longer the caller's to renew.
    fn Renew(&mut self, item: &ItemId, holder: &str, lease: Duration)
    -> Result<Reservation, ClaimRefusal>;

    /// Gives up a claim.
    ///
    /// # Errors
    ///
    /// Returns a [`ClaimRefusal`] if the item is not held by this holder.
    fn Release(
        &mut self,
        item: &ItemId,
        holder: &str,
        outcome: ReleaseOutcome,
    ) -> Result<(), ClaimRefusal>;
}

/// Checks a requested lease against the ceiling.
///
/// # Errors
///
/// Returns [`ClaimRefusal::LeaseTooLong`] when the request exceeds [`MAXIMUM_LEASE`].
pub(crate) fn Check_Lease(requested: Duration) -> Result<(), ClaimRefusal>
{
    if requested > MAXIMUM_LEASE
    {
        return Err(ClaimRefusal::LeaseTooLong {
            requested,
            maximum: MAXIMUM_LEASE,
        });
    }
    return Ok(());
}

/// Turns an overlap answer into a refusal, if it is one.
///
/// The single place the "unknown is not permission" rule is applied, so that no caller
/// can implement it slightly differently. Note that both non-`Disjoint` arms produce a
/// refusal: there is no code path here that turns an unanswered question into a grant.
#[must_use]
pub(crate) fn Refusal_From(
    intersection: &Intersection,
    against: &ItemId,
    holder: &str,
    until: Timestamp,
) -> Option<ClaimRefusal>
{
    return match intersection
    {
        Intersection::Disjoint => None,
        Intersection::Overlaps(_) => Some(ClaimRefusal::HeldBy {
            holder: holder.to_owned(),
            until,
            item: against.clone(),
        }),
        Intersection::Unknown(reason) => Some(ClaimRefusal::UnknownIndependence {
            against: against.clone(),
            reason: reason.clone(),
        }),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::UnknownReason;
    use nomos_model::SetResolution;

    fn At(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }

    #[test]
    fn Test_A_Lease_Within_The_Ceiling_Should_Be_Accepted()
    {
        assert!(Check_Lease(Duration::from_secs(3_600)).is_ok());
        assert!(Check_Lease(MAXIMUM_LEASE).is_ok());
    }

    #[test]
    fn Test_A_Lease_Beyond_The_Ceiling_Should_Be_Refused()
    {
        let refusal = Check_Lease(MAXIMUM_LEASE + Duration::from_secs(1)).unwrap_err();

        assert!(matches!(refusal, ClaimRefusal::LeaseTooLong { .. }));
        assert!(!refusal.Is_Retryable(), "waiting will not shorten the request");
    }

    #[test]
    fn Test_Disjoint_Territory_Should_Produce_No_Refusal()
    {
        let refusal = Refusal_From(
            &Intersection::Disjoint,
            &ItemId::New("T-1"),
            "agent-a",
            At(2_000),
        );

        assert!(refusal.is_none());
    }

    /// The rule this module exists to centralize. An unanswerable overlap question must
    /// produce a refusal, never a grant.
    #[test]
    fn Test_Unknown_Independence_Should_Refuse()
    {
        let unknown = Intersection::Unknown(UnknownReason::IncomparableResolution {
            left: SetResolution::File,
            right: SetResolution::Symbol,
        });

        let refusal = Refusal_From(&unknown, &ItemId::New("T-1"), "agent-a", At(2_000))
            .expect("unknown independence must refuse");

        assert!(matches!(refusal, ClaimRefusal::UnknownIndependence { .. }));
        assert!(
            !refusal.Is_Retryable(),
            "waiting does not resolve a modelling gap; somebody has to close it"
        );
    }

    /// A held item is a queue, not a wall. An agent must be able to tell the difference
    /// so it can wait rather than give up, or move on rather than spin.
    #[test]
    fn Test_A_Held_Item_Should_Be_Retryable()
    {
        let overlaps = Intersection::Overlaps(Vec::new());

        let refusal = Refusal_From(&overlaps, &ItemId::New("T-1"), "agent-b", At(2_000))
            .expect("an overlap must refuse");

        assert!(refusal.Is_Retryable());
        assert!(refusal.Describe().contains("agent-b"));
    }

    /// A lapse is the one refusal whose remedy is a verb, so the refusal has to name it.
    ///
    /// This is the whole difference between `Lapsed` and the `NotClaimable` it replaced for
    /// this case. `NotClaimable` said the item was `Claimed` — true, and indistinguishable
    /// from `Done`, which is why the operator was left with no next step. Waiting is not that
    /// step either: a dead holder does not come back, so this must not be retryable.
    #[test]
    fn Test_A_Lapse_Should_Name_Its_Holder_And_Its_Remedy()
    {
        let refusal = ClaimRefusal::Lapsed {
            item: ItemId::New("T-1"),
            holder: "dead-agent".to_owned(),
            since: At(2_000),
        };

        let said = refusal.Describe();
        assert!(said.contains("dead-agent"), "{said}");
        assert!(said.contains("takeover"), "the refusal must name the remedy: {said}");
        assert!(said.contains("2000"), "{said}");
        assert!(
            !refusal.Is_Retryable(),
            "waiting does not turn a dead holder into a live one; somebody has to take the work"
        );
    }

    #[test]
    fn Test_Every_Refusal_Should_Describe_Itself_Usefully()
    {
        let refusals = [
            ClaimRefusal::HeldBy {
                holder: "agent-a".to_owned(),
                until: At(2_000),
                item: ItemId::New("T-1"),
            },
            ClaimRefusal::UnknownIndependence {
                against: ItemId::New("T-2"),
                reason: UnknownReason::IncomparableSnapshots,
            },
            ClaimRefusal::LeaseTooLong {
                requested: Duration::from_secs(9_999_999),
                maximum: MAXIMUM_LEASE,
            },
            ClaimRefusal::NotClaimable {
                item: ItemId::New("T-3"),
                state: "Done".to_owned(),
            },
            ClaimRefusal::Lapsed {
                item: ItemId::New("T-5"),
                holder: "dead-agent".to_owned(),
                since: At(2_000),
            },
            ClaimRefusal::NoSuchItem {
                item: ItemId::New("T-4"),
            },
        ];

        for refusal in &refusals
        {
            assert!(
                refusal.Describe().len() > 15,
                "{} is too terse to act on",
                refusal.Describe()
            );
        }
    }
}
