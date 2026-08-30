//! Which of the two things being refused was refused.

use nomos_contracts::ProviderId;

use crate::OfferRefusal;

/// Which of the two things being refused was refused.
///
/// The three offer refusals are one variant carrying an [`OfferRefusal`] rather than
/// three, because all three are about an offer and an offer has a provider. Declaring a
/// contract has none, which is the whole distinction this level draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind
{
    /// A second contract for a capability that already has one.
    AlreadyDeclared,
    /// An offer that will not stand, and who made it.
    Offer
    {
        provider: ProviderId,
        refusal: OfferRefusal,
    },
}
