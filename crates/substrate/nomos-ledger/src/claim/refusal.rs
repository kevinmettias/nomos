//! Every way a claim is refused, in the words the holder gets.

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
    /// The item itself is held, and the operation would end somebody's live work.
    ///
    /// Distinct from [`ClaimRefusal::HeldBy`], which is about a *different* item whose
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
    /// Distinct from [`ClaimRefusal::DependencyUnmet`], whose retryability is the whole point
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
            } => Held_By(item, holder, *until),
            Self::UnknownIndependence { against, reason } => Unknown_Independence(against, reason),
            Self::LeaseTooLong { requested, maximum } => Lease_Too_Long(*requested, *maximum),
            Self::Lapsed {
                item,
                holder,
                since,
            } => Lapsed(item, holder, *since),
            Self::StillHeld {
                item,
                holder,
                until,
            } => Still_Held(item, holder, *until),
            Self::NotClaimable { item, state } => Not_Claimable(item, state),
            Self::DependencyUnmet {
                item,
                dependency,
                state,
            } => Dependency_Unmet(item, dependency, state),
            Self::DependencyDeclined {
                item,
                dependency,
                state,
            } => Dependency_Declined(item, dependency, state),
            Self::NoSuchItem { item } => No_Such_Item(item),
            Self::LedgerUnusable { cause } => Ledger_Unusable(cause),
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
    /// the *kind* of thing coordination could ever grant, and [`ClaimRefusal`] is otherwise
    /// the only place that answer lives — in a string, in one CLI's match arm.
    #[must_use]
    pub const fn Layer(&self) -> RefusalLayer
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
            | Self::NoSuchItem { .. } => RefusalLayer::Readiness,
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
            | Self::LedgerUnusable { .. } => RefusalLayer::Dispatch,
        };
    }

    /// Whether the plan itself says this work is not ready, independent of who is asking.
    ///
    /// The narrower of the two questions [`Self::Layer`] answers, spelled as a predicate for
    /// a caller that only needs one side of it — the shape `Applicability::Requires_Agent`
    /// (`OD-CONTRACTS-002`) already used for exactly this: a predicate beside the
    /// classification it is derived from, not a second decision.
    #[must_use]
    pub const fn Is_Readiness(&self) -> bool
    {
        return matches!(self.Layer(), RefusalLayer::Readiness);
    }

    /// Whether coordination is the thing standing between a caller and this work right now.
    ///
    /// See [`Self::Is_Readiness`]: the same derivation, the other side of it.
    #[must_use]
    pub const fn Is_Dispatch(&self) -> bool
    {
        return matches!(self.Layer(), RefusalLayer::Dispatch);
    }
}

/// Which of two owners a [`ClaimRefusal`] is a fact about.
///
/// `ARC-HARNESS-001`'s sentence names the two owners this splits: "the scheduler decides what
/// should run, coordination decides whether it can run now". A refusal answers one of those
/// questions, and this type is what lets a caller above the ledger ask *which* one it answered
/// without reading [`ClaimRefusal::Describe`]'s prose or reconstructing the CLI's own match
/// arm — `OD-LEDGER-022` is the record of why that reconstruction was the defect.
///
/// The one-way rule this exists to make askable: coordination may withhold work the plan
/// calls ready, and coordination alone can never make ready what the plan calls not ready.
/// Nothing in this type enforces the rule — [`ClaimRefusal::Layer`] only classifies what a
/// refusal already is — but the classification is what lets the rule be tested at all, over
/// the whole set of refusals rather than the two variants a caller happens to remember.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalLayer
{
    /// A fact about the plan: whether this item can be worked at all, by anybody, regardless
    /// of when they ask. Stays true until the plan itself changes — a dependency finishes, an
    /// item's state changes, an item is authored.
    Readiness,
    /// A fact about this moment: the plan says the work is ready, and coordination — a lease,
    /// a lock, an unprovable independence, a store it could not reach — stands between a
    /// caller and starting it right now. A later attempt, against the same plan, can find it
    /// already resolved.
    Dispatch,
}

/// Territory another live claim already covers.
fn Held_By(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "territory overlaps {item}, held by {holder} until unix {}",
        until.Unix_Seconds()
    );
}

/// A dependency that will never finish, and the remedy that is not "wait".
fn Dependency_Declined(item: &ItemId, dependency: &ItemId, state: &str) -> String
{
    return format!(
        "{item} depends on {dependency}, which is {state}; a declined dependency never becomes \
         Done, so retrying will not resolve this — {item} must be closed with `nomos work \
         decline` and re-authored against a dependency that can still finish"
    );
}

