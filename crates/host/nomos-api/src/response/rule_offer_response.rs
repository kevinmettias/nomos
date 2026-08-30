//! [`RuleOfferResponse`], one rule entry of [`super::gate_plan_response::GatePlanResponse::Planned`].

use nomos_contracts::RuleId;
use nomos_rules::RuleOffer;
use serde::Serialize;

/// A serializable twin of [`nomos_rules::RuleOffer`], for the same reason `crate::response::
/// disposition::Disposition` twins `GateRunOutcome`: the type it mirrors does not derive
/// `Serialize`, and growing `nomos-rules`' own public surface on behalf of one caller's wire
/// shape is not this increment's to spend.
#[derive(Debug, Serialize)]
pub struct RuleOfferResponse
{
    /// The rule's own identity.
    pub rule: RuleId,
    /// The governing record this rule's implementation cites, e.g. `"D-134"`.
    pub contract_record: String,
    /// The version of `contract_record` this implementation was written against.
    pub contract_record_version: u32,
}

impl RuleOfferResponse
{
    pub(crate) fn From(offer: RuleOffer) -> Self
    {
        return Self {
            rule: offer.rule,
            contract_record: offer.contract_record,
            contract_record_version: offer.contract_record_version,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_rules::COMPLETENESS_MIRROR;

    #[test]
    fn Test_From_Should_Carry_The_Offers_Rule_And_Contract_Record()
    {
        let offer = RuleOffer {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            contract_record: "D-134".to_owned(),
            contract_record_version: 2,
        };

        let response = RuleOfferResponse::From(offer.clone());

        assert_eq!(response.rule, offer.rule);
        assert_eq!(response.contract_record, offer.contract_record);
        assert_eq!(response.contract_record_version, offer.contract_record_version);
    }
}
