//! The seams `nomos_rules` reaches into each capability it depends on, exercised from
//! outside the crate.
//!
//! `checks::test_support` and every rule's own `#[cfg(test)] mod tests` already prove
//! these seams work when the crate is opened up — private items and all. What none of
//! that proves is the PUBLIC contract a real consumer actually depends on: a source, a
//! `nomos_analysis::FactReader`, and one of this crate's own `pub fn Check_*` rules. This
//! file is that view — compiled separately, reaching nothing but `nomos_rules`'s own
//! public surface and each capability crate's own public API, the same way
//! `nomos-check-orchestration` really calls in.
//!
//! One test per capability this crate depends on: the happy path across the boundary (a
//! real fact, read and judged) and the lifecycle every one of them shares (an admitted
//! provider, a materialized fact, a reader handed to the rule) — never re-deriving each
//! rule's own judgment logic, which its own inline suite already covers exhaustively.

use nomos_analysis::{Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128, EvidenceClass, FactVariant, GateCategory,
    GenerationId, Guarantee, IncrementalGranularity, ProviderId, SchemaId, SnapshotId, SubjectId,
};
use nomos_model::Content_Digest;
use nomos_rules::SourceFile;

/// The fixed build/variant/configuration/generation every fact here is materialized
/// under — the identity fields no rule under test ever reads, so one fixed context
/// serves every fixture, the same convention `checks::test_support::Test_Context` states
/// for the crate's own inline suites.
fn Fixed_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
        generation: GenerationId::INITIAL,
    };
}

/// A registry declaring `contract` with one admitted `provider` at `guarantee`, and the
/// empty store a fact is materialized into next — the declare-then-offer pairing every
/// seam below needs before a rule can `Require` anything.
fn Admitted(contract: CapabilityContract, capability: CapabilityId, version: ContractVersion, provider: &str, guarantee: Guarantee) -> (Registry, MemoryFactStore, ProviderOffer)
{
    let mut registry = Registry::New();
    let offer = ProviderOffer {
        provider: ProviderId::New(provider),
        capability,
        version,
        guarantee,
    };

    registry.Declare_And_Offer(contract, offer.clone()).expect("declared and offered within the ceiling");

    return (registry, MemoryFactStore::New(), offer);
}

/// Materializes one already-encoded payload under `subject`, addressed the way the real
/// provider named in `offer` would key it.
fn Materialize(store: &mut MemoryFactStore, subject: SubjectId, offer: &ProviderOffer, semantic_inputs: nomos_analysis::InputDigest, schema: SchemaId, bytes: Vec<u8>)
{
    let context = Fixed_Context();
    let key = FactKey {
        contract: offer.capability.clone(),
        contract_version: offer.version,
        subject,
        semantic_inputs,
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
                guarantee: offer.guarantee.clone(),
                payload: FactPayload::New(schema, bytes),
            },
            &[],
        )
        .expect("nothing here is backdated");
}

fn Source(path: &str) -> SourceFile
{
    return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
}

/// The seam with `nomos_analysis`, `nomos_capability`, `nomos_contracts`, `nomos_cap_syntax`,
/// `nomos_model` and `nomos_lang_rust` together: a real parse of real source text, read back
/// through a real registry, store and reader, and judged by [`nomos_rules::Check_Completeness_Mirrors`]
/// exactly the way `nomos-check-orchestration` really calls in.
#[test]
fn Test_Check_Completeness_Mirrors_Should_Judge_A_Real_Parse_Through_The_Public_Reader()
{
    let text = "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n";
    let source = SourceFile::New(
        "a.rs",
        SubjectId::From_Digest(Content_Digest(b"a.rs")),
        text,
    );

    let nomos_lang_rust::Reading::Parsed(facts) = nomos_lang_rust::Read_Source(&source.text)
    else
    {
        panic!("this fixture is well-formed Rust and must parse");
    };
    let payload = nomos_lang_rust::Encode_Payload(&facts);

    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    let (registry, mut store, offer) = Admitted(
        nomos_cap_syntax::Capability_Contract(),
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        "nomos.test.integration_seams.parses",
        guarantee,
    );
    Materialize(
        &mut store,
        source.subject,
        &offer,
        nomos_analysis::InputDigest::Of(&[source.text.as_bytes()]),
        nomos_cap_syntax::Payload_Schema(),
        payload,
    );

    let mut reader = Reader::On(&store, &registry, Fixed_Context());
    let findings = nomos_rules::Check_Completeness_Mirrors(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = findings.first().expect("asserted len 1 above");
    assert_eq!(finding.subject_name, "TABLES");
    assert_eq!(finding.gate, GateCategory::Blocking, "a mirror naming a check nothing declares must block");
}

/// The seam with `nomos_cap_dependency`: a real dependency-edges fact, judged by
/// [`nomos_rules::Check_Dependency_Direction`] through the public reader.
#[test]
fn Test_Check_Dependency_Direction_Should_Judge_A_Real_Dependency_Fact_Through_The_Public_Reader()
{
    let source = Source("nomos-cap-syntax");
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::Project);
    let (registry, mut store, offer) = Admitted(
        nomos_cap_dependency::Capability_Contract(),
        nomos_cap_dependency::Capability(),
        nomos_cap_dependency::CONTRACT_VERSION,
        "nomos.test.integration_seams.dependency",
        guarantee,
    );
    let payload = nomos_cap_dependency::DependencyPayload {
        package: "nomos-cap-syntax".to_owned(),
        edges: vec![nomos_cap_dependency::DependencyEdge {
            target: "nomos-rules".to_owned(),
            kind: nomos_cap_dependency::DependencyKind::Normal,
            optional: false,
        }],
    };
    Materialize(
        &mut store,
        source.subject,
        &offer,
        nomos_analysis::InputDigest::Of(&[]),
        nomos_cap_dependency::Payload_Schema(),
        nomos_cap_dependency::Encode_Payload(&payload),
    );

    let mut reader = Reader::On(&store, &registry, Fixed_Context());
    let findings = nomos_rules::Check_Dependency_Direction(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "an edge that runs upward across a real band gap must be reported: {findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "nomos-cap-syntax");
}

/// The seam with `nomos_cap_controlflow`: a real reachability fact, judged by
/// [`nomos_rules::Check_Unread_Reaches_A_Finding`] through the public reader.
#[test]
fn Test_Check_Unread_Reaches_A_Finding_Should_Judge_A_Real_Reachability_Fact_Through_The_Public_Reader()
{
    let source = Source("a.rs");
    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File);
    let (registry, mut store, offer) = Admitted(
        nomos_cap_controlflow::Capability_Contract(),
        nomos_cap_controlflow::Capability(),
        nomos_cap_controlflow::CONTRACT_VERSION,
        "nomos.test.integration_seams.controlflow",
        guarantee,
    );
    let payload = nomos_cap_controlflow::ReachabilityPayload {
        sites: vec![nomos_cap_controlflow::ReachabilitySite {
            function: "Payload_Of".to_owned(),
            binding: "applicability".to_owned(),
            shape: nomos_cap_controlflow::ArmShape::Empty,
        }],
    };
    Materialize(
        &mut store,
        source.subject,
        &offer,
        nomos_analysis::InputDigest::Of(&[source.text.as_bytes()]),
        nomos_cap_controlflow::Payload_Schema(),
        nomos_cap_controlflow::Encode_Payload(&payload),
    );

    let mut reader = Reader::On(&store, &registry, Fixed_Context());
    let findings = nomos_rules::Check_Unread_Reaches_A_Finding(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "a.rs");
}

