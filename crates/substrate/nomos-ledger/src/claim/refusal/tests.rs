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
