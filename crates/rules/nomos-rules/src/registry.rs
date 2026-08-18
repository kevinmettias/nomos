//! A rule's own registration contract, extracted before a second rule needs it.
//!
//! `OD-RULES-004` decided this now, ahead of a second rule shipping, so a rule package can
//! be designed against a stated shape rather than by copying `Check_Completeness_Mirrors`'s
//! own hand-written composition into a crate that does not exist yet. Registration and
//! selection are different questions — `OD-HOST-004` already answered the second one, and
//! this module answers only the first: how a rule becomes a nameable thing a registry
//! holds, not which rules `Run()` calls. Nothing here is consulted by `Run()`, and nothing
//! here changes what runs on any given `nomos check`.

use nomos_contracts::RuleId;
use std::collections::BTreeMap;

/// A rule package's claim that it offers one rule, at the contract record and version its
/// implementation was written against.
///
/// Shaped after [`nomos_capability::ProviderOffer`] closely enough that a rule package can
/// hand one of these to a [`RuleRegistry`] without either party naming the other's crate —
/// the same seam `ProviderId`/`ProviderOffer` give `nomos_capability::Registry`. A
/// capability and a guarantee have no rule-side equivalent yet, because nothing here ranks
/// or selects between offers the way a capability registry does; a rule's own record
/// citation (`CONTRACT_RECORD`/`CONTRACT_RECORD_VERSION`, `mirror.rs`'s own worked example)
/// is the closer analogue to what a provider's `version` means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleOffer
{
    /// The rule's own identity, the same [`RuleId`] a [`nomos_contracts::Finding`] carries.
    pub rule: RuleId,
    /// The governing record this rule's implementation cites, e.g. `"D-134"`.
    pub contract_record: String,
    /// The version of `contract_record` this implementation was written against.
    pub contract_record_version: u32,
}

/// Why a [`RuleOffer`] was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleRegistryError
{
    /// This [`RuleId`] is already registered. Two offers for one rule is two
    /// implementations claiming one identity, the same reason
    /// `nomos_capability::Registry::Offer` refuses a provider's second offer against a
    /// capability it already offers.
    AlreadyOffered(RuleId),
}

/// Rules, registered by the packages that offer them.
///
/// Ordered by [`RuleId`] throughout, the same reason `nomos_capability::Registry` orders by
/// name: an iteration order that depended on a hasher would make two runs over the same
/// registrations disagree for no reason a caller could see.
#[derive(Debug, Default)]
pub struct RuleRegistry
{
    offers: BTreeMap<RuleId, RuleOffer>,
}

impl RuleRegistry
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    /// # Errors
    ///
    /// Returns [`RuleRegistryError::AlreadyOffered`] if this offer's [`RuleId`] is already
    /// registered.
    pub fn Offer(&mut self, offer: RuleOffer) -> Result<(), RuleRegistryError>
    {
        if self.offers.contains_key(&offer.rule)
        {
            return Err(RuleRegistryError::AlreadyOffered(offer.rule));
        }

        self.offers.insert(offer.rule.clone(), offer);
        return Ok(());
    }

    /// Every rule registered, in [`RuleId`] order.
    pub fn Offers(&self) -> impl Iterator<Item = &RuleOffer>
    {
        return self.offers.values();
    }

    /// The offer registered under this [`RuleId`], if any.
    #[must_use]
    pub fn Offered(&self, rule: &RuleId) -> Option<&RuleOffer>
    {
        return self.offers.get(rule);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Offer(id: &str) -> RuleOffer
    {
        return RuleOffer {
            rule: RuleId::New(id),
            contract_record: "D-134".to_owned(),
            contract_record_version: 2,
        };
    }

    #[test]
    fn Test_A_Rule_Should_Be_Findable_By_Its_Id()
    {
        let mut registry = RuleRegistry::New();
        registry.Offer(Offer("completeness-mirror")).expect("first offer");

        let found = registry.Offered(&RuleId::New("completeness-mirror"));

        assert_eq!(found, Some(&Offer("completeness-mirror")));
    }

    #[test]
    fn Test_A_Second_Offer_For_One_Rule_Should_Be_Refused()
    {
        let mut registry = RuleRegistry::New();
        registry.Offer(Offer("completeness-mirror")).expect("first offer");

        let refused = registry.Offer(Offer("completeness-mirror"));

        assert_eq!(
            refused,
            Err(RuleRegistryError::AlreadyOffered(RuleId::New("completeness-mirror")))
        );
    }

    #[test]
    fn Test_An_Empty_Registry_Should_Offer_Nothing()
    {
        let registry = RuleRegistry::New();

        assert_eq!(registry.Offers().count(), 0);
        assert_eq!(registry.Offered(&RuleId::New("completeness-mirror")), None);
    }
}
