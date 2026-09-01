//! Every way a claim is refused, in the words the holder gets.

// `Layer` is its own public type and keeps its own file. The one-line renderings
// behind `Describe` are private and keep theirs too, split out by responsibility; neither
// is part of the crate's public surface, so only `Refusal` itself stays here.
#[path = "refusal/layer.rs"]
mod layer;
#[path = "refusal/message.rs"]
mod message;

pub use layer::Layer;

use std::time::Duration;
use nomos_model::UnknownReason;
use crate::ItemId;
use nomos_platform::Timestamp;
/// Why a claim was refused.
///
/// Every variant tells the caller something different about what to do next, which is
/// why this is not a `bool` or a string. An agent that is told "held by agent-b until
/// 14:30" waits or picks something else; an agent told "independence could not be
/// established" has found a modelling gap somebody needs to close.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal
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
    /// Distinct from [`Self::NotClaimable`] because the remedy is distinct and
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
    /// The item itself is held, and the operation would end somebody's live work.
    ///
    /// Distinct from [`Self::HeldBy`], which is about a *different* item whose
    /// territory overlaps the one asked for. Here the subject and the blocker are the same
    /// item, and a sentence about overlapping territory would be false about it — the
    /// mis-subject `OD-LEDGER-014` measured, in the one place reusing that arm would have
    /// reintroduced it.
    ///
    /// Retryable, and for a stronger reason than most: a claim is a lease, so it is released
    /// or it lapses. The remedy is named in the sentence because it is two commands and the
    /// first of them is one only the holder can run. `OD-LEDGER-001` is why it has to be
    /// theirs — territory is declared and not enforced, so the holder is the only party who
    /// knows whether the work is still running.
    StillHeld
    {
        /// The item.
        item: ItemId,
        /// Who holds it.
        holder: String,
        /// When their lease lapses.
        until: Timestamp,
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
    /// Something this item depends on was declined, and will never be done.
    ///
    /// Distinct from [`Self::DependencyUnmet`], whose retryability is the whole point
    /// of that arm: finishing the dependency resolves it, so waiting is the correct advice. A
    /// declined dependency finishes nothing, so the identical advice is a queue with no head —
    /// `OD-LEDGER-020` is the record of the live case this arm exists for, `P10-EDGE-CONSTRAINTS`
    /// depending on the declined `P10-REQUEST-LAYOUT` and reading `waiting` regardless.
    ///
    /// Not retryable: no lease lapsing and no amount of time turns a declined item into a
    /// finished one. The only remedy is the dependent's own — `nomos work decline` then
    /// `nomos work add` against a dependency that can still finish — which is why the sentence
    /// names it rather than leaving the caller to work out that `DependencyUnmet`'s advice does
    /// not apply here.
    DependencyDeclined
    {
        /// The item that was refused.
        item: ItemId,
        /// The dependency that was declined.
        dependency: ItemId,
        /// The declined dependency's state, bounded — see [`crate::ItemState::Describe`].
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
    /// failure used to be reported as [`Self::NoSuchItem`], so an unreadable file,
    /// a parse error and an invalid document all told the operator their identifier was
    /// wrong — sending them to check a spelling while the ledger was broken. `OD-LEDGER-009`
    /// records it as the third instance of a reason not surviving the failure it explains.
    LedgerUnusable
    {
        /// What the store said, verbatim.
        cause: String,
    },
}

impl Refusal
{
    /// A one-line explanation a person or an agent can act on.
    ///
    /// # The refused item is never the grammatical subject here
    ///
    /// This is the form for a caller that has **not** already named the item it asked about,
    /// and every arm is phrased so that it composes under one that has. The distinction is
    /// not cosmetic: several arms carry the identifier of a *different* item — the blocker in
    /// [`Self::HeldBy`], the unfinished dependency in
    /// [`Self::DependencyUnmet`] — and a sentence that opens with one of those names
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
            } => message::Held_By(item, holder, *until),
            Self::UnknownIndependence { against, reason } => message::Unknown_Independence(against, reason),
            Self::LeaseTooLong { requested, maximum } => message::Lease_Too_Long(*requested, *maximum),
            Self::Lapsed {
                item,
                holder,
                since,
            } => message::Lapsed_Claim(item, holder, *since),
            Self::StillHeld {
                item,
                holder,
                until,
            } => message::Still_Held(item, holder, *until),
            Self::NotClaimable { item, state } => message::Not_Claimable(item, state),
            Self::DependencyUnmet {
                item,
                dependency,
                state,
            } => message::Dependency_Unmet(item, dependency, state),
            Self::DependencyDeclined {
                item,
                dependency,
                state,
            } => message::Dependency_Declined(item, dependency, state),
            Self::NoSuchItem { item } => message::No_Such_Item(item),
            Self::LedgerUnusable { cause } => message::Ledger_Unusable(cause),
        };
    }

    /// Whether retrying later might succeed.
    ///
    /// Distinguishes a queue from a dead end, which is what lets an agent decide
    /// between waiting and finding other work. [`Self::DependencyDeclined`] is deliberately
    /// absent from the retryable arms below — it is the dead end [`Self::DependencyUnmet`] is
    /// not, and the two must not answer this the same way.
    #[must_use]
    pub const fn Is_Retryable(&self) -> bool
    {
        return matches!(
            self,
            Self::HeldBy { .. } | Self::DependencyUnmet { .. } | Self::StillHeld { .. }
        );
    }

    /// Whether this refusal is a fact about the plan or a fact about this moment.
    ///
    /// `ARC-HARNESS-001` draws the seam this method reads off: "the scheduler decides what
    /// should run, coordination decides whether it can run now". Every arm above answers
    /// exactly one of those two questions, never both, which is what makes this an exhaustive
    /// match rather than a heuristic — a new variant that answers neither is a compile error
    /// here before it is anybody's confusion later.
    ///
    /// `OD-LEDGER-022` is the record this method exists to let a caller act on: a scheduler
    /// deciding what work exists must not attempt a claim merely to learn whether an item is
    /// the *kind* of thing coordination could ever grant, and [`Self`] is otherwise
    /// the only place that answer lives — in a string, in one CLI's match arm.
    #[must_use]
    pub const fn Layer(&self) -> Layer
    {
        return match self
        {
            // The plan's own answer. None of these three change because a lease lapsed, a
            // lock cleared, or a different holder asked: the item itself is not a live
            // dependency, is not the kind of thing that can be claimed right now, or is not on
            // the board at all, and every one of those stays true regardless of who is looking
            // or when they look.
            Self::NotClaimable { .. }
            | Self::DependencyUnmet { .. }
            | Self::DependencyDeclined { .. }
            | Self::NoSuchItem { .. } => Layer::Readiness,
            // Coordination's own answer. Each of these is a fact about *this* attempt: a
            // holder whose lease has not run out yet, a lease request coordination's own
            // ceiling refuses, an overlap coordination cannot currently prove disjoint, a
            // holder who is dead rather than working, an item whose live claim an operation
            // would end, or the store coordination itself needs and cannot use right now.
            // None of it is a statement about the plan — a later attempt, same plan, can find
            // every one of these already resolved.
            Self::HeldBy { .. }
            | Self::LeaseTooLong { .. }
            | Self::UnknownIndependence { .. }
            | Self::Lapsed { .. }
            | Self::StillHeld { .. }
            | Self::LedgerUnusable { .. } => Layer::Dispatch,
        };
    }

    /// Whether the plan itself says this work is not ready, independent of who is asking.
    ///
    /// The narrower of the two questions [`Self::Layer`] answers, spelled as a predicate for
    /// a caller that only needs one side of it — the shape `Applicability::Is_Agent_Required`
    /// (`OD-CONTRACTS-002`) already used for exactly this: a predicate beside the
    /// classification it is derived from, not a second decision.
    #[must_use]
    pub const fn Is_Readiness(&self) -> bool
    {
        return matches!(self.Layer(), Layer::Readiness);
    }

    /// Whether coordination is the thing standing between a caller and this work right now.
    ///
    /// See [`Self::Is_Readiness`]: the same derivation, the other side of it.
    #[must_use]
    pub const fn Is_Dispatch(&self) -> bool
    {
        return matches!(self.Layer(), Layer::Dispatch);
    }
}

