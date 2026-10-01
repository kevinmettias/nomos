//! One provider's offer against a capability, and what this host settles about it.

use super::ProviderStanding;

/// A provider that offers a capability, and what this host establishes about its answering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OfferStanding
{
    /// The provider's declared identity, as the registry spells it.
    pub(crate) provider: String,
    /// What this host establishes about it.
    pub(crate) standing: ProviderStanding,
}
