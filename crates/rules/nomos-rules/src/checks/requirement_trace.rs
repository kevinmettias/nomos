//! `requirement-trace-staleness` — the one rule that reads `nomos.cap.requirement.trace`.
//!
//! # A second rule whose subject is not source
//!
//! [`crate::Check_Goals_And_Parts_Line_Up`] is the only other rule in this crate handed
//! nothing but a [`FactReader`]; its own module doc explains why that is not an oversight
//! against `OD-RULES-001`. This rule's subject is the identical shape: `OD-TRACE-001`'s
//! whole guard is a comparison between a repository's own committed declaration (which
//! requirement each `tests/contract/requirements/*.assessment` entry claims is satisfied,
//! and where) and the real tree, and neither half is in any one file `sources` would hand
//! this rule — the declaration is a corpus of many files, the tree it is compared against
//! is the whole workspace, and the comparison itself needs a
//! [`nomos_platform::FileSystem`] this crate (Rules zone) may not depend on. So both halves
//! of the judgment already happened by the time this rule runs, inside
//! `nomos_cap_requirement_trace::Materialize_Workspace`, and this rule's whole job is the
//! [`crate::Check_Lint_Diagnostics`]/[`crate::Check_Review_Findings`] shape: relay an
//! already-reached verdict as `Finding`s, one per [`nomos_cap_requirement_trace::Problem`],
//! rather than judge a second time.
//!
//! # What "nothing to report" means here
//!
//! An empty payload means one of three things this rule cannot and need not distinguish:
//! this repository has no `tests/contract/requirements/` directory at all (every repository
//! this rule judges except this one, today), the directory exists and holds no entry, or
//! every committed entry resolves. `nomos_cap_requirement_trace::Discover_Workspace`'s own
//! module doc names the same collapse and why: `OD-ANALYSIS-012` decided a rule that judged
//! an empty population deserves a report apart from one that judged a real population
//! clean, but built no mechanism for it yet, so this rule follows the identical
//! "an absent or empty optional capability judges nothing" idiom [`crate::
//! Check_Goals_And_Parts_Line_Up`] already uses for a repository that declared no goals.

use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_requirement_trace::{Problem, ProblemKind, RequirementTracePayload, REGISTRY};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier.
pub const REQUIREMENT_TRACE_STALENESS: &str = "requirement-trace-staleness";

/// The record this implementation's contract is written against. `OD-TRACE-001` already
/// decided the assessment format this rule enforces and the obligation that a stale
/// citation must be visible rather than indistinguishable from a resolved one;
/// `tests/contract/tests/rule_contract_citation.rs` reads that record's own front matter on
/// every run and compares it against [`REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION`],
/// so an amendment this implementation has not caught up to is a red test rather than
/// silent drift.
pub const REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD: &str = "OD-TRACE-001";

/// The version of [`REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD`] this implementation was
/// written against.
pub const REQUIREMENT_TRACE_STALENESS_CONTRACT_RECORD_VERSION: u32 = 1;

/// Reports every stale or incomplete entry `nomos.cap.requirement.trace` names: an
/// unresolved site, an unresolved gap, an unresolved record, a divergence with no record,
/// or a partial with no gap — one `Finding` per [`Problem`], never collapsed, since two
/// stale citations in one assessment are two separate facts a reader needs to see both of.
///
/// Judges nothing when the fact is unavailable or reports zero problems. Both are the same
/// answer and neither is itself a `Finding` — see this module's own doc.
#[must_use]
pub fn Check_Requirement_Trace_Staleness(facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(payload) = Materialized_Trace(facts)
    else
    {
        return Vec::new();
    };

    return Findings_For(&payload);
}

fn Materialized_Trace(facts: &mut dyn FactReader) -> Option<RequirementTracePayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_requirement_trace::Capability(), &subject, InputDigest::Of(&[]), &Requirement_Trace_Requirement())
        .ok()?;

    return nomos_cap_requirement_trace::Parse_Payload(&fact.payload.bytes).ok();
}

/// This crate's own floor for `nomos.cap.requirement.trace` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this rule could
/// honestly still act on.
fn Requirement_Trace_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_requirement_trace::Capability(),
        nomos_cap_requirement_trace::CONTRACT_VERSION,
        nomos_cap_requirement_trace::Ceiling(),
    );
}

fn Findings_For(payload: &RequirementTracePayload) -> Vec<Finding>
{
    return payload.problems.iter().map(Finding_For_Problem).collect();
}

fn Finding_For_Problem(problem: &Problem) -> Finding
{
    return Finding {
        rule: RuleId::New(REQUIREMENT_TRACE_STALENESS),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: format!("{}:{}", Rank(problem.kind), problem.requirement),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: problem.message.clone(),
        locations: vec![Assessment_File(&problem.requirement)],
    };
}

/// The committed entry a problem's own requirement points back to — the file a reader
/// fixes, since every problem this rule reports is about a stale or incomplete
/// *declaration*, not about the site, gap or record it names (which may not even exist any
/// more).
fn Assessment_File(requirement: &str) -> String
{
    return format!("{REGISTRY}/{requirement}.assessment");
}

