//! Judging the one `nomos.cap.rust.copy_clones` fact a subject's analysis produced,
//! relayed as `Finding`s.
//!
//! `OD-RULES-010` decided a `ToolProvider`'s output is a fact a native rule judges, not
//! a `Finding` the tool emits directly. This rule follows the identical shape for a
//! provider backed by a real compiler frontend rather than a subprocess: since this
//! capability's own analysis has already reached the judgment (a call site clones a
//! `Copy` type), this rule's own judgment is a 1:1 relay, the same
//! `Require`-then-decode-then-emit shape `nomos_rules::Check_Dependency_Policy` uses for
//! the identical reason.
//!
//! # Not yet composed into a real gate run
//!
//! Wiring this rule into `nomos-check-orchestration::Run` and `nomos-rules::DESCRIPTORS`
//! is that crate's and `nomos-rules`' own territory, not this one's -- the same split
//! `nomos_lang_rust_deny`'s own capability-and-provider commits left for a third,
//! separate change to draw. What is here is what this capability's own crate can prove
//! on its own: a real fact, and a real rule that reads it through the same
//! `FactReader`/`Requirement` seam every composed rule reads through.

use crate::payload::clone_on_copy_payload::CloneOnCopyPayload;
use crate::payload::cloned_copy_type::ClonedCopyType;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee, IncrementalGranularity, RuleId, SubjectId,
};
use nomos_capability::Requirement;

/// This rule's own identifier.
pub const COPY_CLONES: &str = "copy-clones";

/// Judges `subject`'s one `nomos.cap.rust.copy_clones` fact, if one was materialized for
/// it, against the real compiler-backed analysis that produced it.
#[must_use]
pub fn Check_Copy_Clones(subject: SubjectId, subject_name: &str, facts: &mut dyn FactReader) -> Vec<Finding>
{
    return match Payload_Of(subject, facts)
    {
        Ok(payload) => Findings_Of(subject, subject_name, &payload),
        Err(finding) => vec![finding],
    };
}

/// The subject's decoded payload, or a finding reporting why it could not be read.
fn Payload_Of(subject: SubjectId, facts: &mut dyn FactReader) -> Result<CloneOnCopyPayload, Finding>
{
    let fact = Require_Fact(subject, facts)?;
    Check_Schema(subject, fact)?;
    return Parse_Fact(subject, fact);
}

