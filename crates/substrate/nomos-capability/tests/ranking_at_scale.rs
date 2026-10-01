//! The ranking still answers, through the whole registry, at a size nobody wrote it against.
//!
//! `resolution::selection`'s own suite holds the ranking against a scan per offer over
//! pseudo-random populations, and times the two side by side. This asks the narrower
//! question that suite cannot: that a caller going through [`Registry::Resolve`] — the floor,
//! the clone into a usable set, the selection, and what the selection reports about what it
//! passed over — gets the same answer over four thousand providers that it gets over two.

use nomos_capability::{
    CapabilityContract, ProviderOffer, Registry, Requirement, Resolution, Selection,
};
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    ProviderId,
};

const V1: ContractVersion = ContractVersion::New(1, 0);

/// The provider counts resolved. The largest is three orders of magnitude past anything this
/// workspace composes, which is the point: the ranking's cost is the reason it was rewritten
/// and its answer is what the rewrite had to keep.
const POPULATIONS: [usize; 3] = [2, 250, 4_000];

fn Capability() -> CapabilityId
{
    return CapabilityId::New("nomos.cap.test.ranking_at_scale");
}

/// The strongest thing any offer here claims, so no offer is refused for exceeding it.
fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::RuntimeObserved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Region,
    );
}

/// The weakest thing the requirement accepts, so the floor removes nobody and what is being
/// exercised is the ranking rather than the filter in front of it.
fn Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Predicted,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::None,
    );
}

/// A parse: sound, syntactic, one file at a time. The only guarantee here that is stronger
/// than another, which is what makes the assertion about maximality mean something.
fn Parse() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// A line-reader: strictly weaker than [`Parse`] on two axes and equal on the third.
fn Scan() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// Incomparable to both of the above: stronger on variant, weaker on soundness and on the
/// granularity it can refresh at. Without it these populations would be chains, and a chain
/// exercises nothing a sort could not have done.
fn Coarse_Semantic() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// The guarantees the population is drawn from, cycled. A registry of four thousand
/// providers does not hold four thousand distinct guarantees, and a ranking measured as
/// though it did would be measured on the wrong shape.
fn Guarantee_Pool() -> [Guarantee; 3]
{
    return [Parse(), Scan(), Coarse_Semantic()];
}

fn Offer_Of(provider: usize, guarantee: Guarantee) -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(format!("nomos.provider.test.{provider:06}")),
        capability: Capability(),
        version: V1,
        guarantee,
    };
}

/// A registry declaring one capability and holding `providers` offers against it.
fn Registry_Of(providers: usize) -> Registry
{
    let mut registry = Registry::New();
    registry
        .Declare(CapabilityContract {
            id: Capability(),
            version: V1,
            ceiling: Ceiling(),
            summary: "the capability whose offers this file resolves at scale".to_owned(),
        })
        .expect("a fresh registry has declared nothing");

    let pool = Guarantee_Pool();
    for provider in 0..providers
    {
        // `checked_rem` rather than `%`: this workspace denies arithmetic that can panic
        // even in a test, and a pool that was somehow empty would divide by zero rather
        // than cycle.
        let at = provider.checked_rem(pool.len()).expect("the pool is a non-empty array");
        let guarantee = *pool.get(at).expect("the index is taken modulo the pool's own length");
        registry.Offer(Offer_Of(provider, guarantee)).expect("every name here is distinct");
    }

    return registry;
}

/// The selection `registry` produces, or a panic naming what it produced instead.
fn Selection_Of(registry: &Registry) -> Selection
{
    return match registry.Resolve(&Requirement::New(Capability(), V1, Floor()))
    {
        Resolution::Satisfied { selection, .. } => selection,
        Resolution::Unsatisfied { reason, .. } => {
            panic!("every offer clears this floor, so nothing should be unsatisfied: {reason:?}")
        },
    };
}

#[test]
fn Test_Resolving_Many_Providers_Should_Still_Choose_A_Maximal_Offer()
{
    for providers in POPULATIONS
    {
        let selection = Selection_Of(&Registry_Of(providers));

        assert!(
            !selection.Has_Passed_Over_Stronger(),
            "over {providers} offers the registry chose one that something usable was \
             strictly stronger than, with no preference to excuse it"
        );
        assert_eq!(
            selection.chosen.guarantee,
            Parse(),
            "over {providers} offers the parse is the only offer nothing is stronger than \
             and that something is weaker than, so it is what must answer"
        );
    }
}

#[test]
fn Test_Resolving_Many_Providers_Should_Account_For_Every_Offer()
{
    for providers in POPULATIONS
    {
        let selection = Selection_Of(&Registry_Of(providers));

        assert_eq!(
            selection.alternatives.len(),
            providers.saturating_sub(1),
            "a ranking over {providers} offers dropped one"
        );
        assert_eq!(
            selection.Weaker().len().saturating_add(selection.Unranked().len()),
            providers.saturating_sub(1),
            "every alternative is either weaker than the parse or one the guarantee could \
             not rank against it, and over {providers} offers some alternative was neither"
        );
    }
}