/// A requested lease past the ceiling coordination allows.
fn Lease_Too_Long(requested: Duration, maximum: Duration) -> String
{
    return format!("a lease of {requested:?} exceeds the {maximum:?} ceiling");
}

/// An item whose current state refuses the operation outright.
fn Not_Claimable(item: &ItemId, state: &str) -> String
{
    return format!("{item} is {state}, so the operation was refused");
}

/// A dependency that has not finished yet, and may still.
fn Dependency_Unmet(item: &ItemId, dependency: &ItemId, state: &str) -> String
{
    return format!("{item} depends on {dependency}, which is {state}");
}

/// An identifier that matched nothing on the board.
fn No_Such_Item(item: &ItemId) -> String
{
    return format!("no item named {item}");
}

/// The ledger's own store could not be read or written.
fn Ledger_Unusable(cause: &str) -> String
{
    return format!("the ledger could not be used: {cause}");
}

/// Two territories the ledger cannot prove disjoint.
///
/// Refused rather than granted: unknown independence is not safe parallelism, and a claim
/// granted on a maybe is two sessions writing one path believing they are alone.
fn Unknown_Independence(against: &ItemId, reason: &UnknownReason) -> String
{
    return format!(
        "cannot establish independence from {against}: {}. Unknown independence is not safe \
         parallelism, so this claim is refused rather than granted",
        reason.Describe()
    );
}

/// A claim whose lease has run out, and what replaces it.
fn Lapsed(item: &ItemId, holder: &str, since: Timestamp) -> String
{
    return format!(
        "{item} was held by {holder} and the lease ran out at unix {}; `nomos work takeover` \
         replaces it and keeps {holder}'s claim on the item",
        since.Unix_Seconds()
    );
}

/// A live claim somebody else holds, and whose call it is to end it.
fn Still_Held(item: &ItemId, holder: &str, until: Timestamp) -> String
{
    return format!(
        "{item} is held by {holder} until unix {}; ending it is {holder}'s call — `nomos work \
         abandon` releases it and `nomos work decline` then ends it",
        until.Unix_Seconds()
    );
}

#[cfg(test)]
mod layer_tests
{
    use super::*;
    use nomos_model::UnknownReason;
    use std::time::Duration;

