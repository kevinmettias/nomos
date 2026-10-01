//! [`ExportedFinding`], one finding as the line and table exports carry it.

use crate::sarif::{Bucket_Word, Bucketed_Findings, SarifSuppression};
use crate::{FindingDisposition, GateRunResult};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use serde::Serialize;

/// One finding, with what a policy did about it in the words the SARIF log uses.
///
/// `bucket` and `suppression` are read from [`Bucket_Word`] and [`SarifSuppression::Of`], the
/// functions the SARIF log's own results are built from, so a line, a table row and a SARIF
/// result for one finding name the same bucket and the same justification. Both are absent for
/// a check run, which applies no policy.
///
/// Field order is the order a line serializes and the order rows sort by, so an export is a
/// function of what was found rather than of the order it was found in.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportedFinding
{
    /// The rule that found it.
    pub(crate) rule: String,
    /// What the finding is about, as the finding names it.
    pub(crate) subject_name: String,
    /// Every location the finding reports, in its own order.
    pub(crate) locations: Vec<String>,
    /// The bucket a gate run placed it in, or `None` for a check run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bucket: Option<&'static str>,
    /// What kept it from blocking, for a bucket a policy answered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) suppression: Option<ExportedSuppression>,
    /// `GateCategory`'s own label.
    pub(crate) gate: &'static str,
    /// `Applicability`'s own label.
    pub(crate) applicability: &'static str,
    /// `EvidenceClass`'s own label.
    pub(crate) evidence: &'static str,
    /// The finding's own summary.
    pub(crate) summary: String,
}

/// The suppression a finding in a policy-answered bucket carries: the SARIF log's own kind and
/// justification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct ExportedSuppression
{
    /// Always SARIF's `external`, since every policy here lives outside the judged file.
    pub(crate) kind: &'static str,
    /// Which policy kept the finding from blocking, naming its bucket first.
    pub(crate) justification: &'static str,
}

impl ExportedFinding
{
    /// `finding`, placed in `bucket` or in none.
    fn Of(finding: &Finding, bucket: Option<FindingDisposition>) -> Self
    {
        return Self {
            rule: finding.rule.As_Str().to_owned(),
            subject_name: finding.subject_name.clone(),
            locations: finding.locations.clone(),
            bucket: bucket.map(Bucket_Word),
            suppression: bucket
                .and_then(SarifSuppression::Of)
                .map(|suppression| return ExportedSuppression { kind: suppression.kind, justification: suppression.justification }),
            gate: finding.gate.Label(),
            applicability: finding.applicability.Label(),
            evidence: finding.evidence.Label(),
            summary: finding.summary.clone(),
        };
    }
}

/// Every finding a gate run reduced, in every group, each in the bucket the run placed it in,
/// sorted.
pub(crate) fn Gate_Run_Rows(result: &GateRunResult) -> Vec<ExportedFinding>
{
    let mut rows: Vec<ExportedFinding> = Bucketed_Findings(&result.findings)
        .into_iter()
        .flat_map(|(bucket, group)| return group.iter().map(move |finding| return ExportedFinding::Of(finding, Some(bucket))))
        .collect();
    rows.sort();

    return rows;
}

/// Every finding a check judged, with no bucket, sorted; nothing for a check that judged
/// nothing, whose reason is the run status's to state.
pub(crate) fn Check_Run_Rows(outcome: &CheckOutcome) -> Vec<ExportedFinding>
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Vec::new();
    };

    let mut rows: Vec<ExportedFinding> = findings.iter().map(|finding| return ExportedFinding::Of(finding, None)).collect();
    rows.sort();

    return rows;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At, Judged_Outcome_With};
    use nomos_check_orchestration::Claim;

    /// Every one of the six groups reaches the rows, each under its own bucket word, and every
    /// bucket a policy answered carries the SARIF log's own justification for it.
    #[test]
    fn Test_Gate_Run_Rows_Should_Carry_Every_Group_With_Its_Bucket_And_Suppression()
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At("a-rule", "a.rs:1"));
        findings.calibrated_findings.push(Finding_At("b-rule", "a.rs:1"));
        findings.suppressed_findings.push(Finding_At("c-rule", "a.rs:1"));
        findings.baselined_findings.push(Finding_At("d-rule", "a.rs:1"));
        findings.baseline_exceeded_findings.push(Finding_At("e-rule", "a.rs:1"));
        findings.below_evidence_floor_findings.push(Finding_At("f-rule", "a.rs:1"));

        let rows = Gate_Run_Rows(&Complete_Gate_Result(findings));

        let buckets: Vec<(&str, Option<&str>)> = rows
            .iter()
            .map(|row| return (row.bucket.unwrap_or("none"), row.suppression.map(|suppression| return suppression.justification)))
            .collect();
        assert_eq!(buckets.len(), 6, "{rows:?}");
        for (bucket, justification) in buckets
        {
            let is_answered = matches!(bucket, "calibrated" | "suppressed" | "baselined" | "below_evidence_floor");
            assert_eq!(justification.is_some(), is_answered, "{bucket}: {justification:?}");
            assert!(justification.is_none_or(|text| return text.starts_with(bucket)), "{bucket}: {justification:?}");
        }
    }

    /// A check applies no policy, so its rows carry neither a bucket nor a suppression.
    #[test]
    fn Test_Check_Run_Rows_Should_Carry_No_Bucket_And_No_Suppression()
    {
        let rows = Check_Run_Rows(&Judged_Outcome_With(vec![Finding_At("a-rule", "a.rs:1")], Claim::Complete));

        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(rows.iter().all(|row| return row.bucket.is_none() && row.suppression.is_none()), "{rows:?}");
    }
}
