//! Taking and holding territory.

use crate::item::{
    Abandonment, ItemId, ItemState, LedgerItem, MAXIMUM_LEASE, VerificationRecord,
};
use crate::territory::Territory;
use nomos_model::{Intersection, UnknownReason};
use nomos_platform::Timestamp;
use std::time::Duration;

/// Why a claim was refused.
///
/// Every variant tells the caller something different about what to do next, which is
/// why this is not a `bool` or a string. An agent that is told "held by agent-b until
/// 14:30" waits or picks something else; an agent told "independence could not be
/// established" has found a modelling gap somebody needs to close.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimRefusal
{
    /// Somebody else holds overlapping territory.
    HeldBy
    {
        /// Who holds it.
        holder: String,
        /// When their lease lapses.
        until: Timestamp,
        /// The item they hold.
        item: ItemId,
    },
    /// Whether the two territories overlap could not be established.
    ///
    /// **Not** a permission to proceed. This is the arm that keeps "we could not tell"
    /// from becoming "go ahead".
    UnknownIndependence
    {
        /// The item whose territory could not be compared.
        against: ItemId,
        /// Why the comparison failed.
        reason: UnknownReason,
    },
    /// The requested lease exceeds [`MAXIMUM_LEASE`].
    LeaseTooLong
    {
        /// What was asked for.
        requested: Duration,
        /// The ceiling.
        maximum: Duration,
    },
    /// The item is not in a state that can be claimed.
    NotClaimable
    {
        /// The item.
        item: ItemId,
        /// What state it is in.
        state: String,
    },
    /// Something this item depends on is not finished.
    ///
    /// A dependency edge that only `validate` reads is a comment. This is the arm that
    /// makes it a constraint, and it is retryable because finishing the dependency is
    /// what resolves it.
    DependencyUnmet
    {
        /// The item that was refused.
        item: ItemId,
        /// The dependency that is not done.
        dependency: ItemId,
        /// What state that dependency is in.
        state: String,
    },
    /// No such item.
    NoSuchItem
    {
        /// The identifier that matched nothing.
        item: ItemId,
    },
}

impl ClaimRefusal
{
    /// A one-line explanation a person or an agent can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::HeldBy {
                holder,
                until,
                item,
            } => format!(
                "{item} overlaps territory held by {holder} until unix {}",
                until.Unix_Seconds()
            ),
            Self::UnknownIndependence { against, reason } => format!(
                "cannot establish independence from {against}: {}. Unknown independence is \
                 not safe parallelism, so this claim is refused rather than granted",
                reason.Describe()
            ),
            Self::LeaseTooLong { requested, maximum } => format!(
                "a lease of {requested:?} exceeds the {maximum:?} ceiling"
            ),
            Self::NotClaimable { item, state } => {
                format!("{item} is {state}, so the operation was refused")
            }
            Self::DependencyUnmet {
                item,
                dependency,
                state,
            } => format!("{item} depends on {dependency}, which is {state}"),
            Self::NoSuchItem { item } => format!("no item named {item}"),
        };
    }

    /// Whether retrying later might succeed.
    ///
    /// Distinguishes a queue from a dead end, which is what lets an agent decide
    /// between waiting and finding other work.
    #[must_use]
    pub const fn Is_Retryable(&self) -> bool
    {
        return matches!(self, Self::HeldBy { .. } | Self::DependencyUnmet { .. });
    }
}

/// A granted claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation
{
    /// The item claimed.
    pub item: ItemId,
    /// Who holds it.
    pub holder: String,
    /// When the lease lapses.
    pub expires_at: Timestamp,
}

/// How a release ended.
///
/// The finished arm carries the [`VerificationRecord`], which is what makes a finished
/// item structurally impossible without one. An earlier shape had a bare `Finished`
/// variant, and it did not work: the release set the item to `Done`, the item had no
/// recorded verification, and the ledger's own validation then refused the write — so
/// the only way to finish anything was a path that could never succeed, reported as
/// "no such item". Carrying the record moves the requirement from a rule that rejects
/// the write to a signature that cannot express it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseOutcome
{
    /// The work was finished, and this is the evidence.
    Finished(VerificationRecord),
    /// The work was abandoned; the item returns to being claimable.
    Abandoned
    {
        /// Why.
        reason: String,
    },
}

impl ReleaseOutcome
{
    /// Writes this outcome onto the item it happened to, and ends the claim.
    ///
    /// The single place a release's evidence is persisted, for the same reason
    /// [`Refusal_From`] is the single place independence is judged: an implementation
    /// that spells the rule out for itself is free to spell one arm of it and not the
    /// other. That is not hypothetical — it is how this function came to exist. The
    /// `Finished` arm carried its [`VerificationRecord`] into storage and the `Abandoned`
    /// arm matched `{ .. }` and dropped the reason, adjacent arms of one match, one
    /// keeping its evidence and one discarding it. Both arms are now written once, next
    /// to the type that carries the evidence, so a second [`ExclusionLedger`] cannot
    /// persist half of it.
    ///
    /// Ending the claim is part of this rather than left to the caller. An abandonment is
    /// a record of something that stopped, and a record that went on excluding people
    /// would be a worse defect than the one this fixes.
    pub fn Record_On(&self, item: &mut LedgerItem, holder: &str, at: Timestamp)
    {
        item.claim = None;

        match self
        {
            Self::Finished(record) =>
            {
                item.state = ItemState::Done;
                item.verified = Some(record.clone());
            }
            Self::Abandoned { reason } =>
            {
                item.state = ItemState::Ready;
                item.abandoned.push(Abandonment {
                    holder: holder.to_owned(),
                    reason: reason.clone(),
                    abandoned_at: at,
                });
            }
        }
    }
}

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

    /// Everything that would stop `territory` from being claimed right now.
    fn Conflicts(&self, territory: &Territory) -> Vec<ClaimRefusal>;
}

/// Checks a requested lease against the ceiling.
///
/// # Errors
///
/// Returns [`ClaimRefusal::LeaseTooLong`] when the request exceeds [`MAXIMUM_LEASE`].
pub fn Check_Lease(requested: Duration) -> Result<(), ClaimRefusal>
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
pub fn Refusal_From(
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
