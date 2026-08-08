//! The registry's answer is never a bare no.
//!
//! A capability nobody offers is coverage debt, not a statement that the analysis does
//! not apply. Collapsing those is the same defect as a validator that never ran: the run
//! reports clean because nothing contradicted it.

use nomos_capability::{
    CapabilityContract, ProviderOffer, Registry, RegistryError, Requirement, Resolution, Unmet,
};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee,
    IncrementalGranularity, ProviderId,
};

const V1: ContractVersion = ContractVersion::New(1, 0);

fn Capability() -> CapabilityId
{
    return CapabilityId::New("nomos.cap.rust.syntax");
}

fn Guarantee_At(variant: FactVariant) -> Guarantee
{
    return Guarantee::New(
        variant,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

fn Contract(ceiling: FactVariant) -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: V1,
        summary: "syntax facts about a Rust file".to_owned(),
        ceiling: Guarantee_At(ceiling),
    };
}

fn Offer(provider: &str, variant: FactVariant) -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(provider),
        capability: Capability(),
        version: V1,
        guarantee: Guarantee_At(variant),
    };
}

fn Needing(variant: FactVariant) -> Requirement
{
    return Requirement::New(Capability(), V1, Guarantee_At(variant));
}

/// The reason this crate exists.
#[test]
fn Test_An_Undeclared_Capability_Should_Be_Coverage_Debt_And_Not_Not_Applicable()
{
    let registry = Registry::New();

    let resolution = registry.Resolve(&Needing(FactVariant::Syntactic));

    assert_eq!(resolution.Applicability(), Applicability::MissingCapability);
    assert_ne!(
        resolution.Applicability(),
        Applicability::NotApplicable,
        "the registry decided a rule does not bind a subject, which is not its call"
    );
    assert!(resolution.Applicability().Is_Coverage_Debt());
    assert!(!resolution.Applicability().Was_Evaluated());
}

/// Exhaustive over the diagnosis vocabulary, so a variant added later cannot quietly
/// map to a judgment.
#[test]
fn Test_No_Unmet_Reason_Should_Ever_Map_To_A_Judgment()
{
    let reasons = [
        Unmet::Undeclared,
        Unmet::NoProvider,
        Unmet::VersionMismatch { offered: V1 },
        Unmet::BelowRequirement {
            closest: ProviderId::New("syn"),
        },
    ];

    for reason in &reasons
    {
        let applicability = reason.Applicability();

        assert!(
            applicability.Is_Coverage_Debt(),
            "{} reports as {applicability}, which is not coverage debt",
            reason.Describe()
        );
        assert_ne!(applicability, Applicability::NotApplicable);
        assert_ne!(applicability, Applicability::ConfigurationDisabled);
        assert!(!reason.Describe().is_empty());
    }
}

/// Declared and offered by nobody is a different remedy from declared by nobody, and the
/// detail says which.
#[test]
fn Test_A_Declared_Capability_With_No_Provider_Should_Say_So()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::SemanticallyResolved)).expect("declares");

    let resolution = registry.Resolve(&Needing(FactVariant::Syntactic));

    assert!(
        matches!(
            resolution,
            Resolution::Unsatisfied {
                reason: Unmet::NoProvider,
                ..
            }
        ),
        "{resolution:?}"
    );
}

/// The negative control. Without it every assertion above passes on a registry that can
/// never satisfy anything.
#[test]
fn Test_A_Provider_That_Meets_The_Requirement_Should_Satisfy_It()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::SemanticallyResolved)).expect("declares");
    registry.Offer(Offer("syn", FactVariant::Syntactic)).expect("offers");

    let resolution = registry.Resolve(&Needing(FactVariant::Syntactic));

    assert_eq!(resolution.Applicability(), Applicability::Supported);
    assert!(resolution.Applicability().Was_Evaluated());
    assert_eq!(
        resolution.Offer().map(|offer| offer.provider.clone()),
        Some(ProviderId::New("syn"))
    );
}

/// A weaker provider is unusable, not merely weaker. `minimum` is a floor.
#[test]
fn Test_A_Provider_Below_The_Requirement_Should_Not_Satisfy_It()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::SemanticallyResolved)).expect("declares");
    registry.Offer(Offer("syn", FactVariant::Syntactic)).expect("offers");

    let resolution = registry.Resolve(&Needing(FactVariant::SemanticallyResolved));

    assert!(
        matches!(
            resolution,
            Resolution::Unsatisfied {
                reason: Unmet::BelowRequirement { .. },
                ..
            }
        ),
        "a syntactic provider satisfied a requirement for name resolution: {resolution:?}"
    );
    assert!(resolution.Offer().is_none(), "an unsatisfied resolution named a provider");
}

