//! The answer to "who serves this capability, at what guarantee?".

use crate::provider_offer::ProviderOffer;
use crate::unmet::Unmet;
use nomos_contracts::CapabilityId;
use nomos_contracts::Applicability;
use crate::selection::Selection;
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
