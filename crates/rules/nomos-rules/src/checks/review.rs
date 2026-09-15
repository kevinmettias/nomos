//! One external code review tool's own observed findings, relayed as `Finding`s.
//!
//! `OD-RULES-010` decided a `ToolProvider`'s output is a fact a native rule judges, not a
//! `Finding` the tool emits directly. This is that rule for `nomos.cap.review.finding`:
//! since a review comment already carries the reviewing tool's own verdict (a severity, a
//! category, a message), this rule's own judgment is a 1:1 relay rather than a second
//! opinion — the same `Require`-then-decode-then-emit shape [`crate::Check_Lint_Diagnostics`]
//! and [`crate::Check_Dependency_Policy`] already use, extended from a same-process
//! `ToolProvider` to a connector under `ARC-CONNECTOR-001`: `OD-EXECUTOR-006` measured that
//! the boundary that actually matters here is `OD-RULES-010`'s fact-not-finding split, not
//! the transport a fact arrived through.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! Behind `OD-GATE-017`'s own per-call rule subset, the same `Wants(selected, ...)` gate
//! `LINT_DIAGNOSTICS` and `DEPENDENCY_POLICY` already sit behind — wiring this rule in is
//! `nomos-check-orchestration`'s own territory, not this crate's.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_capability::Requirement;
use nomos_connector_coderabbit::FindingPayload;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const REVIEW_FINDING: &str = "review-finding";

/// The record this implementation's contract is written in -- `OD-RULES-010` decided a
/// `ToolProvider`'s output is a fact a native rule judges, and `OD-EXECUTOR-006` measured
/// that a connector's own review evidence is the identical fact-not-finding split applied
/// to a peer system reached over its own transport rather than a local subprocess.
/// `tests/contract/tests/rule_contract_citation.rs` reads `OD-RULES-010`'s own front matter
/// on every run and compares it against [`REVIEW_CONTRACT_RECORD_VERSION`], the same
/// `nomos_rules::LINT_CONTRACT_RECORD` shape, so an amendment this implementation has not
/// caught up to is a red test rather than silent drift.
pub const REVIEW_CONTRACT_RECORD: &str = "OD-RULES-010";

/// The version of [`REVIEW_CONTRACT_RECORD`] this implementation was written against.
pub const REVIEW_CONTRACT_RECORD_VERSION: u32 = 2;

/// Judges every `nomos.cap.review.finding` fact `sources` names against the reviewing
/// tool's own reported severity, category and message.
///
/// One fact per source, the same shape [`crate::Check_Lint_Diagnostics`] reads — a source
/// here is the placeholder subject a connector fact files under today
/// (`nomos_model::Subject_Of_Path("")`, per `ARC-CONNECTOR-001`'s "What This Record Does
/// Not Do"), not a file or a workspace member; its `subject` is the one the provider keyed
/// its fact under, and its `text` is unread.
#[must_use]
pub fn Check_Review_Findings(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return super::Relay_Findings(sources, facts, Payload_Of, Findings_Of);
}

