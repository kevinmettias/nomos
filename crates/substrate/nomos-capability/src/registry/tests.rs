use super::*;
use crate::Requirement;
use crate::Unmet;
use nomos_contracts::Applicability;
use nomos_contracts::ProviderId;
use nomos_contracts::ContractVersion;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

const CAPABILITY: &str = "nomos.cap.test.knowledge";

/// How many ways [`Unmet`] can name an absence: one arm of [`Every_Unmet_Reason`] each.
const UNMET_REASONS: usize = 4;

/// A contract version this caller cannot read, which is what makes the version-mismatch
/// arm of [`Every_Unmet_Reason`] reachable.
const UNREADABLE_CONTRACT_MAJOR: u16 = 2;

fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

fn Version() -> ContractVersion
{
    return ContractVersion::New(1, 0);
}

fn Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

fn Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: Version(),
        summary: "a capability that exists so this file can ask about its absence"
            .to_owned(),
        ceiling: Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        ),
    };
}

fn Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New("nomos.test.knowledge"),
        capability: Capability(),
        version: Version(),
        guarantee: Floor(),
    };
}

fn Need() -> Requirement
{
    return Requirement::New(Capability(), Version(), Floor());
}

/// Nothing installed and something that answers are two values, not one empty one.
///
/// The property `P11-KNOWLEDGE-ABSENT` asks for, at the only layer that can hold it. A
/// caller that asks an optional capability for a packet and receives a bare collection
/// has thrown this away — an absent capability and one that answered with nothing then
/// arrive as the same empty answer, and the run continues looking complete.
#[test]
fn Test_An_Absent_Capability_And_One_That_Answers_Should_Be_Different_Resolutions()
{
    let empty = Registry::New();

    let mut installed = Registry::New();
    installed.Declare(Contract()).expect("declared once");
    installed.Offer(Offer()).expect("within the ceiling");

    let absent = empty.Resolve(&Need());
    let answering = installed.Resolve(&Need());

    assert_ne!(absent, answering);
    assert!(
        matches!(absent, Resolution::Unsatisfied { .. }),
        "an uninstalled capability must not resolve to an answer: {absent:?}"
    );
    assert!(
        matches!(answering, Resolution::Satisfied { .. }),
        "an installed provider answers, whatever it goes on to find: {answering:?}"
    );
}

/// The two ways of being absent are also two values, because the remedies differ.
///
/// Authoring a contract and installing a provider are different actions, and a caller
/// told only "unavailable" cannot tell which one it is owed. Documented on [`Unmet`]
/// since it was written and asserted here.
#[test]
fn Test_An_Undeclared_Capability_And_An_Unoffered_One_Should_Be_Different_Absences()
{
    let undeclared = Registry::New().Resolve(&Need());
    let mut declared = Registry::New();
    declared.Declare(Contract()).expect("declared once");

    let unoffered = declared.Resolve(&Need());

    assert_ne!(undeclared, unoffered);
    assert!(matches!(undeclared, Resolution::Unsatisfied { reason: Unmet::Undeclared, .. }));
    assert!(matches!(unoffered, Resolution::Unsatisfied { reason: Unmet::NoProvider, .. }));
    assert_ne!(
        undeclared.Applicability(),
        Applicability::NotApplicable,
        "a capability nobody declared has not been judged inapplicable to anything"
    );
}

/// Today every absence reports `MissingCapability`, and for an *optional* capability
/// that is an overclaim.
///
/// Pinned rather than described, because `OD-CAPABILITY-004` argues from it. That
/// variant says no installed provider offers a capability the rule *requires*, which
/// points a reader at installing something; for a capability the harness is complete
/// without, nothing is required and nothing is owed. The record's answer is that an
/// optional knowledge seam must not be reported through a rule's applicability at all
/// rather than that this mapping should change — so this test is the measurement that
/// answer rests on, and it fails if the mapping moves underneath it.
/// Every way [`Unmet`] can name an absence, once each.
fn Every_Unmet_Reason() -> [Unmet; UNMET_REASONS]
{
    return [
        Unmet::Undeclared,
        Unmet::NoProvider,
        Unmet::VersionMismatch {
            offered: ContractVersion::New(UNREADABLE_CONTRACT_MAJOR, 0),
        },
        Unmet::BelowRequirement {
            closest: ProviderId::New("nomos.test.knowledge"),
        },
    ];
}

