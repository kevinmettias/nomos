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
/// The one byte that tells each identity field's digest apart from its neighbours'; the
/// fields themselves are the identity no rule under test ever reads.
const SNAPSHOT_FILL: u8 = 1;
const VARIANT_FILL: u8 = 2;
const CONFIGURATION_FILL: u8 = 3;

fn Fixed_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_FILL; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// One capability as this file's seams declare it: the contract the provider is declared
/// against, the capability and contract version it is declared at, the provider's own name,
/// and the guarantee it offers under. Grouped because every seam below passes all five
/// together, and four of them are adjacent positions the compiler could not tell apart if a
/// call site ever swapped two.
struct Offered<'a>
{
    contract: CapabilityContract,
    capability: CapabilityId,
    version: ContractVersion,
    provider: &'a str,
    guarantee: Guarantee,
}

/// A capability declared and offered, alongside the empty store its first fact is
/// materialized into — the three values every seam below holds at once, named rather than
/// returned as a tuple so a reader cannot take the store where the registry belongs.
struct Admission
{
    registry: Registry,
    store: MemoryFactStore,
    offer: ProviderOffer,
}

/// One already-encoded payload: its own schema and its own bytes. Grouped for the same
/// reason as [`Offered`] — the two are adjacent positions of different types at every call
/// site below, and a name is the only thing that tells them apart at a glance.
struct Payload
{
    schema: SchemaId,
    bytes: Vec<u8>,
}

/// Declares `offered`'s contract with one admitted provider at `offered`'s own guarantee,
/// materializes `payload` under `source`'s own subject, and hands back the admission a rule
/// is then judged through — the declare-offer-materialize sequence every seam below shares,
/// written down once so no seam's own body is mostly it.
fn Admission_Of(offered: Offered<'_>, source: &SourceFile, semantic_inputs: &[&[u8]], payload: Payload) -> Admission
{
    let mut admission = Admitted(offered);
    let context = Fixed_Context();
    let key = Fact_Key_For(&admission.offer, source.subject, semantic_inputs);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee: admission.offer.guarantee.clone(),
        payload: FactPayload::New(payload.schema, payload.bytes),
    };

    admission.store.Materialize(fact, &[]).expect("nothing here is backdated");

    return admission;
}