/// The seam with `nomos_cap_lint`: a real diagnostics fact, relayed 1:1 by
/// [`nomos_rules::Check_Lint_Diagnostics`] through the public reader.
#[test]
fn Test_Check_Lint_Diagnostics_Should_Relay_A_Real_Diagnostics_Fact_Through_The_Public_Reader()
{
    let source = Source("nomos-rules");
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::Project);
    let (registry, mut store, offer) = Admitted(
        nomos_cap_lint::Capability_Contract(),
        nomos_cap_lint::Capability(),
        nomos_cap_lint::CONTRACT_VERSION,
        "nomos.test.integration_seams.lint",
        guarantee,
    );
    let payload = nomos_cap_lint::DiagnosticsPayload {
        package: "nomos-rules".to_owned(),
        diagnostics: vec![nomos_cap_lint::LintDiagnostic {
            level: nomos_cap_lint::LintLevel::Warning,
            lint: Some("clippy::needless_return".to_owned()),
            message: "unneeded `return` statement".to_owned(),
            file: "crates/rules/nomos-rules/src/lib.rs".to_owned(),
            line: 1,
        }],
    };
    Materialize(
        &mut store,
        source.subject,
        &offer,
        nomos_analysis::InputDigest::Of(&[]),
        nomos_cap_lint::Payload_Schema(),
        nomos_cap_lint::Encode_Payload(&payload),
    );

    let mut reader = Reader::On(&store, &registry, Fixed_Context());
    let findings = nomos_rules::Check_Lint_Diagnostics(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings.first().expect("asserted len 1 above").summary.contains("needless_return"));
}

/// The seam with `nomos_cap_dependency_policy`: a real policy fact, relayed 1:1 by
/// [`nomos_rules::Check_Dependency_Policy`] through the public reader.
#[test]
fn Test_Check_Dependency_Policy_Should_Relay_A_Real_Policy_Fact_Through_The_Public_Reader()
{
    let source = SourceFile::New("workspace", nomos_model::Subject_Of_Path(""), String::new());
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::WholeWorkspace);
    let (registry, mut store, offer) = Admitted(
        nomos_cap_dependency_policy::Capability_Contract(),
        nomos_cap_dependency_policy::Capability(),
        nomos_cap_dependency_policy::CONTRACT_VERSION,
        "nomos.test.integration_seams.policy",
        guarantee,
    );
    let payload = nomos_cap_dependency_policy::PolicyPayload {
        violations: vec![nomos_cap_dependency_policy::PolicyViolation {
            severity: nomos_cap_dependency_policy::PolicySeverity::Warning,
            code: "duplicate".to_owned(),
            message: "found 2 duplicate entries for crate 'syn'".to_owned(),
        }],
    };
    Materialize(
        &mut store,
        source.subject,
        &offer,
        nomos_analysis::InputDigest::Of(&[]),
        nomos_cap_dependency_policy::Payload_Schema(),
        nomos_cap_dependency_policy::Encode_Payload(&payload),
    );

    let mut reader = Reader::On(&store, &registry, Fixed_Context());
    let findings = nomos_rules::Check_Dependency_Policy(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings.first().expect("asserted len 1 above").summary.contains("duplicate"));
}