#[test]
fn Test_Every_Absence_Should_Report_As_A_Missing_Capability_Today()
{
    for reason in Every_Unmet_Reason()
    {
        assert_eq!(
            reason.Applicability(),
            Applicability::MissingCapability,
            "{reason:?} reports as something else, and OD-CAPABILITY-004 reasons from \
             this mapping"
        );
        assert!(
            !reason.Describe().is_empty(),
            "{reason:?} describes itself as nothing, so a caller has no report to make"
        );
    }
}

/// The same registry, the same unavailable provider, and two different resolutions —
/// not merely two applicabilities on the same `Satisfied`.
///
/// `OD-CAPABILITY-005` is what draws this line: a preference the registry cannot honour
/// is still answered, because the guarantee is still met and the caller said "rather
/// than" not "only". A requirement the registry cannot honour is refused, because the
/// caller said "only" — `Resolve` and `Resolve_Requiring` must disagree here or the
/// second strength does not exist.
/// A registry that has declared `Contract()` and can answer it via `Offer()`, alongside the
/// provider these tests ask for and never offer, and the provider that actually answers.
struct Registry_With_One_Offer
{
    registry: Registry,
    absent: ProviderId,
    answering: ProviderId,
}

/// A registry that has declared `Contract()` and can answer it via `Offer()` — but not
/// via the `absent` provider these tests ask for and never offer. Returns the registry,
/// `absent`, and who actually answers, since every assertion below needs at least one.
fn Registry_With_One_Offer() -> Registry_With_One_Offer
{
    let mut registry = Registry::New();
    registry.Declare(Contract()).expect("declared once");
    registry.Offer(Offer()).expect("within the ceiling");

    let absent = ProviderId::New("nomos.test.absent");
    let answering = Offer().provider;

    return Registry_With_One_Offer {
        registry,
        absent,
        answering,
    };
}

fn Assert_Preference_Falls_Back(via_preference: &Resolution)
{
    assert!(
        matches!(
            via_preference,
            Resolution::Satisfied {
                applicability: Applicability::SupportedWithFallback,
                ..
            }
        ),
        "an unavailable preference is still answered by somebody else: {via_preference:?}"
    );
}

fn Assert_Requirement_Refuses(via_requirement: RequiredResolution, required: ProviderId, answered: ProviderId)
{
    assert!(
        matches!(via_requirement, RequiredResolution::Unsatisfied { .. }),
        "an unavailable requirement must not be answered by somebody else: \
         {via_requirement:?}"
    );

    let RequiredResolution::Unsatisfied { reason, .. } = via_requirement
    else
    {
        unreachable!("asserted Unsatisfied above");
    };

    assert_eq!(
        reason,
        RequiredUnmet::AnsweredByOther { required, answered },
        "the refusal must name both who was required and who would have answered"
    );
}

/// The same registry, the same unavailable provider, and two different resolutions —
/// not merely two applicabilities on the same `Satisfied`.
///
/// `OD-CAPABILITY-005` is what draws this line: a preference the registry cannot honour
/// is still answered, because the guarantee is still met and the caller said "rather
/// than" not "only". A requirement the registry cannot honour is refused, because the
/// caller said "only" — `Resolve` and `Resolve_Requiring` must disagree here or the
/// second strength does not exist.
#[test]
fn Test_A_Required_Naming_Should_Refuse_What_A_Preferred_Naming_Falls_Back_To()
{
    let fixture = Registry_With_One_Offer();

    let via_preference = fixture.registry.Resolve(&Need().Preferring(fixture.absent.clone()));
    let via_requirement = fixture.registry.Resolve_Requiring(&Need(), &fixture.absent);

    Assert_Preference_Falls_Back(&via_preference);
    Assert_Requirement_Refuses(via_requirement, fixture.absent, fixture.answering);
}