/// The key a fact materialized under `offer`, about `subject` and derived from
/// `semantic_inputs` is addressed by: the offer's own contract, version, provider and
/// guarantee, and the fixed context's variant and configuration.
fn Fact_Key_For(offer: &ProviderOffer, subject: SubjectId, semantic_inputs: &[&[u8]]) -> FactKey
{
    let context = Fixed_Context();

    return FactKey {
        contract: offer.capability.clone(),
        contract_version: offer.version,
        subject,
        semantic_inputs: nomos_analysis::InputDigest::Of(semantic_inputs),
        provider: offer.provider.clone(),
        provider_version: offer.version,
        guarantee: GuaranteeDigest::Of(&offer.guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

fn Source(path: &str) -> SourceFile
{
    return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
}

/// Declares `offered`'s contract with one admitted provider at `offered`'s own guarantee,
/// and hands back that admission beside the empty store
/// [`Admission_Of`] materializes into.
fn Admitted(offered: Offered<'_>) -> Admission
{
    let mut registry = Registry::New();
    let offer = ProviderOffer {
        provider: ProviderId::New(offered.provider),
        capability: offered.capability.clone(),
        version: offered.version,
        guarantee: offered.guarantee.clone(),
    };

    registry.Declare_And_Offer(offered.contract, offer.clone()).expect("declared and offered within the ceiling");

    return Admission { registry, store: MemoryFactStore::New(), offer };
}

/// The `nomos_cap_syntax` seam's payload: `text` parsed by this workspace's own Rust reader,
/// encoded the way that capability's real provider hands it on.
fn Syntax_Payload(text: &str) -> Payload
{
    let nomos_lang_rust::Reading::Parsed(facts) = nomos_lang_rust::Read_Source(text)
    else
    {
        panic!("this fixture is well-formed Rust and must parse");
    };

    return Payload { schema: nomos_cap_syntax::Payload_Schema(), bytes: nomos_lang_rust::Encode_Payload(&facts) };
}

/// `nomos_cap_syntax`'s own offer: a syntactic, sound, incrementally file-scoped fact, which
/// is what that capability's real provider declares.
fn Syntax_Offered() -> Offered<'static>
{
    return Offered {
        contract: nomos_cap_syntax::Capability_Contract(),
        capability: nomos_cap_syntax::Capability(),
        version: nomos_cap_syntax::CONTRACT_VERSION,
        provider: "nomos.test.integration_seams.parses",
        guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
    };
}

/// The `nomos_cap_dependency` seam's payload: one edge running from `nomos-cap-syntax` up to
/// `nomos-rules`, across a real band gap.
fn Dependency_Payload() -> Payload
{
    let edge = nomos_cap_dependency::DependencyPayload {
        package: "nomos-cap-syntax".to_owned(),
        edges: vec![nomos_cap_dependency::DependencyEdge {
            target: "nomos-rules".to_owned(),
            kind: nomos_cap_dependency::DependencyKind::Normal,
            optional: false,
        }],
    };

    return Payload { schema: nomos_cap_dependency::Payload_Schema(), bytes: nomos_cap_dependency::Encode_Payload(&edge) };
}

/// `nomos_cap_dependency`'s own offer: a semantically resolved, project-scoped fact.
fn Dependency_Offered() -> Offered<'static>
{
    return Offered {
        contract: nomos_cap_dependency::Capability_Contract(),
        capability: nomos_cap_dependency::Capability(),
        version: nomos_cap_dependency::CONTRACT_VERSION,
        provider: "nomos.test.integration_seams.dependency",
        guarantee: Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::Project),
    };
}

/// The `nomos_cap_controlflow` seam's payload: one reachability site whose arm is empty, the
/// shape [`nomos_rules::Check_Unread_Reaches_A_Finding`] reports.
fn Controlflow_Payload() -> Payload
{
    let reachability = nomos_cap_controlflow::ReachabilityPayload {
        sites: vec![nomos_cap_controlflow::ReachabilitySite {
            function: "Payload_Of".to_owned(),
            binding: "applicability".to_owned(),
            shape: nomos_cap_controlflow::ArmShape::Empty,
        }],
    };

    return Payload { schema: nomos_cap_controlflow::Payload_Schema(), bytes: nomos_cap_controlflow::Encode_Payload(&reachability) };
}

/// `nomos_cap_controlflow`'s own offer: a syntactic, sound, file-scoped fact that is not
/// claimed complete (its assurance about *missing* sites is `Unsound`).
fn Controlflow_Offered() -> Offered<'static>
{
    return Offered {
        contract: nomos_cap_controlflow::Capability_Contract(),
        capability: nomos_cap_controlflow::Capability(),
        version: nomos_cap_controlflow::CONTRACT_VERSION,
        provider: "nomos.test.integration_seams.controlflow",
        guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File),
    };
}

/// The `nomos_cap_lint` seam's payload: one lint diagnostic at warning level, in this
/// crate's own `lib.rs`.
fn Lint_Payload() -> Payload
{
    let diagnostics = nomos_cap_lint::DiagnosticsPayload {
        package: "nomos-rules".to_owned(),
        diagnostics: vec![nomos_cap_lint::LintDiagnostic {
            level: nomos_cap_lint::LintLevel::Warning,
            lint: Some("clippy::needless_return".to_owned()),
            message: "unneeded `return` statement".to_owned(),
            file: "crates/rules/nomos-rules/src/lib.rs".to_owned(),
            line: 1,
        }],
    };

    return Payload { schema: nomos_cap_lint::Payload_Schema(), bytes: nomos_cap_lint::Encode_Payload(&diagnostics) };
}

/// `nomos_cap_lint`'s own offer: a semantically resolved, project-scoped fact.
fn Lint_Offered() -> Offered<'static>
{
    return Offered {
        contract: nomos_cap_lint::Capability_Contract(),
        capability: nomos_cap_lint::Capability(),
        version: nomos_cap_lint::CONTRACT_VERSION,
        provider: "nomos.test.integration_seams.lint",
        guarantee: Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::Project),
    };
}

/// The `nomos_cap_dependency_policy` seam's payload: one violation of the `duplicate` code,
/// with no target.
fn Policy_Payload() -> Payload
{
    let policy = nomos_cap_dependency_policy::PolicyPayload {
        violations: vec![nomos_cap_dependency_policy::PolicyViolation {
            severity: nomos_cap_dependency_policy::PolicySeverity::Warning,
            code: "duplicate".to_owned(),
            message: "found 2 duplicate entries for crate 'syn'".to_owned(),
            target: None,
        }],
    };

    return Payload { schema: nomos_cap_dependency_policy::Payload_Schema(), bytes: nomos_cap_dependency_policy::Encode_Payload(&policy) };
}

/// `nomos_cap_dependency_policy`'s own offer: a semantically resolved fact about the whole
/// workspace.
fn Policy_Offered() -> Offered<'static>
{
    return Offered {
        contract: nomos_cap_dependency_policy::Capability_Contract(),
        capability: nomos_cap_dependency_policy::Capability(),
        version: nomos_cap_dependency_policy::CONTRACT_VERSION,
        provider: "nomos.test.integration_seams.policy",
        guarantee: Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::WholeWorkspace),
    };
}

