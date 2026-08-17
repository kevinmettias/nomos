// The two error types are the registry's own vocabulary and nobody else raises them, so
// they live beneath it rather than beside it.
mod error;
mod error_kind;

pub use error::RegistryError;
pub use error_kind::RegistryErrorKind;

use crate::CapabilityContract;
use crate::OfferRefusal;
use crate::ProviderOffer;
use crate::Requirement;
use crate::{Resolution, Selection, Unmet};
use nomos_contracts::{Applicability, CapabilityId, ProviderId};
use std::collections::BTreeMap;

/// Why a *required* naming was not honoured.
///
/// [`Unmet`] already has the vocabulary for "nobody usable answered at all", and that half
/// is reused unchanged — a required naming that nobody could have answered fails for the
/// same reason a preferred one would have. The half `Unmet` cannot say is that somebody
/// usable *did* answer and it was not who was named; [`Registry::Resolve`] calls that
/// [`Applicability::SupportedWithFallback`] and returns [`Resolution::Satisfied`].
/// [`Registry::Resolve_Requiring`] does not, because the caller said which provider, not
/// merely which guarantee — `docs/records/OD-CAPABILITY-005` is why that difference is
/// enough to refuse rather than merely to flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiredUnmet
{
    /// No usable offer existed at all, whoever was required. Carries the same reason
    /// [`Registry::Resolve`] would have reported.
    Unavailable(Unmet),
    /// A usable offer existed and it was not the required provider.
    AnsweredByOther
    {
        required: ProviderId,
        answered: ProviderId,
    },
}

/// The answer to a requirement whose named provider is not negotiable.
///
/// Deliberately shaped like [`Resolution`] — satisfied or not, never a bare no — because it
/// answers the same question at a different strength. It is a distinct type rather than a
/// third [`Applicability`] or a flag beside one, because nothing short of a different return
/// type stops a caller from reading `Applicability::SupportedWithFallback` as good enough
/// when it required the naming rather than merely preferring it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequiredResolution
{
    /// The required provider answered. There is exactly one way to reach this: nothing else
    /// is reported, because a required naming that is honoured is indistinguishable from an
    /// ordinary answer.
    Satisfied
    {
        selection: Selection,
    },
    /// The required provider did not answer — whether because nobody could, or because
    /// somebody else did.
    Unsatisfied
    {
        capability: CapabilityId,
        reason: RequiredUnmet,
    },
}

/// Capability contracts and the offers against them.
///
/// Ordered maps throughout. Resolution picks a provider, and a provider picked by hash
/// order is a run whose results depend on the hasher.
#[derive(Debug, Default)]
pub struct Registry
{
    declared: BTreeMap<CapabilityId, CapabilityContract>,
    offers: BTreeMap<CapabilityId, Vec<ProviderOffer>>,
}

