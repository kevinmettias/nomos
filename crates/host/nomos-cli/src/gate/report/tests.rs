//! [`super::Render_Plan`], [`super::Render_Run`], [`super::Render_Compare`] and
//! [`super::Render_Explain`], exercised.
//!
//! Split out of `report.rs` itself once that file passed the ~500-line review trigger -- the
//! same split this workspace already keeps between `gate.rs` and `gate/tests.rs`. The fixtures
//! more than one subject shares are here; each subject's own are in the submodule named for it.

mod baselines;
mod compare;
mod explain;
mod plan;
mod policy;
mod verdicts;

use nomos_check_orchestration::{CheckOutcome, Claim, Examined, SupportingFactTrail};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use nomos_gate_orchestration::{
    Fresh_Run_Id, GateFindings, GateRunOutcome, GateRunProvenance, GateRunResult,
};
use nomos_platform::Timestamp;
use std::path::PathBuf;

/// The byte an example finding's subject digest is filled with. Any byte would do; naming it
/// says so, rather than leaving a reader to wonder what `9` was chosen for.
const EXAMPLE_SUBJECT_FILL: u8 = 9;

/// A judged run that found nothing, carrying `provenance` -- the only thing the
/// comparability rendering reads, so every test below differs in that alone.
fn Judged_With(fill: u8, provenance: Option<GateRunProvenance>) -> GateRunResult
{
    return GateRunResult {
        provenance,
        // These tests are about rendering, and say nothing about which layer stated the
        // policy this run was judged under.
        policy: None,
        no_verdict: None,
        unmatched_policy: Vec::new(),
        run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(i64::from(fill))),
        root: PathBuf::from("."),
        check_outcome: CheckOutcome::Judged { findings: Vec::new(), examined: Examined { files: 1, facts: 1 }, claim: Claim::Complete, supporting_facts: SupportingFactTrail::New() },
        findings: Empty_Findings(),
        disposition: GateRunOutcome::Passed,
    };
}

/// One real finding, distinguishable from another only by the gate category a caller
/// passes in -- everything else about it is incidental to what these tests check.
fn Example_Finding(gate: GateCategory) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New("unread-reaches-finding"),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([EXAMPLE_SUBJECT_FILL; Digest128::BYTE_LENGTH])),
        subject_name: "Example::Subject".to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate,
        summary: "reaches an unread item".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// No finding blocked, calibrated, suppressed or baselined -- the starting point every
/// test above that does not care about one of these buckets builds on.
fn Empty_Findings() -> GateFindings
{
    return GateFindings {
        blocking_findings: Vec::new(),
        calibrated_findings: Vec::new(),
        suppressed_findings: Vec::new(),
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        suppression_reasons: std::collections::BTreeMap::default(),
    };
}
