//! Every way a composition is refused before anything is resolved.

use nomos_contracts::ProviderId;
use nomos_contracts::CapabilityId;

/// Why a declaration or an offer was refused, always naming the capability it was about.
///
/// The capability is the type's and not the kind's. Every refusal here is a refusal
/// *about one capability*, so a caller reads which one without matching on a reason it
/// does not otherwise care about, and a new reason cannot forget to say which capability
/// it concerns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryError
{
    pub capability: CapabilityId,
    pub kind: RegistryErrorKind,
}

impl core::fmt::Display for RegistryError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let capability = &self.capability;

        return match self.kind
        {
            RegistryErrorKind::AlreadyDeclared =>
            {
                write!(formatter, "{capability} is already declared")
            }
            RegistryErrorKind::Offer {
                ref provider,
                refusal,
            } => match refusal
            {
                OfferRefusal::ForUndeclared => write!(
                    formatter,
                    "{provider} offers {capability}, which no contract declares. An offer \
                     against nothing is a capability with no agreed meaning"
                ),
                OfferRefusal::ExceedsCeiling => write!(
                    formatter,
                    "{provider} claims more for {capability} than its contract permits. A \
                     provider grading its own work is how a syntactic answer comes to satisfy \
                     a rule that needs resolution"
                ),
                OfferRefusal::Duplicate =>
                {
                    write!(formatter, "{provider} already offers {capability}")
                }
            },
        };
    }
}

impl std::error::Error for RegistryError
{}

/// Which of the two things being refused was refused.
///
/// The three offer refusals are one variant carrying an [`OfferRefusal`] rather than
/// three, because all three are about an offer and an offer has a provider. Declaring a
/// contract has none, which is the whole distinction this level draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryErrorKind
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

/// Why one provider's offer will not stand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferRefusal
{
    /// An offer against a capability no contract declares.
    ForUndeclared,
    /// A provider claimed more than its contract permits.
    ExceedsCeiling,
    /// A second offer from a provider that already offers this capability.
    Duplicate,
}
