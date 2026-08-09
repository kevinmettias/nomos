//! Which of several usable offers answers is decided, and the decision is the guarantee's.
//!
//! The floor decides what is *usable* and these decide what is *chosen*. Until there were
//! two providers the second question never arose; when it did, it was being answered by
//! whichever provider's name sorted first, which is not an answer — it produces a
//! defensible outcome only by coincidence and changes when somebody renames a crate.
//!
//! `docs/records/OD-CAPABILITY-001` records what decided it.

use nomos_capability::{
    CapabilityContract, ProviderOffer, Registry, Requirement, Resolution, Selection, Standing,
};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee,
    IncrementalGranularity, ProviderId,
};

const V1: ContractVersion = ContractVersion::New(1, 0);

fn Capability() -> CapabilityId
{
    return CapabilityId::New("nomos.cap.test.items");
}

/// A parse: exact within what it can see, one file at a time.
fn Strong() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// A line-reader: strictly weaker than the parse on two axes and equal on the third.
fn Weak() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// A compiler-backed provider, and neither stronger nor weaker than the parse: it resolves
/// names, which a parse cannot, and can only refresh a whole project, which a parse can
/// beat. There is no answer to which of these two is better, and that is a fact about the
/// two providers rather than a gap in the registry.
fn Sideways() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// High enough to admit every offer in this file. The ceiling is not what is under test
/// here — it bounds what may be claimed, and these tests are about choosing between claims
/// that are all permitted.
fn Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: V1,
        summary: "syntax facts about a source file".to_owned(),
        ceiling: Guarantee::New(
            FactVariant::RuntimeObserved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        ),
    };
}

fn Offer(provider: &str, guarantee: Guarantee) -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(provider),
        capability: Capability(),
        version: V1,
        guarantee,
    };
}

/// Admits everything in this file: the weakest variant, no property demanded, and no
/// incremental capability required.
fn Admitting_Everything() -> Requirement
{
    return Requirement::New(
        Capability(),
        V1,
        Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unknown,
            Assurance::Unknown,
            IncrementalGranularity::None,
        ),
    );
}

fn Registered(offers: &[ProviderOffer]) -> Registry
{
    let mut registry = Registry::New();
    registry.Declare(Contract()).expect("one contract, declared once");

    for offer in offers
    {
        registry.Offer(offer.clone()).expect("every offer here is within the ceiling");
    }

    return registry;
}

fn Selected(registry: &Registry, requirement: &Requirement) -> Selection
{
    let resolution = registry.Resolve(requirement);

    let Resolution::Satisfied { selection, .. } = resolution
    else
    {
        panic!("nothing answered a requirement every offer in this file clears: {resolution:?}")
    };

    return selection;
}

/// The rule.
///
/// The names are chosen so that name order points *against* the answer. With `parse` and
/// `scan` this test passes under the rule it replaces, which makes it no test at all — the
/// composition it was written for has exactly this shape, `nomos.lang.rust.scan` sorting
/// ahead of `nomos.lang.rust.syn`.
#[test]
fn Test_The_Strongest_Usable_Offer_Should_Answer()
{
    let registry = Registered(&[Offer("a.scan", Weak()), Offer("z.parse", Strong())]);

    let selection = Selected(&registry, &Admitting_Everything());

    assert_eq!(selection.chosen.provider, ProviderId::New("z.parse"));
    assert!(!selection.Passed_Over_Stronger());
    assert!(selection.Unranked().is_empty(), "the guarantee ranked these two");
}

/// The done-when, stated directly: the same two guarantees, with the names swapped so that
/// name order points the other way, must produce the same answer.
///
/// Under the rule this replaces, this test is red — the first registry resolves to the
/// line-reader and the second to the parse, because in each case the winner is whichever
/// one is called `a.…`.
#[test]
fn Test_Renaming_A_Provider_Should_Not_Change_Which_Offer_Answers()
{
    let one = Registered(&[Offer("a.scan", Weak()), Offer("z.parse", Strong())]);
    let other = Registered(&[Offer("a.parse", Strong()), Offer("z.scan", Weak())]);

    let chosen = Selected(&one, &Admitting_Everything()).chosen;
    let renamed = Selected(&other, &Admitting_Everything()).chosen;

    assert_eq!(
        chosen.guarantee, renamed.guarantee,
        "{} answered in one and {} in the other, so what a provider is called decided it",
        chosen.provider, renamed.provider
    );
    assert_eq!(chosen.guarantee, Strong());
    assert_ne!(
        chosen.provider, renamed.provider,
        "the names really were swapped, or this test compares a registry with itself"
    );
}

