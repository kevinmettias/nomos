//! [`SarifResult`], one finding as a SARIF consumer reads it.

use super::result_properties::ResultProperties;
use super::sarif_level::SarifLevel;
use super::sarif_location::SarifLocation;
use super::sarif_message::SarifMessage;
use super::sarif_suppression::SarifSuppression;
use crate::response::FindingBucket;
use nomos_contracts::Finding;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The `partialFingerprints` key this workspace's subject identity travels under.
///
/// `Finding::subject` is the finding's identity across revisions and `Finding::locations` is
/// only where to look -- `OD-ANALYSIS-011` and `nomos_contracts::Finding`'s own doc both say
/// the two must not be confused. SARIF's fingerprint slot is the one place in a result that
/// means identity rather than position, so the digest goes there and nowhere else. Namespaced
/// so it cannot collide with a fingerprint a consumer computes for itself.
const SUBJECT_FINGERPRINT_KEY: &str = "nomos/subject";

/// SARIF 2.1.0 §3.27's `result` object.
///
/// `kind` is not emitted and so defaults to `fail`, the one value the specification lets
/// `level` mean anything under. Whether a policy then kept the finding from blocking is
/// `suppressions`' question, not `kind`'s -- see [`SarifLevel`]'s doc for why the level is
/// never downgraded to say so.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SarifResult
{
    /// The rule that produced the finding, as `RuleId` spells it. Every value here is also the
    /// `id` of one of the run's `tool.driver.rules`.
    pub(crate) rule_id: String,
    /// What the finding deserves.
    pub(crate) level: SarifLevel,
    /// What is wrong, for a person.
    pub(crate) message: SarifMessage,
    /// Every location the finding named, relative to the run's root, in the finding's own order.
    pub(crate) locations: Vec<SarifLocation>,
    /// Empty for a finding that blocked or that no policy addressed. Emitted even when empty:
    /// the specification reads an absent array as "unknown" and an empty one as "not
    /// suppressed", and this projection always knows which.
    pub(crate) suppressions: Vec<SarifSuppression>,
    /// [`SUBJECT_FINGERPRINT_KEY`] to the subject digest's hex form.
    pub(crate) partial_fingerprints: BTreeMap<String, String>,
    /// The honesty labels and the bucket.
    pub(crate) properties: ResultProperties,
}

impl SarifResult
{
    /// `finding` as a result, its locations relative to `root`, in `bucket` when a gate run
    /// placed it in one.
    pub(crate) fn Of(root: &Path, finding: &Finding, bucket: Option<FindingBucket>) -> Self
    {
        let mut partial_fingerprints = BTreeMap::new();
        partial_fingerprints.insert(SUBJECT_FINGERPRINT_KEY.to_owned(), finding.subject.to_string());

        return Self {
            rule_id: finding.rule.As_Str().to_owned(),
            level: SarifLevel::Of(finding),
            message: SarifMessage::Of(finding),
            locations: finding.locations.iter().map(|raw| return SarifLocation::Of(root, raw)).collect(),
            suppressions: bucket.and_then(SarifSuppression::Of).into_iter().collect(),
            partial_fingerprints,
            properties: ResultProperties::Of(finding, bucket),
        };
    }

    /// The rule this result cites.
    pub(crate) fn Rule_Id(&self) -> &str
    {
        return &self.rule_id;
    }

