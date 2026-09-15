use super::*;
use nomos_model::UnknownReason;
use std::time::Duration;

/// The instant every fixture below works at, named so it is a decision rather than a value
/// repeated wherever a lease wanted one: a live lease's `until` and a lapsed one's `since`
/// are the same clock reading seen from either side.
const NOW_SECONDS: i64 = 2_000;

/// How many variants `Refusal` declares — what `All()` must produce, one of each.
const DECLARED_REFUSAL_COUNT: usize = 10;

/// The name of every variant `Refusal` declares, sorted.
///
/// Sorted because the assertion that reads it sorts what it saw: the two are compared name
/// for name rather than in whatever order `All()` happens to build them in.
const DECLARED_VARIANTS: [&str; DECLARED_REFUSAL_COUNT] = [
    "DependencyDeclined",
    "DependencyUnmet",
    "HeldBy",
    "Lapsed",
    "LeaseTooLong",
    "LedgerUnusable",
    "NoSuchItem",
    "NotClaimable",
    "StillHeld",
    "UnknownIndependence",
];

/// How many refusals coordination answers, and so the size of `Dispatch_Universe`.
const DISPATCH_REFUSAL_COUNT: usize = 6;

/// How many refusals the plan answers, and so the size of `Readiness_Universe`.
const READINESS_REFUSAL_COUNT: usize = 4;

/// A lease request far past any real ceiling, for the refusal case.
const ABSURDLY_LONG_LEASE_SECONDS: u64 = 9_999_999;

/// One hour, in seconds — the ceiling the absurd request above is measured against.
const ONE_HOUR_SECONDS: u64 = 3_600;

fn At(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

/// One of every variant, built once so a test that forgets one is impossible rather than
/// merely unlikely — the shape `Applicability`'s `ALL` array uses for the same reason.
///
/// Assembled from the two layer universes below rather than written flat: the split is the
/// same one `Test_Readiness_Refusals_Should_Be_Exactly_The_Plan_Facts` and
/// `Test_Dispatch_Refusals_Should_Be_Exactly_The_Coordination_Facts` already draw, so this
/// is the natural seam rather than an arbitrary halving.
fn All() -> [Refusal; DECLARED_REFUSAL_COUNT]
{
    let mut all = Dispatch_Universe().to_vec();
    all.extend(Readiness_Universe());

    return all.try_into().unwrap_or_else(|found: Vec<Refusal>| {
        panic!("All() must produce exactly 10 refusals, found {}", found.len());
    });
}

/// The six refusals coordination answers: a holder, a lease, a lock, an unprovable
/// overlap, an unusable store.
fn Dispatch_Universe() -> [Refusal; DISPATCH_REFUSAL_COUNT]
{
    return [
        Refusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: At(NOW_SECONDS),
            item: ItemId::New("T-1"),
        },
        Refusal::UnknownIndependence {
            against: ItemId::New("T-2"),
            reason: UnknownReason::IncomparableSnapshots,
        },
        Refusal::LeaseTooLong {
            requested: Duration::from_secs(ABSURDLY_LONG_LEASE_SECONDS),
            maximum: Duration::from_secs(ONE_HOUR_SECONDS),
        },
        Refusal::Lapsed {
            item: ItemId::New("T-3"),
            holder: "dead-agent".to_owned(),
            since: At(NOW_SECONDS),
        },
        Refusal::StillHeld {
            item: ItemId::New("T-4"),
            holder: "agent-b".to_owned(),
            until: At(NOW_SECONDS),
        },
        Refusal::LedgerUnusable {
            cause: "disk full".to_owned(),
        },
    ];
}

/// The four refusals the plan answers: whether a dependency is unfinished or declined,
/// whether the item is claimable, whether it exists at all.
fn Readiness_Universe() -> [Refusal; READINESS_REFUSAL_COUNT]
{
    return [
        Refusal::NotClaimable {
            item: ItemId::New("T-5"),
            state: "Done".to_owned(),
        },
        Refusal::DependencyUnmet {
            item: ItemId::New("T-6"),
            dependency: ItemId::New("T-7"),
            state: "Ready".to_owned(),
        },
        Refusal::DependencyDeclined {
            item: ItemId::New("T-8"),
            dependency: ItemId::New("T-9"),
            state: "declined".to_owned(),
        },
        Refusal::NoSuchItem {
            item: ItemId::New("T-10"),
        },
    ];
}