impl Registry
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    /// # Errors
    ///
    /// Returns [`RegistryErrorKind::AlreadyDeclared`] if the capability already has a
    /// contract. Two contracts for one capability is two meanings for one name.
    pub fn Declare(&mut self, contract: CapabilityContract) -> Result<(), RegistryError>
    {
        if self.declared.contains_key(&contract.id)
        {
            return Err(RegistryError {
                capability: contract.id,
                kind: RegistryErrorKind::AlreadyDeclared,
            });
        }

        self.declared.insert(contract.id.clone(), contract);
        return Ok(());
    }

    /// # Errors
    ///
    /// Returns [`RegistryError`] if the capability is undeclared, the provider already
    /// offers it, or the offer claims more than the contract's ceiling.
    pub fn Offer(&mut self, offer: ProviderOffer) -> Result<(), RegistryError>
    {
        self.Refuse_Unofferable(&offer)?;

        let against = self.offers.entry(offer.capability.clone()).or_default();
        if against.iter().any(|existing| existing.provider == offer.provider)
        {
            return Err(Self::Refused(
                &offer.capability,
                &offer.provider,
                OfferRefusal::Duplicate,
            ));
        }

        against.push(offer);
        // Name order, which is no longer what selects. It is here so that the input to
        // [`Selection`] does not depend on registration order, and so the tiebreak it
        // falls back to when the guarantee ranks nothing is at least the same tiebreak
        // every time.
        against.sort_by(|left, right| left.provider.cmp(&right.provider));
        return Ok(());
    }

    /// Declares `contract` and registers `offer` against it, in that order.
    ///
    /// The pairing rather than a convenience. A contract nothing offers against resolves to
    /// nothing, so every composition in this workspace declares and then immediately offers;
    /// written out, that pair was the same two statements repeated at every composition
    /// root and in every test that builds one. The order is the whole reason it cannot be
    /// two calls a caller might transpose: [`Offer`](Self::Offer) refuses an undeclared
    /// capability.
    ///
    /// A second provider against the same contract is still [`Offer`](Self::Offer). This
    /// declares, so calling it twice for one capability is `AlreadyDeclared` and says so.
    ///
    /// # Errors
    ///
    /// Whatever [`Declare`](Self::Declare) or [`Offer`](Self::Offer) returns, unchanged: the
    /// contract is left declared if the offer is the half that is refused, because a partial
    /// registration a caller can inspect is more use than one this rolled back silently.
    pub fn Declare_And_Offer(
        &mut self,
        contract: CapabilityContract,
        offer: ProviderOffer,
    ) -> Result<(), RegistryError>
    {
        self.Declare(contract)?;

        return self.Offer(offer);
    }

    /// One provider's offer, refused for a named reason.
    fn Refused(
        capability: &CapabilityId,
        provider: &ProviderId,
        refusal: OfferRefusal,
    ) -> RegistryError
    {
        return RegistryError {
            capability: capability.clone(),
            kind: RegistryErrorKind::Offer {
                provider: provider.clone(),
                refusal,
            },
        };
    }

    /// Every offer standing against a capability, in name order.
    ///
    /// An empty slice rather than an absence, because "nobody offers this" and "this
    /// capability has no entry yet" are the same answer to the caller and giving them two
    /// shapes would make the caller decide which.
    fn Offers_For(&self, requirement: &Requirement) -> &[ProviderOffer]
    {
        return self
            .offers
            .get(&requirement.capability)
            .map_or(&[] as &[ProviderOffer], Vec::as_slice);
    }

    /// Why the contract itself cannot answer this requirement, if it cannot.
    ///
    /// Both answers are about the capability rather than about any offer: it was never
    /// declared, or it was declared at a version this caller cannot read. Neither depends
    /// on who is offering, which is why they are asked before the offers are looked at.
    fn Unreadable(&self, requirement: &Requirement) -> Option<Unmet>
    {
        let Some(contract) = self.declared.get(&requirement.capability)
        else
        {
            return Some(Unmet::Undeclared);
        };

        if !requirement.version.Can_Read(contract.version)
        {
            return Some(Unmet::VersionMismatch {
                offered: contract.version,
            });
        }

        return None;
    }

    /// An offer against a capability that will not take it.
    ///
    /// The ceiling is checked here rather than at resolution because it is a statement about
    /// the offer and not about any requirement: a provider claiming more than the contract
    /// admits is wrong whether or not anybody ever asks for that much.
    fn Refuse_Unofferable(&self, offer: &ProviderOffer) -> Result<(), RegistryError>
    {
        let Some(contract) = self.declared.get(&offer.capability)
        else
        {
            return Err(Self::Refused(
                &offer.capability,
                &offer.provider,
                OfferRefusal::ForUndeclared,
            ));
        };
        if !contract.ceiling.Satisfies(&offer.guarantee)
        {
            return Err(Self::Refused(
                &offer.capability,
                &offer.provider,
                OfferRefusal::ExceedsCeiling,
            ));
        }

        return Ok(());
    }

    /// Answers a requirement.
    ///
    /// Never returns a bare no. Every path either names a provider or names what is
    /// missing, and no path produces [`Applicability::NotApplicable`].
    ///
    /// # Which of several usable offers answers
    ///
    /// The floor decides which offers are *usable*; [`Selection`] decides which usable
    /// offer is *chosen*, and the rule is that no usable offer is strictly stronger than
    /// the one that answers. A named preference outranks that rule, and every offer the
    /// rule passed over comes back in the selection. Nothing here consults a provider's
    /// name except to break a tie the guarantee itself does not break — see
    /// [`Selection::Unranked`], which is how a caller tells that apart from a decision.
    ///
    /// This used to be `offers.find(usable)` over a name-sorted list, which resolved to
    /// whichever provider sorted first. `docs/records/OD-CAPABILITY-001` records what
    /// decided it.
    #[must_use]
    pub fn Resolve(&self, requirement: &Requirement) -> Resolution
    {
        let selection = match self.Selected(requirement)
        {
            Ok(selection) => selection,
            Err(reason) => return Resolution::Unsatisfied {
                capability: requirement.capability.clone(),
                reason,
            },
        };
        let applicability = Honoured(requirement, &selection);

        return Resolution::Satisfied {
            selection,
            applicability,
        };
    }

    /// Answers a requirement whose named provider is required rather than preferred.
    ///
    /// `Resolve` treats a name as a preference: an answer from anybody else still satisfies
    /// the requirement, reported through `Applicability::SupportedWithFallback` so a caller
    /// that reads that far can tell. This treats the same name as the whole point. An offer
    /// from anybody but `required` is refused rather than substituted, and the refusal names
    /// both `required` and, when somebody else was usable, who that was.
    ///
    /// This does not let the registry decide anything `OD-CAPABILITY-001` reserved to the
    /// caller — the guarantee still ranks and a caller still spends. What changed is not who
    /// ranks; it is that `required` forces the ranking's head the way a preference already
    /// could, and a caller who cannot be answered by anyone else is told so instead of being
    /// handed a fallback it did not ask for. `docs/records/OD-CAPABILITY-005` records why
    /// refusing is right here though `OD-CAPABILITY-001` declined to let the registry refuse
    /// on the caller's behalf: there, no caller had said which provider it would accept:
    /// here, one has.
    #[must_use]
    pub fn Resolve_Requiring(
        &self,
        requirement: &Requirement,
        required: &ProviderId,
    ) -> RequiredResolution
    {
        let scoped = Requirement {
            preferred: Some(required.clone()),
            ..requirement.clone()
        };

        let selection = match self.Selected(&scoped)
        {
            Ok(selection) => selection,
            Err(reason) => return RequiredResolution::Unsatisfied {
                capability: requirement.capability.clone(),
                reason: RequiredUnmet::Unavailable(reason),
            },
        };

        if &selection.chosen.provider == required
        {
            return RequiredResolution::Satisfied { selection };
        }

        return RequiredResolution::Unsatisfied {
            capability: requirement.capability.clone(),
            reason: RequiredUnmet::AnsweredByOther {
                required: required.clone(),
                answered: selection.chosen.provider.clone(),
            },
        };
    }

    /// Which offer answers, or which of the four things was missing.
    ///
    /// The four are distinct on purpose: nobody declared it, it was declared at a version
    /// this caller cannot read, nobody offers it, and everyone who offers it is below the
    /// floor. Collapsing any two of them would leave a caller unable to tell a composition
    /// mistake from a missing dependency.
    fn Selected(&self, requirement: &Requirement) -> Result<Selection, Unmet>
    {
        if let Some(reason) = self.Unreadable(requirement)
        {
            return Err(reason);
        }

        let offers = self.Offers_For(requirement);
        let Some(first) = offers.first()
        else
        {
            return Err(Unmet::NoProvider);
        };

        let usable = Usable(offers, requirement);

        return Selection::Over(usable, requirement.preferred.as_ref()).ok_or_else(|| {
            return Unmet::BelowRequirement {
                closest: first.provider.clone(),
            };
        });
    }

    pub fn Declared(&self) -> impl Iterator<Item = &CapabilityContract>
    {
        return self.declared.values();
    }

    #[must_use]
    pub fn Offers(&self, capability: &CapabilityId) -> &[ProviderOffer]
    {
        return self
            .offers
            .get(capability)
            .map_or(&[] as &[ProviderOffer], Vec::as_slice);
    }
}