    fn At(seconds: i64) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(seconds);
    }

    /// One of every variant, built once so a test that forgets one is impossible rather than
    /// merely unlikely — the shape `Applicability`'s `ALL` array uses for the same reason.
    fn All() -> [ClaimRefusal; 10]
    {
        return [
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
                maximum: Duration::from_secs(3_600),
            },
            ClaimRefusal::Lapsed {
                item: ItemId::New("T-3"),
                holder: "dead-agent".to_owned(),
                since: At(2_000),
            },
            ClaimRefusal::StillHeld {
                item: ItemId::New("T-4"),
                holder: "agent-b".to_owned(),
                until: At(2_000),
            },
            ClaimRefusal::NotClaimable {
                item: ItemId::New("T-5"),
                state: "Done".to_owned(),
            },
            ClaimRefusal::DependencyUnmet {
                item: ItemId::New("T-6"),
                dependency: ItemId::New("T-7"),
                state: "Ready".to_owned(),
            },
            ClaimRefusal::DependencyDeclined {
                item: ItemId::New("T-8"),
                dependency: ItemId::New("T-9"),
                state: "declined".to_owned(),
            },
            ClaimRefusal::NoSuchItem {
                item: ItemId::New("T-10"),
            },
            ClaimRefusal::LedgerUnusable {
                cause: "disk full".to_owned(),
            },
        ];
    }

    /// The plan's own answers: whether a dependency is unfinished or declined, whether the
    /// item is claimable, whether it exists at all. None of these mention a holder, a lease
    /// or a lock, which is the property that makes them plan facts rather than moment facts.
    #[test]
    fn Test_Readiness_Refusals_Should_Be_Exactly_The_Plan_Facts()
    {
        let readiness = [
            ClaimRefusal::NotClaimable {
                item: ItemId::New("T-1"),
                state: "Done".to_owned(),
            },
            ClaimRefusal::DependencyUnmet {
                item: ItemId::New("T-2"),
                dependency: ItemId::New("T-3"),
                state: "Ready".to_owned(),
            },
            ClaimRefusal::DependencyDeclined {
                item: ItemId::New("T-4"),
                dependency: ItemId::New("T-5"),
                state: "declined".to_owned(),
            },
            ClaimRefusal::NoSuchItem {
                item: ItemId::New("T-6"),
            },
        ];

        for refusal in readiness
        {
            assert_eq!(refusal.Layer(), RefusalLayer::Readiness, "{refusal:?}");
            assert!(refusal.Is_Readiness(), "{refusal:?}");
            assert!(!refusal.Is_Dispatch(), "{refusal:?}");
        }
    }

    /// Coordination's own answers: a holder, a lease, a lock, an unprovable overlap, an
    /// unusable store. Every one of these can be true of an item the plan calls perfectly
    /// ready, which is exactly why they must not share a layer with the readiness facts above.
    #[test]
    fn Test_Dispatch_Refusals_Should_Be_Exactly_The_Coordination_Facts()
    {
        let dispatch = [
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
                maximum: Duration::from_secs(3_600),
            },
            ClaimRefusal::Lapsed {
                item: ItemId::New("T-3"),
                holder: "dead-agent".to_owned(),
                since: At(2_000),
            },
            ClaimRefusal::StillHeld {
                item: ItemId::New("T-4"),
                holder: "agent-b".to_owned(),
                until: At(2_000),
            },
            ClaimRefusal::LedgerUnusable {
                cause: "disk full".to_owned(),
            },
        ];

        for refusal in dispatch
        {
            assert_eq!(refusal.Layer(), RefusalLayer::Dispatch, "{refusal:?}");
            assert!(refusal.Is_Dispatch(), "{refusal:?}");
            assert!(!refusal.Is_Readiness(), "{refusal:?}");
        }
    }

    /// The one-way rule stated as a disjointness assertion over the whole set rather than
    /// over the two variants a caller happens to remember — `OD-CONTRACTS-002`'s shape for
    /// `Applicability`'s three predicates, applied to this type's two. A refusal that answered
    /// both would be a refusal a scheduler could read as a plan fact in one place and a
    /// coordination fact in another, which is the confusion this type exists to end.
    #[test]
    fn Test_No_Refusal_Should_Answer_Both_Layers()
    {
        for refusal in All()
        {
            let answers = usize::from(refusal.Is_Readiness()) + usize::from(refusal.Is_Dispatch());
            assert_eq!(answers, 1, "{refusal:?} answered {answers} layers");
        }
    }

    /// `ALL` is a hand-written universe. The match below has no wildcard, so an eleventh
    /// variant arriving stops the build here, beside the list it has to be added to —
    /// `OD-COMPLETENESS-001`'s reasoning applied to this enum's own coverage of itself.
    #[test]
    fn Test_Every_Variant_Should_Be_In_The_Tested_Universe()
    {
        for refusal in All()
        {
            match refusal
            {
                ClaimRefusal::HeldBy { .. }
                | ClaimRefusal::UnknownIndependence { .. }
                | ClaimRefusal::LeaseTooLong { .. }
                | ClaimRefusal::Lapsed { .. }
                | ClaimRefusal::StillHeld { .. }
                | ClaimRefusal::NotClaimable { .. }
                | ClaimRefusal::DependencyUnmet { .. }
                | ClaimRefusal::DependencyDeclined { .. }
                | ClaimRefusal::NoSuchItem { .. }
                | ClaimRefusal::LedgerUnusable { .. } =>
                {}
            }
        }
    }

    /// A refusal that is retryable but not a plan fact — the case `Is_Retryable` alone cannot
    /// tell apart from `DependencyUnmet`, which is retryable and *is* a plan fact. Layer and
    /// retryability are different axes, and this is the pair that proves it: both `true` on
    /// one and `true`/`true` on the other for different reasons.
    #[test]
    fn Test_Layer_And_Retryability_Are_Independent_Axes()
    {
        let held_by = ClaimRefusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: At(2_000),
            item: ItemId::New("T-1"),
        };
        let dependency_unmet = ClaimRefusal::DependencyUnmet {
            item: ItemId::New("T-2"),
            dependency: ItemId::New("T-3"),
            state: "Ready".to_owned(),
        };

        assert!(held_by.Is_Retryable() && held_by.Is_Dispatch());
        assert!(dependency_unmet.Is_Retryable() && dependency_unmet.Is_Readiness());
    }
}
