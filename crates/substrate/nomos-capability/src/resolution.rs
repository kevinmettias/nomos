//! The answer to "who serves this capability, at what guarantee?".

// What a resolution is made of: the ranking that chooses between usable offers, the
// standing an offer has under that ranking, and the named absence when none is usable.
mod selection;
mod standing;
mod unmet;

pub use selection::Selection;
pub use standing::Standing;
pub use unmet::Unmet;

use crate::ProviderOffer;
use nomos_contracts::CapabilityId;
use nomos_contracts::Applicability;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

    fn Offer() -> ProviderOffer
    {
        return ProviderOffer {
            provider: ProviderId::New("nomos.test.resolution"),
            capability: CapabilityId::New("nomos.cap.test.resolution"),
            version: ContractVersion::New(1, 0),
            guarantee: Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        };
    }

    fn Satisfied() -> Resolution
    {
        return Resolution::Satisfied {
            selection: Selection {
                chosen: Offer(),
                alternatives: Vec::new(),
            },
            applicability: Applicability::Supported,
        };
    }

    fn Unsatisfied() -> Resolution
    {
        return Resolution::Unsatisfied {
            capability: CapabilityId::New("nomos.cap.test.resolution"),
            reason: Unmet::NoProvider,
        };
    }

    #[test]
    fn Test_Applicability_Should_Read_From_Either_Branch()
    {
        assert_eq!(Satisfied().Applicability(), Applicability::Supported);
        assert_eq!(Unsatisfied().Applicability(), Applicability::MissingCapability);
    }

    #[test]
    fn Test_Offer_Should_Be_Present_Only_When_Satisfied()
    {
        assert_eq!(Satisfied().Offer(), Some(&Offer()));
        assert_eq!(Unsatisfied().Offer(), None);
    }

    #[test]
    fn Test_Selection_Should_Be_Present_Only_When_Satisfied()
    {
        assert!(Satisfied().Selection().is_some());
        assert!(Unsatisfied().Selection().is_none());
    }
}
