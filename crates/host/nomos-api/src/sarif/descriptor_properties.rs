//! [`DescriptorProperties`], what the registry knows about a rule beyond its id.

use crate::response::RuleOfferResponse;
use serde::Serialize;

/// SARIF 2.1.0 §3.8's property bag, as one `reportingDescriptor` carries it.
///
/// The governing record a rule's implementation cites, and the version of it the
/// implementation was written against -- `AGT-008`'s "rule version" clause, which
/// `nomos_rules::RuleOffer` already carries and `nomos gate plan` already reports. A consumer
/// holding a result can follow its `ruleId` to this and from here to the record, which is the
/// one thing a bare id cannot give it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DescriptorProperties
{
    /// The governing record, e.g. `"D-134"`.
    pub(crate) contract_record: String,
    /// The version of `contract_record` the rule's implementation cites.
    pub(crate) contract_record_version: u32,
}

impl DescriptorProperties
{
    /// What `offer` cites.
    pub(crate) fn Of(offer: &RuleOfferResponse) -> Self
    {
        return Self { contract_record: offer.contract_record.clone(), contract_record_version: offer.contract_record_version };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::RuleId;

    /// A version number that is not the crate's current one, so `Of` substituting a live
    /// version for the offer's own would fail the check below.
    const OFFERED_CONTRACT_RECORD_VERSION: u32 = 2;

    #[test]
    fn Test_Of_Should_Carry_The_Offers_Record_And_Version_Under_Camel_Case_Names()
    {
        let offer = RuleOfferResponse {
            rule: RuleId::New("a-rule"),
            contract_record: "D-134".to_owned(),
            contract_record_version: OFFERED_CONTRACT_RECORD_VERSION,
        };

        let rendered = serde_json::to_value(DescriptorProperties::Of(&offer)).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/contractRecord").and_then(serde_json::Value::as_str), Some("D-134"), "{rendered}");
        assert_eq!(rendered.pointer("/contractRecordVersion").and_then(serde_json::Value::as_u64), Some(u64::from(OFFERED_CONTRACT_RECORD_VERSION)), "{rendered}");
    }
}
