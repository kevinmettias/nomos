use crate::contract::{CapabilityContract, ProviderOffer, Requirement};
use crate::selection::Selection;
use nomos_contracts::{Applicability, CapabilityId, ContractVersion, ProviderId};
use std::collections::BTreeMap;

/// Why a requirement could not be met.
///
/// Separate from [`Applicability`] on purpose. `Applicability` is the vocabulary a run
/// reports in and is deliberately small; this is the actionable detail behind one of its
/// values. One relation, two projections — the alternative is growing `Applicability` a
/// variant per diagnosis until nothing can match on it exhaustively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unmet
{
    /// Nobody declared this capability. Distinct from having no provider, because the
    /// remedy is authoring a contract rather than installing something.
    Undeclared,
    /// Declared, and nothing offers it.
    NoProvider,
    /// Offered at a contract version this caller cannot read.
    VersionMismatch
    {
        offered: ContractVersion,
    },
    /// Offered, and no offer reaches the required guarantee.
    BelowRequirement
    {
        closest: ProviderId,
    },
}

impl Unmet
{
    /// How a run reports this.
    ///
    /// Every arm is [`Applicability::MissingCapability`] and none is
    /// [`Applicability::NotApplicable`] — that distinction is the entire point. "No
    /// provider offers what this rule needs" is coverage debt; "this rule does not bind
    /// this subject" is a judgement about the subject, and the registry is in no
    /// position to make it. Collapsing the two is how a rule that could not run reads
    /// like a rule that did not need to.
    #[must_use]
    pub const fn Applicability(&self) -> Applicability
    {
        return match *self
        {
            Self::Undeclared
            | Self::NoProvider
            | Self::VersionMismatch { .. }
            | Self::BelowRequirement { .. } => Applicability::MissingCapability,
        };
    }

    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Undeclared => "no capability contract declares this".to_owned(),
            Self::NoProvider => "the contract is declared and no provider offers it".to_owned(),
            Self::VersionMismatch { offered } => {
                format!("offered at contract version {offered}, which this caller cannot read")
            }
            Self::BelowRequirement { closest } => {
                format!("{closest} is the closest offer and does not reach the required guarantee")
            }
        };
    }
}

/// The answer to a requirement.
///
/// Two results, never a `bool` and never an `Option`. An `Option::None` here would be a
/// caller's invitation to write `unwrap_or_default`, and there is no defensible default
/// for "can this analysis be performed".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution
{
    Satisfied
    {
        /// Which offer answers, and every usable offer it was chosen over.
        ///
        /// A [`Selection`] rather than a bare offer, because the answer to "who answers"
        /// is not complete without "instead of whom". A caller that lowered its floor to
        /// buy coverage bought the weaker offers too, and a resolution that named only the
        /// winner would spend the floor on its behalf and hand back one provider.
        selection: Selection,
        applicability: Applicability,
    },
    Unsatisfied
    {
        capability: CapabilityId,
        reason: Unmet,
    },
}

impl Resolution
{
    /// How a run reports this. There is deliberately no `Is_Available`.
    #[must_use]
    pub fn Applicability(&self) -> Applicability
    {
        return match self
        {
            Self::Satisfied { applicability, .. } => *applicability,
            Self::Unsatisfied { reason, .. } => reason.Applicability(),
        };
    }

    /// The offer that answers.
    #[must_use]
    pub const fn Offer(&self) -> Option<&ProviderOffer>
    {
        return match self
        {
            Self::Satisfied { selection, .. } => Some(&selection.chosen),
            Self::Unsatisfied { .. } => None,
        };
    }

    /// The offer that answers together with the ones it was chosen over.
    #[must_use]
    pub const fn Selection(&self) -> Option<&Selection>
    {
        return match self
        {
            Self::Satisfied { selection, .. } => Some(selection),
            Self::Unsatisfied { .. } => None,
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError
{
    AlreadyDeclared
    {
        capability: CapabilityId,
    },
    OfferForUndeclared
    {
        capability: CapabilityId,
        provider: ProviderId,
    },
    /// A provider claimed more than its contract permits.
    ExceedsCeiling
    {
        capability: CapabilityId,
        provider: ProviderId,
    },
    DuplicateOffer
    {
        capability: CapabilityId,
        provider: ProviderId,
    },
}

impl core::fmt::Display for RegistryError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::AlreadyDeclared { capability } => {
                write!(formatter, "{capability} is already declared")
            }
            Self::OfferForUndeclared {
                capability,
                provider,
            } => write!(
                formatter,
                "{provider} offers {capability}, which no contract declares. An offer \
                 against nothing is a capability with no agreed meaning"
            ),
            Self::ExceedsCeiling {
                capability,
                provider,
            } => write!(
                formatter,
                "{provider} claims more for {capability} than its contract permits. A \
                 provider grading its own work is how a syntactic answer comes to satisfy \
                 a rule that needs resolution"
            ),
            Self::DuplicateOffer {
                capability,
                provider,
            } => write!(formatter, "{provider} already offers {capability}"),
        };
    }
}

