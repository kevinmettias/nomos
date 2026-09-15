//! [`BucketChange`], one finding present in both runs whose bucket moved.

use super::finding_bucket::FindingBucket;
use super::suppressed_because::SuppressedBecause;
use super::suppression_standing::SuppressionStanding;
use nomos_contracts::{RuleId, SubjectId};
use nomos_gate_orchestration::DispositionChange;
use serde::Serialize;

/// One finding present in both runs whose bucket moved.
///
/// Carries both ends rather than a single "changed" flag: a finding that moved from blocking
/// to baselined and one that moved the other way are opposite facts about a build, and a
/// caller that cannot tell them apart learns nothing from being told something moved.
#[derive(Debug, Serialize)]
pub struct BucketChange
{
    /// The rule that produced the finding on both sides.
    pub rule: RuleId,
    /// The subject it was produced about, the identity the two runs are matched by.
    pub subject: SubjectId,
    /// That subject's readable name.
    pub subject_name: String,
    /// Which bucket it fell into in the baseline run.
    pub before: FindingBucket,
    /// Which bucket it falls into in the candidate run.
    pub after: FindingBucket,
    /// Why it was in `before`'s bucket, when a disposition named it.
    ///
    /// Without this a headless caller sees a bucket that did not move and concludes nothing
    /// changed, when a `false_positive_disposition` may have become a
    /// `formal_risk_acceptance` -- the same treatment, an opposite engineering claim.
    pub before_reason: Option<SuppressedBecause>,
    /// Why it is in `after`'s bucket, when a disposition names it.
    pub after_reason: Option<SuppressedBecause>,
    /// Whether `before_reason` still applied when the baseline run was judged.
    pub before_standing: Option<SuppressionStanding>,
    /// Whether `after_reason` still applies. `expired` beside a `blocking` bucket is a
    /// tolerance that came due rather than a violation that appeared.
    pub after_standing: Option<SuppressionStanding>,
}

impl BucketChange
{
    pub(crate) fn From(change: DispositionChange) -> Self
    {
        return Self {
            rule: change.rule,
            subject: change.subject,
            subject_name: change.subject_name,
            before: FindingBucket::From(change.before),
            after: FindingBucket::From(change.after),
            before_reason: change.before_reason.map(|reason| return SuppressedBecause::From(reason.disposition)),
            after_reason: change.after_reason.map(|reason| return SuppressedBecause::From(reason.disposition)),
            before_standing: change.before_reason.map(|reason| return SuppressionStanding::From(reason.status)),
            after_standing: change.after_reason.map(|reason| return SuppressionStanding::From(reason.status)),
        };
    }
}