/// Nothing usable is dropped on the way out.
///
/// The property that makes a lowered floor mean something. A caller widens what it will
/// accept in order to reach a provider that can answer where the strong one cannot; a
/// resolution that named only the winner would have spent that floor on the caller's behalf
/// and handed back one provider.
#[test]
fn Test_Every_Usable_Offer_Should_Be_Reachable_From_The_Selection()
{
    let registry = Registered(&[
        Offer("scan", Weak()),
        Offer("parse", Strong()),
        Offer("compiler", Sideways()),
    ]);

    let selection = Selected(&registry, &Admitting_Everything());

    let mut named: Vec<String> = selection
        .alternatives
        .iter()
        .chain(std::iter::once(&selection.chosen))
        .map(|offer| return offer.provider.As_Str().to_owned())
        .collect();
    named.sort();

    assert_eq!(named, vec!["compiler", "parse", "scan"]);
}

/// And nothing unusable is smuggled in with them.
///
/// Without this the assertion above would be satisfied by a selection that simply returns
/// every registered offer, which would put an offer below the caller's floor in front of a
/// caller that stated the floor.
#[test]
fn Test_An_Offer_Below_The_Floor_Should_Not_Appear_As_An_Alternative()
{
    let registry = Registered(&[Offer("scan", Weak()), Offer("parse", Strong())]);

    let needs_a_parse = Requirement::New(Capability(), V1, Strong());
    let selection = Selected(&registry, &needs_a_parse);

    assert_eq!(selection.chosen.provider, ProviderId::New("parse"));
    assert!(
        selection.alternatives.is_empty(),
        "the line-reader does not clear this floor and must not be offered as a fallback \
         from it: {:?}",
        selection.alternatives
    );
}

/// A preference outranks the rule, and what it was honoured over stays visible.
///
/// The registry does not overrule a caller that says what it wants — a preference is the
/// only cost knowledge in the system, since a `Guarantee` says how good an answer is and
/// nothing about what it costs. But a caller has to be able to record what its preference
/// passed over, and `Applicability::Supported` alone cannot say that.
#[test]
fn Test_An_Honoured_Preference_Should_Report_What_It_Passed_Over()
{
    let registry = Registered(&[Offer("scan", Weak()), Offer("parse", Strong())]);

    let resolution =
        registry.Resolve(&Admitting_Everything().Preferring(ProviderId::New("scan")));
    let selection = resolution.Selection().expect("the preference is usable");

    assert_eq!(selection.chosen.provider, ProviderId::New("scan"));
    assert_eq!(
        resolution.Applicability(),
        Applicability::Supported,
        "the caller asked for this one and got it"
    );
    assert!(
        selection.Passed_Over_Stronger(),
        "and the parse was available and stronger, which is what the preference cost"
    );

    // The control. Without a preference the rule chooses, and then nothing stronger can
    // have been passed over — if this reads true, the rule is not choosing a maximal offer.
    assert!(!Selected(&registry, &Admitting_Everything()).Passed_Over_Stronger());
}

/// Where the guarantee ranks nothing, the registry says so rather than implying it decided.
///
/// A parse and a compiler-backed provider are incomparable, and the offer that answers is
/// then the first by provider name — deterministic, and not a reason. A composition root
/// that wants its provider choice to be a decision asserts `Unranked` is empty; this one
/// cannot, and the difference is visible rather than inferred.
#[test]
fn Test_Offers_The_Guarantee_Cannot_Rank_Should_Be_Reported_As_Undecided()
{
    let registry = Registered(&[Offer("a.compiler", Sideways()), Offer("z.parse", Strong())]);

    let selection = Selected(&registry, &Admitting_Everything());

    assert_eq!(
        selection.Unranked().len(),
        1,
        "these two are incomparable and the selection must not pretend otherwise"
    );
    assert_eq!(
        selection.Standing_Of(&Offer("z.parse", Strong())),
        Standing::Incomparable
    );
    assert!(
        selection.Weaker().is_empty(),
        "neither is weaker than the other, so neither is a fallback from the other"
    );
}

/// Two providers making the same promise are equivalent, which is not the same as
/// incomparable, and neither is a decision.
#[test]
fn Test_Two_Equal_Offers_Should_Be_Equivalent_Rather_Than_Incomparable()
{
    let registry = Registered(&[Offer("one", Strong()), Offer("two", Strong())]);

    let selection = Selected(&registry, &Admitting_Everything());

    assert_eq!(selection.Unranked().len(), 1);
    assert_eq!(
        selection.Standing_Of(&Offer("two", Strong())),
        Standing::Equivalent,
        "each reaches everything the other does; nobody has to look for a tiebreak that \
         would tell them apart, because there is nothing to tell apart"
    );
    assert!(!Standing::Equivalent.Decided());
    assert!(!Standing::Incomparable.Decided());
    assert!(Standing::Stronger.Decided());
}
