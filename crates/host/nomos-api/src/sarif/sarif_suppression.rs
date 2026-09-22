//! [`SarifSuppression`], what a gate policy did about a finding the run kept from blocking.

use crate::response::FindingBucket;
use serde::Serialize;

/// SARIF 2.1.0 §3.35.3's `external` suppression kind: the decision lives outside the source
/// file, which is where every one of this workspace's policies lives -- a declared
/// `nomos-gate.json` or a caller-built `GateCommand`, never a marker in the judged file.
const EXTERNAL_KIND: &str = "external";

const CALIBRATED_JUSTIFICATION: &str =
    "calibrated: an adoption policy declares this rule not yet blocking for this repository, so the finding is reported and cannot fail the build";
const SUPPRESSED_JUSTIFICATION: &str =
    "suppressed: a declared suppression matches this finding's rule and subject, so the finding is reported and cannot fail the build";
const BASELINED_JUSTIFICATION: &str =
    "baselined: a declared baseline debt entry accepts this finding's rule and subject within its allowance, so the finding is reported and cannot fail the build";
const BELOW_EVIDENCE_FLOOR_JUSTIFICATION: &str =
    "below_evidence_floor: this gate declares an evidence floor above the class backing this finding, so the finding is reported and cannot fail the build";

/// SARIF 2.1.0 §3.35's `suppression` object.
///
/// A suppressed, baselined or calibrated finding keeps its whole result -- its level, its
/// location, its message -- and carries one of these beside it, so a consumer can tell a
/// finding a policy answered from one that failed the build without either being dropped.
/// That is the same "never silently" every one of `nomos-gate-orchestration`'s policy
/// increments promises for its own bucket, projected into the one vocabulary a SARIF consumer
/// reads it from.
///
/// The justification names the bucket first, in the same word `FindingBucket` serializes
/// under, so a consumer that wants the bucket without reading prose has it in
/// [`super::result_properties::ResultProperties`] and one that reads only the suppression
/// still learns which policy spoke.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct SarifSuppression
{
    /// Always [`EXTERNAL_KIND`]; see its doc.
    pub(crate) kind: &'static str,
    /// Which policy kept the finding from blocking, and what that means for the build.
    pub(crate) justification: &'static str,
}

impl SarifSuppression
{
    /// The suppression a finding in `bucket` carries, or `None` for a bucket that blocks.
    ///
    /// `BelowEvidenceFloor` carries one because the finding was reported and could not fail
    /// the build, which is exactly what §3.35 describes, and its justification names the floor
    /// rather than borrowing a neighbour's word: `OD-GATE-034` keeps a policy a person authored
    /// apart from a mechanical statement about a class of evidence, and a consumer told
    /// "calibrated" would go looking for a calibration nobody wrote.
    ///
    /// `BaselineExceeded` is `None` on purpose: `OD-GATE-030` decided a baseline may tolerate
    /// no more than the quantity it adopted, and a scope past its allowance blocks as a whole
    /// -- a suppression on any of its occurrences would tell a consumer the entry still
    /// covers them.
    pub(crate) const fn Of(bucket: FindingBucket) -> Option<Self>
    {
        return match bucket
        {
            FindingBucket::Blocking | FindingBucket::BaselineExceeded => None,
            FindingBucket::Calibrated => Some(Self { kind: EXTERNAL_KIND, justification: CALIBRATED_JUSTIFICATION }),
            FindingBucket::Suppressed => Some(Self { kind: EXTERNAL_KIND, justification: SUPPRESSED_JUSTIFICATION }),
            FindingBucket::Baselined => Some(Self { kind: EXTERNAL_KIND, justification: BASELINED_JUSTIFICATION }),
            FindingBucket::BelowEvidenceFloor => Some(Self { kind: EXTERNAL_KIND, justification: BELOW_EVIDENCE_FLOOR_JUSTIFICATION }),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Should_Be_None_For_A_Bucket_That_Blocks()
    {
        assert_eq!(SarifSuppression::Of(FindingBucket::Blocking), None);
        assert_eq!(SarifSuppression::Of(FindingBucket::BaselineExceeded), None);
    }

    /// Each kept-from-blocking bucket names itself first, in the word it serializes under, so
    /// the prose and the property can never disagree about which policy spoke.
    #[test]
    fn Test_Of_Should_Name_The_Bucket_First_For_A_Bucket_A_Policy_Answered()
    {
        for (bucket, word) in [
            (FindingBucket::Calibrated, "calibrated"),
            (FindingBucket::Suppressed, "suppressed"),
            (FindingBucket::Baselined, "baselined"),
            (FindingBucket::BelowEvidenceFloor, "below_evidence_floor"),
        ]
        {
            let suppression = SarifSuppression::Of(bucket).expect("a bucket a policy answered carries a suppression");

            assert_eq!(suppression.kind, "external", "{bucket:?}");
            assert!(suppression.justification.starts_with(word), "{bucket:?}: {}", suppression.justification);
            assert_eq!(serde_json::to_value(bucket).ok().and_then(|value| return value.as_str().map(str::to_owned)), Some(word.to_owned()));
        }
    }

    #[test]
    fn Test_A_Suppression_Should_Serialize_Its_Kind_And_Justification_Under_The_Specifications_Names()
    {
        let rendered = serde_json::to_value(SarifSuppression::Of(FindingBucket::Suppressed))
            .expect("a derived Serialize over two static strings has nothing to refuse");

        assert_eq!(rendered.pointer("/kind").and_then(serde_json::Value::as_str), Some("external"), "{rendered}");
        assert!(rendered.pointer("/justification").and_then(serde_json::Value::as_str).is_some(), "{rendered}");
    }
}
