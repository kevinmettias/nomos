//! Every way a composition is refused before anything is resolved.

use nomos_contracts::ProviderId;
use nomos_contracts::CapabilityId;
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
