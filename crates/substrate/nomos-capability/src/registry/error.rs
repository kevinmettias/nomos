//! Every way a composition is refused before anything is resolved.

use nomos_contracts::CapabilityId;

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
        return write!(formatter, "{}", Message(&self.capability, &self.kind));
    }
}

/// The one human-readable sentence for `capability`'s refusal, gathered here rather than
/// assembled across several `write!` calls sharing one `Formatter` — a `RegistryError` is
/// one sentence, not several fragments.
fn Message(capability: &CapabilityId, kind: &RegistryErrorKind) -> String
{
    use crate::OfferRefusal;

    return match kind
    {
        RegistryErrorKind::AlreadyDeclared => format!("{capability} is already declared"),
        RegistryErrorKind::Offer { provider, refusal } => match refusal
        {
            OfferRefusal::ForUndeclared => format!(
                "{provider} offers {capability}, which no contract declares. An offer \
                 against nothing is a capability with no agreed meaning"
            ),
            OfferRefusal::ExceedsCeiling => format!(
                "{provider} claims more for {capability} than its contract permits. A \
                 provider grading its own work is how a syntactic answer comes to satisfy \
                 a rule that needs resolution"
            ),
            OfferRefusal::Duplicate => format!("{provider} already offers {capability}"),
        },
    };
}

impl std::error::Error for RegistryError
{}
