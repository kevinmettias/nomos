//! One analyzed crate's own lock-inside-a-lock sites, relayed as `Finding`s.
//!
//! `OD-RULES-010` decided a provider's output is a fact a native rule judges, not a
//! `Finding` the provider emits directly. This rule follows the identical shape
//! [`crate::Check_Copy_Clones`] already follows for the other capability the same
//! compiler-backed provider answers: since `nomos.cap.rust.nested_locks`' own analysis has
//! already reached the judgment (an outer lock's type argument resolved to another lock
//! type), this rule's own judgment is a 1:1 relay.
//!
//! # Why this rule lives here and not beside its provider
//!
//! The same sequence [`crate::checks::copy_clones`]' own module doc states, for the same
//! reason: `OD-ANALYSIS-007` version 2 decided a rule naming this capability belongs in this
//! crate, because `Permits` forbids `Rules` from naming `Provider` and a rule written inside
//! the provider crate could never be composed into a run at all.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_rust_nested_locks::{NestedLockFinding, NestedLockPayload};
use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const NESTED_LOCKS: &str = "nested-locks";

/// The record this implementation's contract is written in -- the same `OD-RULES-010`
/// [`crate::checks::copy_clones::COPY_CLONES_CONTRACT_RECORD`] cites, for the identical
/// relay shape over the identical provider.
pub const NESTED_LOCKS_CONTRACT_RECORD: &str = "OD-RULES-010";

/// The version of [`NESTED_LOCKS_CONTRACT_RECORD`] this implementation was written against.
pub const NESTED_LOCKS_CONTRACT_RECORD_VERSION: u32 = 2;

/// Judges each analyzed crate's one `nomos.cap.rust.nested_locks` fact, if `sources` names
/// one, against the real compiler-backed analysis that produced it.
#[must_use]
pub fn Check_Nested_Locks(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return super::Relay_Findings(sources, facts, Payload_Of, Findings_Of);
}

/// The analyzed crate's decoded payload, or a finding reporting why it could not be read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<NestedLockPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires the subject's nested-locks fact, turning an inadmissible answer into an unread
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Nested_Locks_Requirement();
    let capability = nomos_cap_rust_nested_locks::Capability();
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

/// What this rule needs from `nomos.cap.rust.nested_locks` before it will believe an answer
/// -- the same "nothing weaker could be trusted" reasoning
/// [`crate::checks::copy_clones`]' own `Copy_Clones_Requirement` already gives: a nested
/// lock this rule cannot attribute to a real, resolved type is not one it should relay as if
/// a real compiler frontend vouched for it.
#[must_use]
fn Nested_Locks_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );

    return Requirement::New(nomos_cap_rust_nested_locks::Capability(), nomos_cap_rust_nested_locks::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_rust_nested_locks::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this subject carries payload schema `{}`, which this build does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`NestedLockPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<NestedLockPayload, Finding>
{
    return nomos_cap_rust_nested_locks::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`'s own findings, each relayed as a `Finding` -- never judged a second time, the
/// same reason [`crate::checks::copy_clones`]' own `Findings_Of` already gives.
fn Findings_Of(source: &SourceFile, payload: &NestedLockPayload) -> Vec<Finding>
{
    return payload.findings.iter().map(|finding| return Finding_For(source, finding)).collect();
}

fn Finding_For(source: &SourceFile, finding: &NestedLockFinding) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(NESTED_LOCKS),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("the lock at {} guards a value whose own type is already a Mutex or RwLock", finding.location),
        locations: vec![finding.location.clone()],
    };
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(NESTED_LOCKS),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this subject's nested-locks could not be judged: {because}"),
        locations: Vec::new(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;

    const PROVIDER: &str = "nomos.test.compiler.resolves";

    /// The two findings `Test_Multiple_Nested_Locks_Should_Each_Become_A_Finding` writes into
    /// its payload, and therefore the two findings a relay of it must produce.
    const EXPECTED_NESTED_LOCK_FINDINGS: usize = 2;

    #[test]
    fn Test_Check_Nested_Locks_Should_Relay_A_Real_Findings_Own_Location()
    {
        let source = Source();
        let payload = NestedLockPayload { findings: vec![NestedLockFinding { location: "src/lib.rs:15:5".to_owned() }] };
        let TestOffering { store, registry, .. } = Offering_With_Nested_Locks_Fact(&source, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Nested_Locks(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "crates/example");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Derived);
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("src/lib.rs:15:5"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Nested_Locks_Should_Produce_No_Finding_For_A_Clean_Fact()
    {
        let source = Source();
        let TestOffering { store, registry, .. } = Offering_With_Nested_Locks_Fact(&source, &NestedLockPayload { findings: Vec::new() });

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Nested_Locks(&[source], &mut reader);

        assert!(findings.is_empty(), "a clean report must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
    {
        let source = Source();
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Nested_Locks(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Advisory);
    }

    /// An empty slice is what a run hands this rule when its family was never materialized,
    /// and it must judge nothing rather than invent a subject to report against.
    #[test]
    fn Test_An_Empty_Source_List_Should_Be_Judged_As_Nothing()
    {
        let TestOffering { store, registry, .. } = Offering();
        let mut reader = Reader::On(&store, &registry, Test_Context());

        let findings = Check_Nested_Locks(&[], &mut reader);

        assert!(findings.is_empty());
    }

    #[test]
    fn Test_Multiple_Nested_Locks_Should_Each_Become_A_Finding()
    {
        let source = Source();
        let payload = NestedLockPayload {
            findings: vec![
                NestedLockFinding { location: "src/lib.rs:15:5".to_owned() },
                NestedLockFinding { location: "src/lib.rs:31:9".to_owned() },
            ],
        };
        let TestOffering { store, registry, .. } = Offering_With_Nested_Locks_Fact(&source, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Nested_Locks(&[source], &mut reader);

        assert_eq!(findings.len(), EXPECTED_NESTED_LOCK_FINDINGS, "{findings:?}");
    }

    fn Offering_With_Nested_Locks_Fact(source: &SourceFile, payload: &NestedLockPayload) -> TestOffering
    {
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Nested_Locks_Fact(&mut store, source, &offer, payload);

        return TestOffering { store, registry, offer };
    }

    fn Source() -> SourceFile
    {
        return SourceFile::New("crates/example", nomos_model::Subject_Of_Path("crates/example"), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::Project,
        );
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offered_Registry(
            OfferedProvider {
                contract: nomos_cap_rust_nested_locks::Capability_Contract(),
                capability: nomos_cap_rust_nested_locks::Capability(),
                version: nomos_cap_rust_nested_locks::CONTRACT_VERSION,
                provider: PROVIDER,
                guarantee: Guarantee_At_Floor(),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }

    fn Materialize_Nested_Locks_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &NestedLockPayload)
    {
        let bytes = nomos_cap_rust_nested_locks::Encode_Payload(payload);
        test_support::Materialize_Fact(
            store,
            FactToFile {
                subject: source.subject,
                offer,
                semantic_inputs: InputDigest::Of(&[]),
                schema: nomos_cap_rust_nested_locks::Payload_Schema(),
                bytes,
            },
        )
        .expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