/// What resolution says about a capability nobody installed, pinned.
///
/// These assert the half of `P11-KNOWLEDGE-ABSENT` that belongs here, and they are in `src`
/// rather than in `tests/` because they are about this type's own vocabulary — the same
/// reason `selection.rs` keeps its own.
///
/// The other half deliberately has no test here, because it has no mechanism here: whether
/// a provider that answered found anything *relevant* is a statement about content, and a
/// registry that could report it would be grading a provider's work. `OD-CAPABILITY-004`
/// carries that argument and says where the combined distinction lives instead.
/// Every offer this requirement could actually use: readable at its version, and at or
/// above its floor.
fn Usable(offers: &[ProviderOffer], requirement: &Requirement) -> Vec<ProviderOffer>
{
    return offers
        .iter()
        .filter(|offer| {
            return requirement.version.Can_Read(offer.version)
                && offer.guarantee.Satisfies(&requirement.minimum);
        })
        .cloned()
        .collect();
}

/// Whether the answer came from the provider the caller asked for.
///
/// A named preference that was not honoured is what makes this a fallback. The judgment
/// still stands; its provenance is not what was asked for, and a caller that cannot tell
/// cannot record why.
///
/// Compared against who actually answered rather than against a second search for the
/// preference, because those are the same question and asking it twice is how the two
/// answers come to disagree.
fn Honoured(requirement: &Requirement, selection: &Selection) -> Applicability
{
    let Some(preferred) = &requirement.preferred
    else
    {
        return Applicability::Supported;
    };

    if *preferred == selection.chosen.provider
    {
        return Applicability::Supported;
    }

    return Applicability::SupportedWithFallback;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::ProviderId;
    use nomos_contracts::ContractVersion;
    use crate::Requirement;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    const CAPABILITY: &str = "nomos.cap.test.knowledge";

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
    #[test]
    fn Test_Every_Absence_Should_Report_As_A_Missing_Capability_Today()
    {
        for reason in [
            Unmet::Undeclared,
            Unmet::NoProvider,
            Unmet::VersionMismatch {
                offered: ContractVersion::New(2, 0),
            },
            Unmet::BelowRequirement {
                closest: ProviderId::New("nomos.test.knowledge"),
            },
        ]
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
    #[test]
    fn Test_A_Required_Naming_Should_Refuse_What_A_Preferred_Naming_Falls_Back_To()
    {
        let mut registry = Registry::New();
        registry.Declare(Contract()).expect("declared once");
        registry.Offer(Offer()).expect("within the ceiling");

        let absent = ProviderId::New("nomos.test.absent");
        let answering = Offer().provider;

        let via_preference = registry.Resolve(&Need().Preferring(absent.clone()));
        let via_requirement = registry.Resolve_Requiring(&Need(), &absent);

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
            RequiredUnmet::AnsweredByOther {
                required: absent,
                answered: answering,
            },
            "the refusal must name both who was required and who would have answered"
        );
    }
}
