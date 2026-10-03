//! Constructed judgments the projection's own tests state rather than run.
//!
//! Constructed rather than run, deliberately: a test about how a judgment *projects* must be
//! able to state the judgment it is projecting, including the combinations a real run over a
//! scratch tree cannot be made to produce on demand -- a judged-but-incomplete claim, a
//! malformed policy, one finding in each of six buckets. The one test about the path a caller
//! takes runs a real gate instead, in `sarif_log`.

use crate::{GateFindings, GateRunOutcome, GateRunResult};
use nomos_check_orchestration::{CheckOutcome, Claim, Examined, SupportingFactTrail};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, RunId, SubjectId};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The byte every fixture digest is filled with.
const FIXTURE_DIGEST_BYTE: u8 = 7;

/// The summary every fixture finding carries.
const FIXTURE_SUMMARY: &str = "a real summary";

/// How many files and facts a judged fixture reports having examined.
const FIXTURE_EXAMINED: usize = 1;

/// A blocking, supported, derived finding from `rule` at `location`, whose subject name is the
/// location's path half.
pub(crate) fn Finding_At(rule: &str, location: &str) -> Finding
{
    let subject_name = match location.split_once(':')
    {
        Some((path, _line)) => path,
        None => location,
    };

    return Finding {
        rule: RuleId::New(rule),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
        subject_name: subject_name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: FIXTURE_SUMMARY.to_owned(),
        locations: vec![location.to_owned()],
        address: None,
    };
}

/// A gate run's finding groups, every one empty.
pub(crate) fn Empty_Gate_Findings() -> GateFindings
{
    return GateFindings {
        blocking_findings: Vec::new(),
        calibrated_findings: Vec::new(),
        suppressed_findings: Vec::new(),
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        // The projection reads no suppression reason, so none is stated.
        suppression_reasons: BTreeMap::new(),
    };
}

/// A gate run that judged its tree completely and reached a verdict, carrying `findings`.
pub(crate) fn Complete_Gate_Result(findings: GateFindings) -> GateRunResult
{
    return GateRunResult {
        run: RunId::From_Digest(Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
        root: PathBuf::from("."),
        check_outcome: Judged_Outcome(Claim::Complete),
        findings,
        disposition: GateRunOutcome::Passed,
        unmatched_policy: Vec::new(),
        no_verdict: None,
        // A result a test states rather than runs says nothing about what produced it or
        // which layer decided its policy, so it reports neither rather than an invented one.
        provenance: None,
        policy: None,
    };
}

/// A check that judged its tree and found nothing, under `claim`.
pub(crate) fn Judged_Outcome(claim: Claim) -> CheckOutcome
{
    return Judged_Outcome_With(Vec::new(), claim);
}

/// A check that judged its tree and found `findings`, under `claim`.
pub(crate) fn Judged_Outcome_With(findings: Vec<Finding>, claim: Claim) -> CheckOutcome
{
    return CheckOutcome::Judged {
        findings,
        examined: Examined { files: FIXTURE_EXAMINED, facts: FIXTURE_EXAMINED },
        claim,
        supporting_facts: SupportingFactTrail::New(),
        populations: nomos_check_orchestration::Populations::New(),
    };
}

/// A check stopped by a real registry refusal: the standard offers applied twice offer every
/// provider twice, which `nomos_capability::Registry` refuses on the first repeat. Real rather
/// than constructed because a `RegistryError` has no public constructor, and a refusal the
/// registry actually gives is a better fixture than one this module could invent.
pub(crate) fn Contradictory_Outcome() -> CheckOutcome
{
    let refused = nomos_check_orchestration::Registered(|registry| {
        nomos_composer_providers::Standard_Offers(registry)?;
        return nomos_composer_providers::Standard_Offers(registry);
    });

    return match refused
    {
        Err(error) => CheckOutcome::Contradictory(error),
        Ok(_) => panic!("offering every standard provider twice must be refused by the registry"),
    };
}