/// A named preference that was honoured is `Supported`; one that was not is a fallback.
/// The judgment stands either way — what changes is whether the caller can tell.
#[test]
fn Test_An_Unhonoured_Preference_Should_Read_As_Fallback()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::SemanticallyResolved)).expect("declares");
    registry.Offer(Offer("syn", FactVariant::Syntactic)).expect("offers");

    let honoured = registry
        .Resolve(&Needing(FactVariant::Syntactic).Preferring(ProviderId::New("syn")));
    let unhonoured = registry
        .Resolve(&Needing(FactVariant::Syntactic).Preferring(ProviderId::New("rust-analyzer")));

    assert_eq!(honoured.Applicability(), Applicability::Supported);
    assert_eq!(unhonoured.Applicability(), Applicability::SupportedWithFallback);
    assert!(
        unhonoured.Applicability().Was_Evaluated(),
        "a fallback still produced an answer"
    );
    assert_eq!(
        unhonoured.Offer().map(|offer| offer.provider.clone()),
        Some(ProviderId::New("syn")),
        "the fallback must name who actually answered"
    );
}

/// A provider grading its own work. A syntactic parser claiming resolution would make
/// every rule that needs resolution silently accept an answer that cannot support it.
#[test]
fn Test_An_Offer_Above_The_Contract_Ceiling_Should_Be_Refused()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::Syntactic)).expect("declares");

    let refusal = registry
        .Offer(Offer("optimistic", FactVariant::SemanticallyResolved))
        .expect_err("a claim above the ceiling must be refused");

    assert!(matches!(refusal, RegistryError::ExceedsCeiling { .. }), "{refusal}");
}

#[test]
fn Test_An_Offer_Against_No_Contract_Should_Be_Refused()
{
    let mut registry = Registry::New();

    let refusal = registry
        .Offer(Offer("syn", FactVariant::Syntactic))
        .expect_err("an offer against nothing must be refused");

    assert!(matches!(refusal, RegistryError::OfferForUndeclared { .. }), "{refusal}");
}

#[test]
fn Test_Two_Contracts_For_One_Capability_Should_Be_Refused()
{
    let mut registry = Registry::New();
    registry.Declare(Contract(FactVariant::Syntactic)).expect("declares");

    let refusal = registry
        .Declare(Contract(FactVariant::SemanticallyResolved))
        .expect_err("one name, one meaning");

    assert!(matches!(refusal, RegistryError::AlreadyDeclared { .. }), "{refusal}");
}

/// A caller that cannot read the contract version must be told, not quietly served.
#[test]
fn Test_An_Unreadable_Contract_Version_Should_Be_Refused()
{
    let mut registry = Registry::New();
    let mut newer = Contract(FactVariant::Syntactic);
    newer.version = ContractVersion::New(2, 0);
    registry.Declare(newer).expect("declares");

    let resolution = registry.Resolve(&Needing(FactVariant::Syntactic));

    assert!(
        matches!(
            resolution,
            Resolution::Unsatisfied {
                reason: Unmet::VersionMismatch { .. },
                ..
            }
        ),
        "{resolution:?}"
    );
}

/// Which provider answers must not depend on the order they registered in. A run whose
/// provenance changes with registration order is a run nobody can reproduce.
#[test]
fn Test_Provider_Selection_Should_Not_Depend_On_Registration_Order()
{
    let build = |first: &str, second: &str| {
        let mut registry = Registry::New();
        registry.Declare(Contract(FactVariant::SemanticallyResolved)).expect("declares");
        registry.Offer(Offer(first, FactVariant::Syntactic)).expect("offers");
        registry.Offer(Offer(second, FactVariant::Syntactic)).expect("offers");
        return registry
            .Resolve(&Needing(FactVariant::Syntactic))
            .Offer()
            .map(|offer| offer.provider.clone());
    };

    let forward = build("syn", "tree-sitter");
    let backward = build("tree-sitter", "syn");

    assert!(forward.is_some(), "neither order resolved, so this proved nothing");
    assert_eq!(forward, backward);
}
