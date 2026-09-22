//! [`ReportingDescriptor`], one rule as `tool.driver.rules` lists it.

use super::descriptor_properties::DescriptorProperties;
use crate::response::RuleOfferResponse;
use serde::Serialize;

/// SARIF 2.1.0 §3.49's `reportingDescriptor` object, carrying the one property the
/// specification requires and the one thing beyond it this workspace's registry knows.
///
/// No `shortDescription` or `fullDescription` is emitted. A rule's implementation cites a
/// governing record and nothing here has that record's prose at hand without reading the
/// specification store; a description invented from the id would be a second spelling of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ReportingDescriptor
{
    /// The rule's stable identifier, as `RuleId` spells it.
    pub(crate) id: String,
    /// What the registry cites for this rule, when the registry holds it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<DescriptorProperties>,
}

impl ReportingDescriptor
{
    /// A rule the registry holds, with what it cites.
    pub(crate) fn Registered(offer: &RuleOfferResponse) -> Self
    {
        return Self { id: offer.rule.As_Str().to_owned(), properties: Some(DescriptorProperties::Of(offer)) };
    }

    /// A rule a finding cites that the registry does not hold -- a rule composed elsewhere, or
    /// a test-built finding -- listed by id alone so every `ruleId` in the run still resolves.
    pub(crate) fn Bare(id: &str) -> Self
    {
        return Self { id: id.to_owned(), properties: None };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Registered_Should_Carry_The_Offers_Id_And_Its_Citation()
    {
        let offer = RuleOfferResponse { rule: RuleId::New("a-rule"), contract_record: "D-1".to_owned(), contract_record_version: 1 };

        let rendered = serde_json::to_value(ReportingDescriptor::Registered(&offer)).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/id").and_then(serde_json::Value::as_str), Some("a-rule"), "{rendered}");
        assert_eq!(rendered.pointer("/properties/contractRecord").and_then(serde_json::Value::as_str), Some("D-1"), "{rendered}");
    }

    #[test]
    fn Test_Bare_Should_Carry_An_Id_And_No_Properties_Object_At_All()
    {
        let rendered = serde_json::to_value(ReportingDescriptor::Bare("a-rule")).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/id").and_then(serde_json::Value::as_str), Some("a-rule"), "{rendered}");
        assert!(rendered.get("properties").is_none(), "{rendered}");
    }
}