/// Requires the subject's copy-clones fact, turning an inadmissible answer into an
/// unread finding.
fn Require_Fact(subject: SubjectId, facts: &mut dyn FactReader) -> Result<&MaterializedFact, Finding>
{
    let need = Copy_Clones_Requirement();
    let capability = crate::contract::Capability();
    let inputs = InputDigest::Of(&[]);

    return match facts.Require(&capability, &subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread_Finding(
            subject,
            "",
            applicability,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// What this rule needs from `nomos.cap.rust.copy_clones` before it will believe an
/// answer -- the same "nothing weaker could be trusted" reasoning
/// `nomos_rules::checks::policy::Policy_Requirement` gives: a clone-on-copy call this
/// rule cannot attribute to a real, resolved type is not one it should relay as if a
/// real compiler frontend vouched for it.
#[must_use]
fn Copy_Clones_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );

    return Requirement::New(crate::contract::Capability(), crate::contract::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(subject: SubjectId, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != crate::contract::Payload_Schema()
    {
        return Err(Unread_Finding(
            subject,
            "",
            Applicability::Unparseable,
            &format!(
                "the fact for this subject carries payload schema `{}`, which this build does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`CloneOnCopyPayload`] shape.
fn Parse_Fact(subject: SubjectId, fact: &MaterializedFact) -> Result<CloneOnCopyPayload, Finding>
{
    return crate::payload::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(subject, "", Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`'s own findings, each relayed as a `Finding` -- never judged a second time,
/// the whole point `OD-RULES-010` decided: this capability's own analysis already
/// reached the verdict this rule reports.
fn Findings_Of(subject: SubjectId, subject_name: &str, payload: &CloneOnCopyPayload) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = payload.findings.iter().map(|finding| return Finding_For(subject, subject_name, finding)).collect();

    findings.sort_by(|left, right| return left.summary.cmp(&right.summary));
    return findings;
}

fn Finding_For(subject: SubjectId, subject_name: &str, finding: &ClonedCopyType) -> Finding
{
    return Finding {
        rule: RuleId::New(COPY_CLONES),
        subject,
        subject_name: subject_name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("`.clone()` at {} duplicates a value whose type already implements Copy", finding.location),
        locations: vec![finding.location.clone()],
    };
}

fn Unread_Finding(subject: SubjectId, subject_name: &str, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(COPY_CLONES),
        subject,
        subject_name: subject_name.to_owned(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this subject's copy-clones could not be judged: {because}"),
        locations: Vec::new(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::contract::{Capability, Capability_Contract, Payload_Schema, CONTRACT_VERSION};
    use crate::guarantee::{Declared_Guarantee, PROVIDER};
    use crate::payload::Encode_Payload;
    use nomos_analysis::{Context, FactKey, FactPayload, GuaranteeDigest, MemoryFactStore, Reader};
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, ProviderId, SnapshotId};

    #[test]
    fn Test_Check_Copy_Clones_Should_Relay_A_Real_Findings_Own_Location()
    {
        let subject = nomos_model::Subject_Of_Path("crates/example");
        let mut store = MemoryFactStore::New();
        let registry = Offered_Registry();
        Materialize(
            &mut store,
            &registry,
            subject,
            &CloneOnCopyPayload { findings: vec![ClonedCopyType { location: "src/lib.rs:23:12".to_owned() }] },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(subject, "crates/example", &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "crates/example");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Derived);
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("src/lib.rs:23:12"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Copy_Clones_Should_Produce_No_Finding_For_A_Clean_Fact()
    {
        let subject = nomos_model::Subject_Of_Path("crates/example");
        let mut store = MemoryFactStore::New();
        let registry = Offered_Registry();
        Materialize(&mut store, &registry, subject, &CloneOnCopyPayload { findings: Vec::new() });

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(subject, "crates/example", &mut reader);

        assert!(findings.is_empty(), "a clean report must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
    {
        let subject = nomos_model::Subject_Of_Path("crates/example");
        let registry = Offered_Registry();
        let store = MemoryFactStore::New();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(subject, "crates/example", &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Advisory);
    }

    fn Offered_Registry() -> Registry
    {
        let mut registry = Registry::New();
        registry
            .Declare_And_Offer(
                Capability_Contract(),
                ProviderOffer { provider: ProviderId::New(PROVIDER), capability: Capability(), version: CONTRACT_VERSION, guarantee: Declared_Guarantee() },
            )
            .expect("declared and offered within the ceiling");
        return registry;
    }

    fn Materialize(store: &mut MemoryFactStore, registry: &Registry, subject: SubjectId, payload: &CloneOnCopyPayload)
    {
        let offer = registry.Offers(&Capability()).first().expect("Offered_Registry always offers exactly one provider").clone();
        let context = Test_Context();
        let key = FactKey {
            contract: offer.capability.clone(),
            contract_version: offer.version,
            subject,
            semantic_inputs: InputDigest::Of(&[]),
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: context.variant,
            configuration: context.configuration,
        };

        store
            .Materialize(
                MaterializedFact {
                    identity: key.At(context.generation),
                    snapshot: context.snapshot,
                    evidence: EvidenceClass::Verified,
                    guarantee: offer.guarantee,
                    payload: FactPayload::New(Payload_Schema(), Encode_Payload(payload)),
                },
                &[],
            )
            .expect("nothing here is backdated");
    }

    fn Test_Context() -> Context
    {
        return Context {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
            generation: GenerationId::INITIAL,
        };
    }
}
