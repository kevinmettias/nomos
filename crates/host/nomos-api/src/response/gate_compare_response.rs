//! [`Handle_Gate_Compare`] and its own [`GateCompareResponse`], paired in one file: the
//! response type exists only for this one handler, the same "handler beside its own response"
//! locality every other endpoint file in this crate keeps.
//!
//! # What this answers that the other three do not
//!
//! `Handle_Gate_Run` answers what a tree looks like now. This answers what changed, which is
//! the question a gate exists to answer over time rather than once: which findings are new,
//! which are gone, and which changed bucket because a policy moved rather than because the
//! code did. `ARC-ROADMAP-001`'s constraint 5 names `compare` as one of the product Gate's
//! four verbs, and `IF-001` requires every user-visible action to have a canonical
//! application service; until this handler existed, `compare` had one only in a terminal.
//!
//! # Why it re-derives both runs rather than looking either up
//!
//! `OD-GATE-022-A` decided exactly that: a compare caller re-derives both runs inside one
//! process, builds no run-history store, and serializes no `GateRunResult`. Both cases the
//! comparison is for are same-process cases -- two trees under one policy, or one tree under
//! two policies -- and neither needs a value to outlive the process that produced it.
//! Comparing against a run some earlier invocation produced is real and deferred, and would
//! need either a persisted history keyed by `RunId` or a round-trippable twin; that record
//! says which, and says it is not today's case.

use crate::{composition, sources};
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_contracts::{Finding, RuleId, RunId, SubjectId};
use nomos_gate_orchestration::{
    DispositionChange, FindingDisposition, GateCommand, GateCompareResult, GateRunResult, SuppressionDisposition, SuppressionReason,
    SuppressionStatus,
};
use nomos_platform::Clock;
use serde::Serialize;

/// Judges `baseline.root` and `candidate.root` exactly as `nomos gate run` would, and hands
/// back what changed between them as a JSON-serializable [`GateCompareResponse`].
///
/// Each side is a whole [`GateCommand`], not a shared one over two roots, because the
/// policy comparison `OD-GATE-022-A` names -- one tree under two policies -- varies
/// `suppressions`, `baseline`, `rules` or `adoption` between the two sides rather than the
/// tree. A caller wanting the code comparison instead passes the same policy twice and two
/// roots. Both are the same call.
#[must_use]
pub fn Handle_Gate_Compare(baseline: &GateCommand, candidate: &GateCommand) -> GateCompareResponse
{
    let before = Judged(baseline);
    let after = Judged(candidate);

    return GateCompareResponse::From(nomos_gate_orchestration::Compare_Gate_Runs(&before, &after));
}

/// One walk-and-judge of `command.root`, under a freshly minted `RunId`.
///
/// Each side gets its own id from the same clock: two walks in one process are still two
/// executions, which is what `GateRunResult::run` distinguishes. Sequential rather than
/// concurrent -- `Run_Gate` is the expensive part and nothing here waits on I/O it could
/// overlap. This is deliberately the same composition `gate_run_response.rs` performs and
/// `nomos-cli`'s own `gate.rs` performs; unifying the three is a change to that file rather
/// than this one.
fn Judged(command: &GateCommand) -> GateRunResult
{
    let walked = sources::Walked_Sources(&command.root);
    let now = CLOCK.Now();
    let run = nomos_gate_orchestration::Fresh_Run_Id(now);

    return nomos_gate_orchestration::Run_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment {
            variant: composition::Host_Variant(),
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            now,
        },
        command,
        run,
    );
}

/// A serializable twin of [`nomos_gate_orchestration::FindingDisposition`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason `crate::response`'s own doc gives, and because `OD-GATE-022-A` refuses to
/// give it one on a caller's behalf. Kept to the same four variants, in the same order, so a
/// mismatch is a compile error in [`FindingBucket::From`] rather than a silent divergence --
/// the discipline [`super::Disposition`] already uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingBucket
{
    /// The finding failed the build.
    Blocking,
    /// An `AdoptionPolicy` calibration kept it from blocking.
    Calibrated,
    /// A `Suppression` kept it from blocking.
    Suppressed,
    /// A `BaselineDebt` entry kept it from blocking.
    Baselined,
}

