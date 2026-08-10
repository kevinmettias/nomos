//! Taking and holding territory.

use crate::item::{
    Abandonment, ItemId, ItemState, LedgerItem, MAXIMUM_LEASE, VerificationRecord,
};
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
    /// The item's holder is gone: the lease ran out and nobody renewed it.
    ///
    /// Distinct from [`ClaimRefusal::NotClaimable`] because the remedy is distinct and
    /// nameable. A `Done` item is a dead end; a lapsed one is takeable by anybody willing to
    /// say so, and a refusal that does not say which of the two it is sends the caller to
    /// read the JSON. `OD-LEDGER-005` is this repository's record of what a state word costs
    /// when it means something other than what a reader takes it to mean, and `claimed`
    /// covering both of these was the second instance.
    ///
    /// Not retryable, and that is the arm's point rather than an oversight: no amount of
    /// waiting turns a dead holder into a live one. Somebody has to decide to take the work.
    Lapsed
    {
        /// The item.
        item: ItemId,
        /// Who held it when the lease ran out.
        holder: String,
        /// When it ran out.
        since: Timestamp,
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
    /// The ledger itself could not be read or written.
    ///
    /// Nothing to do with the item, and that is why it is its own arm. Every load and save
    /// failure used to be reported as [`ClaimRefusal::NoSuchItem`], so an unreadable file,
    /// a parse error and an invalid document all told the operator their identifier was
    /// wrong — sending them to check a spelling while the ledger was broken. `OD-LEDGER-009`
    /// records it as the third instance of a reason not surviving the failure it explains.
    LedgerUnusable
    {
        /// What the store said, verbatim.
        cause: String,
    },
}

impl ClaimRefusal
{
    /// A one-line explanation a person or an agent can act on.
    ///
    /// # The refused item is never the grammatical subject here
    ///
    /// This is the form for a caller that has **not** already named the item it asked about,
    /// and every arm is phrased so that it composes under one that has. The distinction is
    /// not cosmetic: several arms carry the identifier of a *different* item — the blocker in
    /// [`ClaimRefusal::HeldBy`], the unfinished dependency in
    /// [`ClaimRefusal::DependencyUnmet`] — and a sentence that opens with one of those names
    /// reads as a statement about the wrong item.
    ///
    /// `OD-LEDGER-014` records the measurement. The held arm used to read
    /// `{blocker} overlaps territory held by {holder}`, which is true standing alone and says
    /// the reverse of what happened the moment a caller prints it beneath the subject's own
    /// identifier: `work audit` emitted `P1-MODEL: P9-AUTHORING overlaps territory held by …`
    /// for forty-four items, and no reader could tell which of the two names was refused.
    ///
    /// So a caller that has named its subject prints this straight after it, and one that has
    /// not still gets a sentence that claims nothing false. Anything needing a different
    /// phrasing reads the identifiers off the variant rather than writing a second rendering
    /// — that second rendering is what this record removed.
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
                "territory overlaps {item}, held by {holder} until unix {}",
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
            Self::Lapsed {
                item,
                holder,
                since,
            } => format!(
                "{item} was held by {holder} and the lease ran out at unix {}; \
                 `nomos work takeover` replaces it and keeps {holder}'s claim on the item",
                since.Unix_Seconds()
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
            Self::LedgerUnusable { cause } => {
                format!("the ledger could not be used: {cause}")
            }
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