impl std::error::Error for RegistryError {}

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
    /// Returns [`RegistryError::AlreadyDeclared`] if the capability already has a
    /// contract. Two contracts for one capability is two meanings for one name.
    pub fn Declare(&mut self, contract: CapabilityContract) -> Result<(), RegistryError>
    {
        if self.declared.contains_key(&contract.id)
        {
            return Err(RegistryError::AlreadyDeclared {
                capability: contract.id,
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
        let Some(contract) = self.declared.get(&offer.capability)
        else
        {
            return Err(RegistryError::OfferForUndeclared {
                capability: offer.capability,
                provider: offer.provider,
            });
        };

        if !contract.ceiling.Satisfies(&offer.guarantee)
        {
            return Err(RegistryError::ExceedsCeiling {
                capability: offer.capability,
                provider: offer.provider,
            });
        }

        let against = self.offers.entry(offer.capability.clone()).or_default();
        if against
            .iter()
            .any(|existing| existing.provider == offer.provider)
        {
            return Err(RegistryError::DuplicateOffer {
                capability: offer.capability,
                provider: offer.provider,
            });
        }

        against.push(offer);
        // Name order, which is no longer what selects. It is here so that the input to
        // [`Selection`] does not depend on registration order, and so the tiebreak it
        // falls back to when the guarantee ranks nothing is at least the same tiebreak
        // every time.
        against.sort_by(|left, right| left.provider.cmp(&right.provider));
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
        let unsatisfied = |reason: Unmet| Resolution::Unsatisfied {
            capability: requirement.capability.clone(),
            reason,
        };

        let Some(contract) = self.declared.get(&requirement.capability)
        else
        {
            return unsatisfied(Unmet::Undeclared);
        };

        if !requirement.version.Can_Read(contract.version)
        {
            return unsatisfied(Unmet::VersionMismatch {
                offered: contract.version,
            });
        }

        let offers = self
            .offers
            .get(&requirement.capability)
            .map_or(&[] as &[ProviderOffer], Vec::as_slice);

        let Some(first) = offers.first()
        else
        {
            return unsatisfied(Unmet::NoProvider);
        };

        let usable: Vec<ProviderOffer> = offers
            .iter()
            .filter(|offer| {
                return requirement.version.Can_Read(offer.version)
                    && offer.guarantee.Satisfies(&requirement.minimum);
            })
            .cloned()
            .collect();

        let Some(selection) = Selection::Over(usable, requirement.preferred.as_ref())
        else
        {
            return unsatisfied(Unmet::BelowRequirement {
                closest: first.provider.clone(),
            });
        };

        // A named preference that was not honoured is what makes this a fallback. The
        // judgment still stands; its provenance is not what was asked for, and a caller
        // that cannot tell cannot record why.
        //
        // Compared against who actually answered rather than against a second search for
        // the preference, because those are the same question and asking it twice is how
        // the two answers come to disagree.
        let applicability = match &requirement.preferred
        {
            Some(preferred) if *preferred != selection.chosen.provider => {
                Applicability::SupportedWithFallback
            }
            _ => Applicability::Supported,
        };

        return Resolution::Satisfied {
            selection,
            applicability,
        };
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
#[cfg(test)]
mod tests
{
    use super::*;
    use crate::contract::Requirement;
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

        let mut unoffered = Registry::New();
        unoffered.Declare(Contract()).expect("declared once");
        let unoffered = unoffered.Resolve(&Need());

        assert_ne!(undeclared, unoffered);
        assert!(matches!(
            undeclared,
            Resolution::Unsatisfied {
                reason: Unmet::Undeclared,
                ..
            }
        ));
        assert!(matches!(
            unoffered,
            Resolution::Unsatisfied {
                reason: Unmet::NoProvider,
                ..
            }
        ));
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
}
