//! Why a [`super::RuleRegistry`] refused an offer.

use nomos_contracts::RuleId;

/// Why a [`super::RuleOffer`] was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleRegistryError
{
    /// This [`RuleId`] is already registered. Two offers for one rule is two
    /// implementations claiming one identity, the same reason
    /// `nomos_capability::Registry::Offer` refuses a provider's second offer against a
    /// capability it already offers.
    AlreadyOffered(RuleId),
}
