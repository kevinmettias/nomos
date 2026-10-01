//! One analyzed crate's own `.clone()`-on-`Copy` sites, relayed as `Finding`s.
//!
//! `OD-RULES-010` decided a provider's output is a fact a native rule judges, not a
//! `Finding` the provider emits directly. This rule is that decision's shape for a provider
//! backed by a real compiler frontend rather than a subprocess: since
//! `nomos.cap.rust.copy_clones`' own analysis has already reached the judgment (a call site
//! clones a `Copy` type), this rule's own judgment is a 1:1 relay -- the same
//! `Require`-then-decode-then-emit shape [`crate::Check_Dependency_Policy`] uses, minus even
//! that rule's second opinion.
//!
//! # Why this rule lives here and not beside its provider
//!
//! It was written inside `nomos-lang-rust-compiler`, where its capability's contract also
//! sat, and nothing composed either. `OD-ANALYSIS-007` version 2 decided both halves of
//! that at once: the contract moves to its own crate the moment a rule names the capability,
//! and the rule that names it belongs here, because `Permits` forbids `Rules` from naming
//! `Provider` and a rule written inside a provider crate can therefore never be composed
//! into a run at all. What crossed with it is only the judgment; the `ra_ap_hir` analysis
//! stayed where its dependencies are.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_rust_copy_clones::{CloneOnCopyPayload, ClonedCopyType};
use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const COPY_CLONES: &str = "copy-clones";

/// The record this implementation's contract is written in -- `OD-RULES-010` decided a
/// provider's output is a fact a native rule judges, which is what this rule does with a
/// compiler-backed provider's answer, the same record
/// [`crate::checks::policy::DEPENDENCY_POLICY_CONTRACT_RECORD`] and
/// [`crate::checks::review::REVIEW_CONTRACT_RECORD`] both cite for the identical relay shape
/// over a subprocess and over a connector respectively.
pub const COPY_CLONES_CONTRACT_RECORD: &str = "OD-RULES-010";

/// The version of [`COPY_CLONES_CONTRACT_RECORD`] this implementation was written against.
pub const COPY_CLONES_CONTRACT_RECORD_VERSION: u32 = 2;

/// Judges each analyzed crate's one `nomos.cap.rust.copy_clones` fact, if `sources` names
/// one, against the real compiler-backed analysis that produced it.
///
/// At most one source, the same shape [`crate::Check_Dependency_Policy`] reads for a list:
/// this capability's ceiling is `IncrementalGranularity::Project` and its one real provider
/// is run over the root a check was asked about, so the materialized slice holds one entry
/// per analyzed project rather than one per walked file.
#[must_use]
pub fn Check_Copy_Clones(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return super::Relay_Findings(sources, facts, Payload_Of, Findings_Of);
}

/// The analyzed crate's decoded payload, or a finding reporting why it could not be read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<CloneOnCopyPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires the subject's copy-clones fact, turning an inadmissible answer into an unread
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Copy_Clones_Requirement();
    let capability = nomos_cap_rust_copy_clones::Capability();
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

/// What this rule needs from `nomos.cap.rust.copy_clones` before it will believe an answer
/// -- the same "nothing weaker could be trusted" reasoning
/// [`crate::checks::policy`]'s own `Policy_Requirement` gives: a clone-on-copy call this
/// rule cannot attribute to a real, resolved type is not one it should relay as if a real
/// compiler frontend vouched for it.
#[must_use]
fn Copy_Clones_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );

    return Requirement::New(nomos_cap_rust_copy_clones::Capability(), nomos_cap_rust_copy_clones::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_rust_copy_clones::Payload_Schema()
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

/// Decodes `fact`'s payload bytes into this rule's own [`CloneOnCopyPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<CloneOnCopyPayload, Finding>
{
    return nomos_cap_rust_copy_clones::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`'s own findings, each relayed as a `Finding` -- never judged a second time, the
/// whole point `OD-RULES-010` decided: this capability's own analysis already reached the
/// verdict this rule reports.
fn Findings_Of(source: &SourceFile, payload: &CloneOnCopyPayload) -> Vec<Finding>
{
    return payload.findings.iter().map(|finding| return Finding_For(source, finding)).collect();
}

fn Finding_For(source: &SourceFile, finding: &ClonedCopyType) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(COPY_CLONES),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("`.clone()` at {} duplicates a value whose type already implements Copy", finding.location),
        locations: vec![finding.location.clone()],
    };
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(COPY_CLONES),
        subject: source.subject,
        subject_name: source.path.clone(),
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
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::ProviderOffer;

    const PROVIDER: &str = "nomos.test.compiler.resolves";

    /// The two findings `Test_Multiple_Clone_Sites_Should_Each_Become_A_Finding` writes into
    /// its payload, and therefore the two findings a relay of it must produce.
    const EXPECTED_CLONE_FINDINGS: usize = 2;

    #[test]
    fn Test_Check_Copy_Clones_Should_Relay_A_Real_Findings_Own_Location()
    {
        let source = Source();
        let payload = CloneOnCopyPayload { findings: vec![ClonedCopyType { location: "src/lib.rs:23:12".to_owned() }] };
        let TestOffering { store, registry, .. } = Offering_With_Copy_Clones_Fact(&source, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(&[source], &mut reader);

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
        let source = Source();
        let TestOffering { store, registry, .. } = Offering_With_Copy_Clones_Fact(&source, &CloneOnCopyPayload { findings: Vec::new() });

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(&[source], &mut reader);

        assert!(findings.is_empty(), "a clean report must not manufacture a finding: {findings:?}");
    }

    #[test]
    fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
    {
        let source = Source();
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(&[source], &mut reader);

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

        let findings = Check_Copy_Clones(&[], &mut reader);

        assert!(findings.is_empty());
    }

    #[test]
    fn Test_Multiple_Clone_Sites_Should_Each_Become_A_Finding()
    {
        let source = Source();
        let payload = CloneOnCopyPayload {
            findings: vec![
                ClonedCopyType { location: "src/lib.rs:7:16".to_owned() },
                ClonedCopyType { location: "src/lib.rs:12:5".to_owned() },
            ],
        };
        let TestOffering { store, registry, .. } = Offering_With_Copy_Clones_Fact(&source, &payload);

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Copy_Clones(&[source], &mut reader);

        assert_eq!(findings.len(), EXPECTED_CLONE_FINDINGS, "{findings:?}");
    }

    fn Offering_With_Copy_Clones_Fact(source: &SourceFile, payload: &CloneOnCopyPayload) -> TestOffering
    {
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Copy_Clones_Fact(&mut store, source, &offer, payload);

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
                contract: nomos_cap_rust_copy_clones::Capability_Contract(),
                capability: nomos_cap_rust_copy_clones::Capability(),
                version: nomos_cap_rust_copy_clones::CONTRACT_VERSION,
                provider: PROVIDER,
                guarantee: Guarantee_At_Floor(),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }

    fn Materialize_Copy_Clones_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &CloneOnCopyPayload)
    {
        let bytes = nomos_cap_rust_copy_clones::Encode_Payload(payload);
        test_support::Materialize_Fact(
            store,
            FactToFile {
                subject: source.subject,
                offer,
                semantic_inputs: InputDigest::Of(&[]),
                schema: nomos_cap_rust_copy_clones::Payload_Schema(),
                bytes,
            },
        )
        .expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
