use crate::contract::{CapabilityContract, ProviderOffer, Requirement};
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
        offer: ProviderOffer,
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

    #[must_use]
    pub const fn Offer(&self) -> Option<&ProviderOffer>
    {
        return match self
        {
            Self::Satisfied { offer, .. } => Some(offer),
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
        against.sort_by(|left, right| left.provider.cmp(&right.provider));
        return Ok(());
    }

    /// Answers a requirement.
    ///
    /// Never returns a bare no. Every path either names a provider or names what is
    /// missing, and no path produces [`Applicability::NotApplicable`].
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

        let usable = |offer: &&ProviderOffer| {
            return requirement.version.Can_Read(offer.version)
                && offer.guarantee.Satisfies(&requirement.minimum);
        };

        if let Some(preferred) = &requirement.preferred
            && let Some(offer) = offers
                .iter()
                .find(|offer| &offer.provider == preferred && usable(offer))
        {
            return Resolution::Satisfied {
                offer: offer.clone(),
                applicability: Applicability::Supported,
            };
        }

        let Some(offer) = offers.iter().find(usable)
        else
        {
            return unsatisfied(Unmet::BelowRequirement {
                closest: first.provider.clone(),
            });
        };

        // A named preference that was not honoured is what makes this a fallback. The
        // judgment still stands; its provenance is not what was asked for, and a caller
        // that cannot tell cannot record why.
        let applicability = if requirement.preferred.is_some()
        {
            Applicability::SupportedWithFallback
        }
        else
        {
            Applicability::Supported
        };

        return Resolution::Satisfied {
            offer: offer.clone(),
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