/// One source's decoded review-finding payload, or a finding reporting why it could not be
/// read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<FindingPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this source's review-finding fact, turning an inadmissible answer into an
/// [`Unread`] finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Review_Requirement();
    let capability = nomos_connector_coderabbit::Capability();
    let inputs = InputDigest::Of(&[]);

    return match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread_Finding(
            source,
            applicability,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// What this rule needs from `nomos.cap.review.finding` before it will believe an answer —
/// the same "nothing weaker could be trusted" reasoning
/// [`crate::checks::lint::Lint_Requirement`] gives, stated at the exact guarantee this
/// capability's one real provider declares: a review finding this rule cannot attribute to
/// a live, observed vendor read is not one it should relay as if the reviewing tool itself
/// vouched for it.
#[must_use]
fn Review_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::RuntimeObserved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::None,
    );

    return Requirement::New(nomos_connector_coderabbit::Capability(), nomos_connector_coderabbit::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_connector_coderabbit::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this subject carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`FindingPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<FindingPayload, Finding>
{
    return nomos_connector_coderabbit::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`, relayed as exactly one `Finding` — never judged a second time, the whole
/// point `OD-RULES-010` decided: the reviewing tool already reached the verdict this rule
/// reports. Always one finding per payload, unlike [`crate::Check_Lint_Diagnostics`]'s
/// per-diagnostic list: this capability's `IncrementalGranularity::None` ceiling never
/// produces more than one finding's own fields per fact.
fn Findings_Of(source: &SourceFile, payload: &FindingPayload) -> Vec<Finding>
{
    return vec![Finding_For_Payload(source, payload)];
}

fn Finding_For_Payload(source: &SourceFile, payload: &FindingPayload) -> Finding
{
    return Finding {
        rule: RuleId::New(REVIEW_FINDING),
        subject: source.subject,
        subject_name: payload.external_id.As_Str().to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Observed,
        gate: GateCategory::Advisory,
        summary: Summary_Of(payload),
        locations: vec![payload.path.clone()],
    };
}

/// `payload`, rendered as one line — the reviewing tool's own severity, category and
/// message, with the vendor's own words kept rather than translated into this workspace's
/// vocabulary: `ARC-CONNECTOR-001`'s second invariant permits vendor identity as data, and
/// [`nomos_connector_coderabbit::translation`]'s own module doc already declines to force
/// either word into a fixed enum this rule would then have to invent a mapping for.
fn Summary_Of(payload: &FindingPayload) -> String
{
    return format!(
        "{} [{}] ({}): {} ({}:{})",
        payload.external_system, payload.category, payload.severity, payload.message, payload.path, payload.line
    );
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(REVIEW_FINDING),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this subject's review finding could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;
    use nomos_connector_coderabbit::ReviewFindingId;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    const PROVIDER: &str = "nomos.test.review.resolves";

    /// Also [`super::Relay_Findings`]'s own shape: a real fact read and its finding relayed
    /// 1:1, never judged a second time.
    #[test]
    fn Test_Check_Review_Findings_Should_Read_A_Real_Fact_And_Relay_It()
    {
        let source = Source_File("workspace");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Review_Fact(
            &mut store,
            &source,
            &offer,
            &Sample_Payload(),
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Review_Findings(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "coderabbitai/rabbits-playground#review-comment:3521038097");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Observed, "ARC-CONNECTOR-001 invariant 3");
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("Minor"), "{}", found.summary);
        assert_eq!(found.locations, vec!["modules/security/main.tf".to_owned()]);
    }

    fn Materialize_Review_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &FindingPayload)
    {
        let bytes = nomos_connector_coderabbit::Encode_Payload(payload);
        test_support::Materialize(store, source.subject, offer, InputDigest::Of(&[]), nomos_connector_coderabbit::Payload_Schema(), bytes);
    }

    fn Sample_Payload() -> FindingPayload
    {
        return FindingPayload {
            external_system: "coderabbit".to_owned(),
            external_id: ReviewFindingId::Of_Review_Comment("coderabbitai/rabbits-playground", REVIEW_COMMENT_ID),
            locator: "https://github.com/coderabbitai/rabbits-playground/pull/13#discussion_r3521038097".to_owned(),
            category: "🔒 Security & Privacy".to_owned(),
            severity: "🟡 Minor".to_owned(),
            path: "modules/security/main.tf".to_owned(),
            line: "43".to_owned(),
            message: "Consider defining an explicit KMS key policy.".to_owned(),
        };
    }

    /// Also [`Test_Context`]'s own shape: the fixed context every fact and every reader
    /// this suite builds resolves under.
    #[test]
    fn Test_Test_Context_Should_Be_The_Context_A_Real_Reader_Resolves_Facts_Under()
    {
        let source = Source_File("workspace");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Review_Findings(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Advisory);
    }

    /// The review comment id the sample payload names -- the one the real provider's own
    /// locator URL ends with.
    const REVIEW_COMMENT_ID: u64 = 3_521_038_097;

    #[test]
    fn Test_An_Empty_Source_List_Should_Produce_No_Findings()
    {
        let TestOffering { registry, store, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let findings = Check_Review_Findings(&[], &mut reader);

        assert!(findings.is_empty());
    }

    #[test]
    fn Test_A_Wrong_Schema_Should_Be_Reported_As_Unread_Rather_Than_Silently_Ignored()
    {
        let source = Source_File("workspace");
        let TestOffering { mut store, registry, offer } = Offering();
        test_support::Materialize(&mut store, source.subject, &offer, InputDigest::Of(&[]), nomos_contracts::SchemaId::New("nomos.wrong.schema.v1"), b"garbage".to_vec());

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Review_Findings(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").applicability, Applicability::Unparseable);
    }

    fn Source_File(name: &str) -> SourceFile
    {
        return SourceFile::New(name, SubjectId::From_Digest(Content_Digest(name.as_bytes())), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::RuntimeObserved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::None,
        );
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_connector_coderabbit::Capability_Contract(),
            nomos_connector_coderabbit::Capability(),
            nomos_connector_coderabbit::CONTRACT_VERSION,
            PROVIDER,
            Guarantee_At_Floor(),
        );
    }
}