impl FindingBucket
{
    fn From(disposition: FindingDisposition) -> Self
    {
        return match disposition
        {
            FindingDisposition::Blocking => Self::Blocking,
            FindingDisposition::Calibrated => Self::Calibrated,
            FindingDisposition::Suppressed => Self::Suppressed,
            FindingDisposition::Baselined => Self::Baselined,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::SuppressionDisposition`].
///
/// A twin for the reason [`FindingBucket`] is one. Six variants in the same order, so a
/// reordering or a dropped arm is a compile error in [`SuppressedBecause::From`] rather than
/// the wrong word on a wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressedBecause
{
    /// Suppressed at the finding's own site.
    InlineSuppression,
    /// Exempted by a repository-wide policy.
    RepositoryPolicyException,
    /// Accepted for a bounded period, and carrying an end date.
    TemporaryWaiver,
    /// Existing debt a baseline tolerates.
    AcceptedBaselineDebt,
    /// The finding does not hold.
    FalsePositiveDisposition,
    /// A deliberate, owned decision to accept the risk.
    FormalRiskAcceptance,
}

impl SuppressedBecause
{
    fn From(disposition: SuppressionDisposition) -> Self
    {
        return match disposition
        {
            SuppressionDisposition::InlineSuppression => Self::InlineSuppression,
            SuppressionDisposition::RepositoryPolicyException => Self::RepositoryPolicyException,
            SuppressionDisposition::TemporaryWaiver => Self::TemporaryWaiver,
            SuppressionDisposition::AcceptedBaselineDebt => Self::AcceptedBaselineDebt,
            SuppressionDisposition::FalsePositiveDisposition => Self::FalsePositiveDisposition,
            SuppressionDisposition::FormalRiskAcceptance => Self::FormalRiskAcceptance,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::SuppressionStatus`].
///
/// Two variants because two is the whole population: a third, for a disposition stale by rule
/// version, subject identity, evidence or scope, is named in that type's own documentation and
/// deliberately not invented before something can construct it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionStanding
{
    /// The disposition applied when the run was judged.
    Active,
    /// It named an end date and the run was at or past it.
    Expired,
}

impl SuppressionStanding
{
    fn From(status: SuppressionStatus) -> Self
    {
        return match status
        {
            SuppressionStatus::Active => Self::Active,
            SuppressionStatus::Expired => Self::Expired,
        };
    }
}

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
    fn From(change: DispositionChange) -> Self
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

/// What changed between two real gate runs, in a shape `serde_json` can hand across a wire.
///
/// Three distinct lists rather than a count or a boolean. New debt, resolved debt and a
/// finding whose bucket moved are three different things to a caller deciding whether a
/// change is admissible, and collapsing them is the behaviour that makes a gate indist-
/// inguishable from a linter runner.
#[derive(Debug, Serialize)]
pub struct GateCompareResponse
{
    /// The identity of the run compared against.
    pub baseline: RunId,
    /// The identity of the run being compared.
    pub candidate: RunId,
    /// Present in the candidate, absent from the baseline -- by rule and subject identity.
    pub added: Vec<Finding>,
    /// Present in the baseline, absent from the candidate.
    pub removed: Vec<Finding>,
    /// Present in both, in different buckets.
    pub changed: Vec<BucketChange>,
}

impl GateCompareResponse
{
    pub(crate) fn From(result: GateCompareResult) -> Self
    {
        return Self {
            baseline: result.baseline,
            candidate: result.candidate,
            added: result.added,
            removed: result.removed,
            changed: result.changed.into_iter().map(BucketChange::From).collect(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    /// [`GateCommand`] over `root`, every selector at its select-everything default.
    fn Command_At(root: &str) -> GateCommand
    {
        return GateCommand { root: PathBuf::from(root), ..Default::default() };
    }

    /// Comparing a tree with itself under one policy reports nothing changed.
    ///
    /// The identity case, and the one that would catch a comparison matching findings by
    /// something that varies between two runs of the same thing -- a `RunId`, a timestamp, a
    /// vector index. Both sides are real walks of this crate's own tree, not fixtures.
    #[test]
    fn Test_Comparing_A_Tree_With_Itself_Should_Report_No_Change()
    {
        let response = Handle_Gate_Compare(&Command_At("."), &Command_At("."));

        assert!(
            response.added.is_empty() && response.removed.is_empty() && response.changed.is_empty(),
            "comparing a tree with itself reported {} added, {} removed and {} changed. Two \
             runs of the same tree under the same policy answer the same question, so a \
             difference here is the comparison matching on something that is not the \
             finding's identity.",
            response.added.len(),
            response.removed.len(),
            response.changed.len()
        );
    }

    /// The two sides are distinct executions.
    ///
    /// `RunId` is what tells a caller which side a reported difference came from, so two
    /// walks sharing one id would make the answer unreadable even when it is correct.
    #[test]
    fn Test_The_Two_Sides_Should_Be_Distinct_Runs()
    {
        let response = Handle_Gate_Compare(&Command_At("."), &Command_At("."));

        assert_ne!(
            response.baseline, response.candidate,
            "both sides reported the same RunId, so nothing in the response says which run a \
             difference belongs to"
        );
    }

    /// Every bucket maps to its own twin variant.
    ///
    /// The twin exists to be exhaustive; this is what makes a reordering or a dropped arm
    /// fail here rather than serialize as the wrong word.
    #[test]
    fn Test_Every_Disposition_Should_Map_To_Its_Own_Bucket()
    {
        assert_eq!(FindingBucket::From(FindingDisposition::Blocking), FindingBucket::Blocking);
        assert_eq!(FindingBucket::From(FindingDisposition::Calibrated), FindingBucket::Calibrated);
        assert_eq!(FindingBucket::From(FindingDisposition::Suppressed), FindingBucket::Suppressed);
        assert_eq!(FindingBucket::From(FindingDisposition::Baselined), FindingBucket::Baselined);
    }

    /// A bucket change serializes with both ends, under the names a wire caller reads.
    #[test]
    fn Test_A_Bucket_Change_Should_Serialize_Both_Ends()
    {
        let change = BucketChange {
            rule: RuleId::New("check-naming-convention"),
            subject: SubjectId::From_Digest(nomos_model::Content_Digest(b"src/lib.rs")),
            subject_name: "src/lib.rs".to_string(),
            before: FindingBucket::Blocking,
            after: FindingBucket::Baselined,
            before_reason: None,
            after_reason: Some(SuppressedBecause::TemporaryWaiver),
            before_standing: None,
            after_standing: Some(SuppressionStanding::Expired),
        };

        let rendered = serde_json::to_string(&change).expect("a BucketChange serializes");

        assert!(
            rendered.contains("\"before\":\"blocking\"")
                && rendered.contains("\"after\":\"baselined\"")
                && rendered.contains("\"after_reason\":\"temporary_waiver\"")
                && rendered.contains("\"after_standing\":\"expired\""),
            "a bucket change must carry both ends in snake_case, and rendered as {rendered}"
        );
    }
}