    /// The order results are emitted in: by rule, then by location, then by message.
    ///
    /// A run's findings arrive grouped by bucket and, within a bucket, in whatever order the
    /// rules ran; the same finding does not move between two projections of one response, but
    /// two responses that list their findings differently would otherwise project differently.
    /// Sorting on the result's own content makes the log a function of what was found rather
    /// than of the order it was found in.
    pub(crate) fn Canonical_Key(&self) -> (String, Vec<SarifLocation>, String)
    {
        return (self.rule_id.clone(), self.locations.clone(), self.message.text.clone());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Finding_At;

    fn Rendered(finding: &Finding, bucket: Option<FindingBucket>) -> serde_json::Value
    {
        return serde_json::to_value(SarifResult::Of(Path::new("."), finding, bucket))
            .expect("a derived Serialize over owned data has nothing to refuse");
    }

    #[test]
    fn Test_Of_Should_Carry_Rule_Level_Message_And_A_Physical_Location_Under_The_Specifications_Names()
    {
        let rendered = Rendered(&Finding_At("a-rule", "src/lib.rs:12"), Some(FindingBucket::Blocking));

        assert_eq!(rendered.pointer("/ruleId").and_then(serde_json::Value::as_str), Some("a-rule"), "{rendered}");
        assert_eq!(rendered.pointer("/level").and_then(serde_json::Value::as_str), Some("error"), "{rendered}");
        assert!(rendered.pointer("/message/text").and_then(serde_json::Value::as_str).is_some(), "{rendered}");
        assert_eq!(rendered.pointer("/locations/0/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str), Some("src/lib.rs"), "{rendered}");
        assert_eq!(rendered.pointer("/locations/0/physicalLocation/region/startLine").and_then(serde_json::Value::as_u64), Some(12), "{rendered}");
    }

    /// The identity slot carries the digest and the location slot carries the path: a
    /// consumer keyed on the fingerprint follows the finding across a file move, which is the
    /// property `OD-ANALYSIS-011` gives the subject and denies the location.
    #[test]
    fn Test_Of_Should_Put_The_Subject_Digest_Under_A_Namespaced_Partial_Fingerprint()
    {
        let finding = Finding_At("a-rule", "src/lib.rs:12");

        let rendered = Rendered(&finding, None);

        assert_eq!(
            rendered.pointer("/partialFingerprints/nomos~1subject").and_then(serde_json::Value::as_str),
            Some(finding.subject.to_string().as_str()),
            "{rendered}"
        );
    }

    #[test]
    fn Test_A_Blocking_Result_Should_Carry_An_Empty_Suppressions_Array_Rather_Than_None()
    {
        let rendered = Rendered(&Finding_At("a-rule", "src/lib.rs:12"), Some(FindingBucket::Blocking));

        assert_eq!(rendered.pointer("/suppressions").and_then(serde_json::Value::as_array).map(Vec::len), Some(0), "{rendered}");
    }

    #[test]
    fn Test_A_Result_A_Policy_Answered_Should_Carry_One_External_Suppression()
    {
        let rendered = Rendered(&Finding_At("a-rule", "src/lib.rs:12"), Some(FindingBucket::Baselined));

        assert_eq!(rendered.pointer("/suppressions/0/kind").and_then(serde_json::Value::as_str), Some("external"), "{rendered}");
        assert_eq!(rendered.pointer("/properties/bucket").and_then(serde_json::Value::as_str), Some("baselined"), "{rendered}");
    }

    #[test]
    fn Test_Of_Should_Carry_Every_Location_The_Finding_Named_In_Its_Own_Order()
    {
        let mut finding = Finding_At("a-rule", "b.rs:2");
        finding.locations.push("a.rs".to_owned());

        let rendered = Rendered(&finding, None);

        assert_eq!(rendered.pointer("/locations/0/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str), Some("b.rs"), "{rendered}");
        assert_eq!(rendered.pointer("/locations/1/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str), Some("a.rs"), "{rendered}");
    }

    /// The falsifier for the sort key: two results differing only in rule order their keys by
    /// rule, and two from one rule order by location.
    #[test]
    fn Test_Canonical_Key_Should_Order_By_Rule_Then_By_Location()
    {
        let later_rule = SarifResult::Of(Path::new("."), &Finding_At("b-rule", "a.rs:1"), None);
        let earlier_rule_later_file = SarifResult::Of(Path::new("."), &Finding_At("a-rule", "z.rs:1"), None);
        let earlier_rule_earlier_file = SarifResult::Of(Path::new("."), &Finding_At("a-rule", "b.rs:9"), None);

        assert!(earlier_rule_later_file.Canonical_Key() < later_rule.Canonical_Key());
        assert!(earlier_rule_earlier_file.Canonical_Key() < earlier_rule_later_file.Canonical_Key());
    }
}