/// The seam with `nomos_analysis`, `nomos_capability`, `nomos_contracts`, `nomos_cap_syntax`,
/// `nomos_model` and `nomos_lang_rust` together: a real parse of real source text, read back
/// through a real registry, store and reader, and judged by [`nomos_rules::Check_Completeness_Mirrors`]
/// exactly the way `nomos-check-orchestration` really calls in.
#[test]
fn Test_Check_Completeness_Mirrors_Should_Judge_A_Real_Parse_Through_The_Public_Reader()
{
    let text = "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n";
    let source = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), text);
    let admission = Admission_Of(Syntax_Offered(), &source, &[text.as_bytes()], Syntax_Payload(text));
    let mut reader = Reader::On(&admission.store, &admission.registry, Fixed_Context());
    let findings = nomos_rules::Check_Completeness_Mirrors(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "TABLES");
    assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Blocking, "a mirror naming a check nothing declares must block");
}

/// The seam with `nomos_cap_dependency`: a real dependency-edges fact, judged by
/// [`nomos_rules::Check_Dependency_Direction`] through the public reader.
#[test]
fn Test_Check_Dependency_Direction_Should_Judge_A_Real_Dependency_Fact_Through_The_Public_Reader()
{
    let source = Source("nomos-cap-syntax");
    let admission = Admission_Of(Dependency_Offered(), &source, &[], Dependency_Payload());
    let mut reader = Reader::On(&admission.store, &admission.registry, Fixed_Context());
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
    let admission = Admission_Of(Controlflow_Offered(), &source, &[source.text.as_bytes()], Controlflow_Payload());
    let mut reader = Reader::On(&admission.store, &admission.registry, Fixed_Context());
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
    let admission = Admission_Of(Lint_Offered(), &source, &[], Lint_Payload());
    let mut reader = Reader::On(&admission.store, &admission.registry, Fixed_Context());
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
    let admission = Admission_Of(Policy_Offered(), &source, &[], Policy_Payload());
    let mut reader = Reader::On(&admission.store, &admission.registry, Fixed_Context());
    let findings = nomos_rules::Check_Dependency_Policy(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings.first().expect("asserted len 1 above").summary.contains("duplicate"));
}
