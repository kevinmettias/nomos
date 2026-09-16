//! Rules, registered by the packages that offer them.

use super::{RuleOffer, RuleRegistryError};
use nomos_contracts::RuleId;
use std::collections::BTreeMap;

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

    /// The contract-record revision every offer in these tests is written against — the
    /// fixture's own declaration, named so the reader is not left wondering whether the
    /// `2` is meant to match something elsewhere.
    const FIXTURE_CONTRACT_RECORD_VERSION: u32 = 2;

    #[test]
    fn Test_New_Should_Produce_A_Registry_Whose_Offered_Rule_Is_Findable_By_Its_Id()
    {
        let mut registry = RuleRegistry::New();
        registry
            .Offer(Rule_Offer("completeness-mirror"))
            .expect("RuleRegistry::New returns an empty registry, so this id is not yet in it");

        let found = registry.Offered(&RuleId::New("completeness-mirror"));

        assert_eq!(found, Some(&Rule_Offer("completeness-mirror")));
    }

    #[test]
    fn Test_A_Second_Offer_For_One_Rule_Should_Be_Refused()
    {
        let mut registry = RuleRegistry::New();
        registry
            .Offer(Rule_Offer("completeness-mirror"))
            .expect("RuleRegistry::New returns an empty registry, so this id is not yet in it");

        let refused = registry.Offer(Rule_Offer("completeness-mirror"));

        assert_eq!(
            refused,
            Err(RuleRegistryError::AlreadyOffered(RuleId::New("completeness-mirror")))
        );
    }

    #[test]
    fn Test_New_Should_Return_An_Empty_Registry()
    {
        let registry = RuleRegistry::New();

        assert_eq!(registry.Offers().count(), 0);
        assert_eq!(registry.Offered(&RuleId::New("completeness-mirror")), None);
    }

    #[test]
    fn Test_Offers_Should_List_Every_Registered_Rule()
    {
        let mut registry = RuleRegistry::New();
        registry
            .Offer(Rule_Offer("completeness-mirror"))
            .expect("RuleRegistry::New returns an empty registry, so this id is not yet in it");
        registry
            .Offer(Rule_Offer("dependency-direction"))
            .expect("only completeness-mirror has been offered so far, not this id");

        let listed: Vec<RuleOffer> = registry.Offers().cloned().collect();

        assert_eq!(listed, vec![Rule_Offer("completeness-mirror"), Rule_Offer("dependency-direction")]);
    }

    fn Rule_Offer(id: &str) -> RuleOffer
    {
        return RuleOffer {
            rule: RuleId::New(id),
            contract_record: "D-134".to_owned(),
            contract_record_version: FIXTURE_CONTRACT_RECORD_VERSION,
        };
    }
}