#[cfg(test)]
#[path = "refusal/tests.rs"]
mod tests;

/// Narrow, file-local proofs for each of this file's own methods, addressed by name.
///
/// [`tests`] above is `refusal/tests.rs`, a separate physical file whose behavioural suite
/// this does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's
/// companion unit off the literal file it is textually written in, so a test living in that
/// separate file can never address a function declared here, however it is named — this
/// module gives each method here the one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Name_The_State_A_Not_Claimable_Item_Is_In()
    {
        let refusal = Refusal::NotClaimable {
            item: ItemId::New("T-1"),
            state: "Done".to_owned(),
        };

        let said = refusal.Describe();
        assert!(said.contains("T-1"), "{said}");
        assert!(said.contains("Done"), "{said}");
    }

    #[test]
    fn Test_Is_Retryable_Should_Be_True_For_A_Held_Item_And_False_For_A_Declined_Dependency()
    {
        let held_by = Refusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: Timestamp::From_Unix_Seconds(2_000),
            item: ItemId::New("T-1"),
        };
        let dependency_declined = Refusal::DependencyDeclined {
            item: ItemId::New("T-2"),
            dependency: ItemId::New("T-3"),
            state: "declined".to_owned(),
        };

        assert!(held_by.Is_Retryable());
        assert!(!dependency_declined.Is_Retryable());
    }

    #[test]
    fn Test_Layer_Should_Put_A_Held_Item_In_Dispatch_And_A_Not_Claimable_One_In_Readiness()
    {
        let held_by = Refusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: Timestamp::From_Unix_Seconds(2_000),
            item: ItemId::New("T-1"),
        };
        let not_claimable = Refusal::NotClaimable {
            item: ItemId::New("T-2"),
            state: "Done".to_owned(),
        };

        assert_eq!(held_by.Layer(), Layer::Dispatch);
        assert_eq!(not_claimable.Layer(), Layer::Readiness);
    }

    #[test]
    fn Test_Is_Readiness_Should_Agree_With_Layer()
    {
        let not_claimable = Refusal::NotClaimable {
            item: ItemId::New("T-1"),
            state: "Done".to_owned(),
        };

        assert!(not_claimable.Is_Readiness());
        assert_eq!(
            not_claimable.Is_Readiness(),
            matches!(not_claimable.Layer(), Layer::Readiness)
        );
    }

    #[test]
    fn Test_Is_Dispatch_Should_Agree_With_Layer()
    {
        let held_by = Refusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: Timestamp::From_Unix_Seconds(2_000),
            item: ItemId::New("T-1"),
        };

        assert!(held_by.Is_Dispatch());
        assert_eq!(held_by.Is_Dispatch(), matches!(held_by.Layer(), Layer::Dispatch));
    }
}
