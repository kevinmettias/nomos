//! [`ResultProperties`], the three answers a finding carries that a printed line loses.

use crate::response::FindingBucket;
use nomos_contracts::Finding;
use serde::Serialize;

/// SARIF 2.1.0 §3.8's property bag, as one result carries it.
///
/// `nomos_contracts::Finding`'s own doc says why it is a type rather than a formatted line:
/// a rendering that keeps only the text cannot be asked whether the rule reached its subject,
/// how the claim was come by, or whether anything would have failed a build over it. `level`
/// folds the first and third together into one word a consumer acts on; this bag keeps all
/// three apart, in the same labels the contract types render them under, so nothing the
/// finding knew is lost in the crossing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ResultProperties
{
    /// Which of a gate run's buckets the finding fell into, and absent for a check run, which
    /// applies no policy and so has no bucket to report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bucket: Option<FindingBucket>,
    /// Whether the rule actually reached this subject -- `Applicability`'s own label.
    pub(crate) applicability: &'static str,
    /// How the claim was come by -- `EvidenceClass`'s own label.
    pub(crate) evidence: &'static str,
    /// What the wiring would do about a violation -- `GateCategory`'s own label.
    pub(crate) gate: &'static str,
}

impl ResultProperties
{
    /// `finding`'s three honesty labels, beside the `bucket` a gate run placed it in.
    pub(crate) const fn Of(finding: &Finding, bucket: Option<FindingBucket>) -> Self
    {
        return Self {
            bucket,
            applicability: finding.applicability.Label(),
            evidence: finding.evidence.Label(),
            gate: finding.gate.Label(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Finding_At;

    #[test]
    fn Test_Of_Should_Carry_The_Findings_Three_Labels_Beside_Its_Bucket()
    {
        let rendered = serde_json::to_value(ResultProperties::Of(&Finding_At("a-rule", "a.rs:1"), Some(FindingBucket::Suppressed)))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/bucket").and_then(serde_json::Value::as_str), Some("suppressed"), "{rendered}");
        assert_eq!(rendered.pointer("/applicability").and_then(serde_json::Value::as_str), Some("Supported"), "{rendered}");
        assert_eq!(rendered.pointer("/evidence").and_then(serde_json::Value::as_str), Some("Derived"), "{rendered}");
        assert_eq!(rendered.pointer("/gate").and_then(serde_json::Value::as_str), Some("Blocking"), "{rendered}");
    }

    /// A check run has no bucket, and the property is absent rather than `null` so a consumer
    /// cannot read "no policy was applied" as "a policy applied and said nothing".
    #[test]
    fn Test_A_Result_With_No_Bucket_Should_Serialize_No_Bucket_Property_At_All()
    {
        let rendered = serde_json::to_value(ResultProperties::Of(&Finding_At("a-rule", "a.rs:1"), None))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert!(rendered.get("bucket").is_none(), "{rendered}");
    }
}
