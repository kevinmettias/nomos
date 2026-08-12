//! Every way a composition is refused before anything is resolved.

use nomos_contracts::CapabilityId;

use crate::offer_refusal::OfferRefusal;
use super::RegistryErrorKind;

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
