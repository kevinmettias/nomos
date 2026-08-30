//! The workspace's own bans/licenses/sources verdict from an external policy tool,
//! relayed as `Finding`s.
//!
//! `OD-RULES-010` decided a `ToolProvider`'s output is a fact a native rule judges, not a
//! `Finding` the tool emits directly. This is that rule's second real instance, for
//! `nomos.cap.dependency.policy`: since a policy tool's own verdict is already a judgment,
//! this rule's own judgment is a 1:1 relay rather than a second opinion — the same
//! `Require`-then-decode-then-emit shape [`crate::Check_Lint_Diagnostics`] already uses for
//! the identical reason, minus even that rule's own per-member loop: this capability's one
//! real provider materializes exactly one fact for the whole workspace, so `sources` here
//! carries at most one entry rather than one per workspace member.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! Behind `OD-GATE-017`'s own per-call rule subset, the same `Wants(selected, ...)` gate
//! `LINT_DIAGNOSTICS` already sits behind — wiring this rule in is
//! `nomos-check-orchestration`'s own territory, not this crate's.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_dependency_policy::{PolicyPayload, PolicyViolation};
use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const DEPENDENCY_POLICY: &str = "dependency-policy";

/// Judges the workspace's one `nomos.cap.dependency.policy` fact, if `sources` names one,
/// against `cargo deny`'s own reported violations.
///
/// At most one source, the same shape [`crate::Check_Lint_Diagnostics`] reads for a list —
/// here there is only ever one to read, since `IncrementalGranularity::WholeWorkspace`
/// means the provider materialized exactly one fact for the workspace as a whole rather
/// than one per member.
#[must_use]
pub fn Check_Dependency_Policy(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let source_findings = Findings_Of(source, &payload);
                findings.extend(source_findings);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
    return findings;
}

/// The workspace's decoded policy payload, or a finding reporting why it could not be
/// read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<PolicyPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires the workspace's policy fact, turning an inadmissible answer into an [`Unread`]
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Policy_Requirement();
    let capability = nomos_cap_dependency_policy::Capability();
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

/// What this rule needs from `nomos.cap.dependency.policy` before it will believe an
/// answer — the same "nothing weaker could be trusted" reasoning
/// `crate::lint::Lint_Requirement` gives: a violation this rule cannot attribute to the
/// real, resolved dependency graph is not one it should relay as if `cargo deny` itself
/// vouched for it.
#[must_use]
fn Policy_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::WholeWorkspace,
    );

    return Requirement::New(nomos_cap_dependency_policy::Capability(), nomos_cap_dependency_policy::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_dependency_policy::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this workspace carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`PolicyPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<PolicyPayload, Finding>
{
    return nomos_cap_dependency_policy::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`'s own violations, each relayed as a `Finding` — never judged a second time,
/// the whole point `OD-RULES-010` decided: `cargo deny` already reached the verdict this
/// rule reports.
fn Findings_Of(source: &SourceFile, payload: &PolicyPayload) -> Vec<Finding>
{
    return payload.violations.iter().map(|violation| return Finding_For_Violation(source, violation)).collect();
}

fn Finding_For_Violation(source: &SourceFile, violation: &PolicyViolation) -> Finding
{
    return Finding {
        rule: RuleId::New(DEPENDENCY_POLICY),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: Summary_Of(violation),
        locations: Vec::new(),
    };
}

/// `violation`, rendered as one line — the tool's own severity, code and message. No
/// location: unlike a lint diagnostic's primary span, a `cargo deny` violation does not
/// always name one file, and `PolicyViolation`'s own module doc already declines to invent
/// one.
fn Summary_Of(violation: &PolicyViolation) -> String
{
    return format!("{} [{}]: {}", violation.severity, violation.code, violation.message);
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(DEPENDENCY_POLICY),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("the workspace's dependency policy could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact as Fact, MemoryFactStore, Reader,
    };
    use nomos_cap_dependency_policy::PolicySeverity;
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, ProviderId, SnapshotId};

    const PROVIDER: &str = "nomos.test.policy.resolves";

    #[test]
    fn Test_A_Real_Fact_With_A_Violation_Should_Be_Read_And_Relayed()
    {
        let source = Source();
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Policy_Fact(
            &mut store,
            &source,
            &offer,
            &PolicyPayload {
                violations: vec![PolicyViolation {
                    severity: PolicySeverity::Warning,
                    code: "duplicate".to_owned(),
                    message: "found 2 duplicate entries for crate 'syn'".to_owned(),
                }],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Policy(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "workspace");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Derived);
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("duplicate"), "{}", found.summary);
    }

    #[test]
    fn Test_A_Clean_Facts_Should_Produce_No_Finding()
    {
        let source = Source();
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Policy_Fact(&mut store, &source, &offer, &PolicyPayload { violations: Vec::new() });

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Policy(&[source], &mut reader);

        assert!(findings.is_empty(), "a clean report must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_No_Sources_Should_Produce_No_Finding()
    {
        let TestOffering { registry, store, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let findings = Check_Dependency_Policy(&[], &mut reader);

        assert!(findings.is_empty());
    }

    struct TestOffering
    {
        store: MemoryFactStore,
        registry: Registry,
        offer: ProviderOffer,
    }

    #[test]
    fn Test_A_Source_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
    {
        let source = Source();
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Policy(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Multiple_Violations_Should_Each_Become_A_Finding()
    {
        let source = Source();
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Policy_Fact(
            &mut store,
            &source,
            &offer,
            &PolicyPayload {
                violations: vec![
                    PolicyViolation { severity: PolicySeverity::Warning, code: "duplicate".to_owned(), message: "m1".to_owned() },
                    PolicyViolation { severity: PolicySeverity::Error, code: "banned".to_owned(), message: "m2".to_owned() },
                ],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Policy(&[source], &mut reader);

        assert_eq!(findings.len(), 2, "{findings:?}");
    }

    fn Source() -> SourceFile
    {
        return SourceFile::New("workspace", nomos_model::Subject_Of_Path(""), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::WholeWorkspace,
        );
    }

    fn Test_Context() -> Context
    {
        return Context {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
            generation: GenerationId::INITIAL,
        };
    }

    fn Offering() -> TestOffering
    {
        let mut registry = Registry::New();
        registry
            .Declare(nomos_cap_dependency_policy::Capability_Contract())
            .expect("the dependency-policy capability is declared once");

        let offer = ProviderOffer {
            provider: ProviderId::New(PROVIDER),
            capability: nomos_cap_dependency_policy::Capability(),
            version: nomos_cap_dependency_policy::CONTRACT_VERSION,
            guarantee: Guarantee_At_Floor(),
        };
        registry.Offer(offer.clone()).expect("within the ceiling");

        return TestOffering { store: MemoryFactStore::New(), registry, offer };
    }

    fn Materialize_Policy_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &PolicyPayload)
    {
        let context = Test_Context();
        let bytes = nomos_cap_dependency_policy::Encode_Payload(payload);
        let key = FactKey {
            contract: nomos_cap_dependency_policy::Capability(),
            contract_version: offer.version,
            subject: source.subject,
            semantic_inputs: InputDigest::Of(&[]),
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: context.variant,
            configuration: context.configuration,
        };

        store
            .Materialize(
                Fact {
                    identity: key.At(context.generation),
                    snapshot: context.snapshot,
                    evidence: EvidenceClass::Verified,
                    guarantee: offer.guarantee,
                    payload: FactPayload::New(nomos_cap_dependency_policy::Payload_Schema(), bytes),
                },
                &[],
            )
            .expect("nothing here is backdated");
    }
}