/// The sort rank [`crate::Check_Goals_And_Parts_Line_Up`]'s own `Kind::Rank` states for the
/// identical reason: kept out of the subject text so two kinds never interleave, and fixed
/// at the granularity `nomos_cap_requirement_trace::payload`'s own module doc already
/// orders `RequirementTracePayload::problems` in, so this never needs to re-sort what the
/// provider already produced in order.
const fn Rank(kind: ProblemKind) -> u8
{
    return match kind
    {
        ProblemKind::UnresolvedSite => 0,
        ProblemKind::UnresolvedGap => 1,
        ProblemKind::UnresolvedRecord => 2,
        ProblemKind::DivergenceWithNoRecord => 3,
        ProblemKind::PartialWithNoGap => 4,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_requirement_trace::{Encode_Payload, Payload_Schema};

    #[test]
    fn Test_Check_Requirement_Trace_Staleness_Should_Judge_Nothing_When_The_Fact_Is_Unavailable()
    {
        let store = MemoryFactStore::New();
        let registry = nomos_capability::Registry::New();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let findings = Check_Requirement_Trace_Staleness(&mut reader);

        assert!(findings.is_empty(), "an unmaterialized fact must judge nothing, not report an absence: {findings:?}");
    }

    #[test]
    fn Test_Check_Requirement_Trace_Staleness_Should_Judge_Nothing_Over_An_Empty_Payload()
    {
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize(&mut store, &offer, &RequirementTracePayload::default());

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Requirement_Trace_Staleness(&mut reader);

        assert!(findings.is_empty(), "zero problems reported must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_Check_Requirement_Trace_Staleness_Should_Report_One_Finding_Per_Problem()
    {
        let TestOffering { mut store, registry, offer } = Offering();
        let payload = RequirementTracePayload {
            problems: vec![
                Problem {
                    kind: ProblemKind::UnresolvedSite,
                    requirement: "CHK-003".to_owned(),
                    message: "CHK-003: site crates/x.rs is not a file in this workspace".to_owned(),
                },
                Problem {
                    kind: ProblemKind::UnresolvedRecord,
                    requirement: "CAP-002".to_owned(),
                    message: "CAP-002: OD-NOTHING-999 has no registration under crates/spec/nomos-spec-store/records/".to_owned(),
                },
            ],
        };
        Materialize(&mut store, &offer, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Requirement_Trace_Staleness(&mut reader);

        assert_eq!(findings.len(), 2, "{findings:?}");
        let first = findings.first().expect("asserted len 2 above");
        assert_eq!(first.rule, RuleId::New(REQUIREMENT_TRACE_STALENESS));
        assert_eq!(first.applicability, Applicability::Supported);
        assert_eq!(first.evidence, EvidenceClass::Derived);
        assert_eq!(first.gate, GateCategory::Blocking);
        assert_eq!(first.summary, "CHK-003: site crates/x.rs is not a file in this workspace");
        assert_eq!(first.locations, vec!["tests/contract/requirements/CHK-003.assessment".to_owned()]);

        let second = findings.get(1).expect("asserted len 2 above");
        assert_eq!(second.locations, vec!["tests/contract/requirements/CAP-002.assessment".to_owned()]);
    }

    #[test]
    fn Test_Check_Requirement_Trace_Staleness_Should_Not_Collapse_Two_Problems_From_One_Requirement()
    {
        let TestOffering { mut store, registry, offer } = Offering();
        let payload = RequirementTracePayload {
            problems: vec![
                Problem {
                    kind: ProblemKind::UnresolvedSite,
                    requirement: "CHK-003".to_owned(),
                    message: "CHK-003: site a.rs is not a file in this workspace".to_owned(),
                },
                Problem {
                    kind: ProblemKind::UnresolvedSite,
                    requirement: "CHK-003".to_owned(),
                    message: "CHK-003: site b.rs is not a file in this workspace".to_owned(),
                },
            ],
        };
        Materialize(&mut store, &offer, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Requirement_Trace_Staleness(&mut reader);

        assert_eq!(findings.len(), 2, "two stale citations from one assessment are two reportable facts: {findings:?}");
    }

    const PROVIDER: &str = "nomos.test.requirement.trace.resolves";

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_requirement_trace::Capability_Contract(),
            nomos_cap_requirement_trace::Capability(),
            nomos_cap_requirement_trace::CONTRACT_VERSION,
            PROVIDER,
            nomos_cap_requirement_trace::Ceiling(),
        );
    }

    fn Materialize(store: &mut MemoryFactStore, offer: &nomos_capability::ProviderOffer, payload: &RequirementTracePayload)
    {
        let bytes = Encode_Payload(payload);
        test_support::Materialize(store, nomos_model::Subject_Of_Path(""), offer, InputDigest::Of(&[]), Payload_Schema(), bytes);
    }
}