/// The plan's own answers: whether a dependency is unfinished or declined, whether the
/// item is claimable, whether it exists at all. None of these mention a holder, a lease
/// or a lock, which is the property that makes them plan facts rather than moment facts.
#[test]
fn Test_Readiness_Refusals_Should_Be_Exactly_The_Plan_Facts()
{
    for refusal in Readiness_Universe()
    {
        assert_eq!(refusal.Layer(), Layer::Readiness, "{refusal:?}");
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
    for refusal in Dispatch_Universe()
    {
        assert_eq!(refusal.Layer(), Layer::Dispatch, "{refusal:?}");
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
///
/// The exhaustive match alone only proves that every refusal `All()` happens to produce is
/// *some* declared variant — it says nothing about which ones. `Dispatch_Universe` or
/// `Readiness_Universe` could be edited into two entries of one variant and none of another,
/// and ten refusals would still satisfy every arm here. So each arm answers with its own
/// variant's name, and the assertion below is the other half: the names `All()` produced are
/// exactly the declared ones, so every variant is present and none has crowded another out.
#[test]
fn Test_Every_Variant_Should_Be_In_The_Tested_Universe()
{
    let mut seen = Names_Seen_In_All();
    seen.sort_unstable();

    assert_eq!(
        seen,
        DECLARED_VARIANTS,
        "All() must contain one of every declared Refusal variant"
    );
}

/// The name of the variant each refusal `All()` produces belongs to, in the order it
/// produces them.
fn Names_Seen_In_All() -> Vec<&'static str>
{
    return All().iter().map(Name_Of).collect();
}

/// The name of the variant `refusal` is.
///
/// The match has no wildcard, so an eleventh variant arriving stops the build here, beside
/// the list it has to be added to. An arm can only be wrong by naming a variant the declared
/// list does not carry, which the assertion above rejects — there is no separate flag for it
/// to have been recorded in the wrong one of.
fn Name_Of(refusal: &Refusal) -> &'static str
{
    return match refusal
    {
        Refusal::HeldBy { .. } => "HeldBy",
        Refusal::UnknownIndependence { .. } => "UnknownIndependence",
        Refusal::LeaseTooLong { .. } => "LeaseTooLong",
        Refusal::Lapsed { .. } => "Lapsed",
        Refusal::StillHeld { .. } => "StillHeld",
        Refusal::NotClaimable { .. } => "NotClaimable",
        Refusal::DependencyUnmet { .. } => "DependencyUnmet",
        Refusal::DependencyDeclined { .. } => "DependencyDeclined",
        Refusal::NoSuchItem { .. } => "NoSuchItem",
        Refusal::LedgerUnusable { .. } => "LedgerUnusable",
    };
}

/// A refusal that is retryable but not a plan fact — the case `Is_Retryable` alone cannot
/// tell apart from `DependencyUnmet`, which is retryable and *is* a plan fact. Layer and
/// retryability are different axes, and this is the pair that proves it: both `true` on
/// one and `true`/`true` on the other for different reasons.
#[test]
fn Test_Layer_And_Retryability_Are_Independent_Axes()
{
    let held_by = Refusal::HeldBy {
        holder: "agent-a".to_owned(),
        until: At(NOW_SECONDS),
        item: ItemId::New("T-1"),
    };
    let dependency_unmet = Refusal::DependencyUnmet {
        item: ItemId::New("T-2"),
        dependency: ItemId::New("T-3"),
        state: "Ready".to_owned(),
    };

    assert!(held_by.Is_Retryable() && held_by.Is_Dispatch());
    assert!(dependency_unmet.Is_Retryable() && dependency_unmet.Is_Readiness());
}
