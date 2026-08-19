//! A workspace member's own dependencies must run strictly downward, the same property
//! `tests/contract/tests/boundaries/graph.rs`'s `Test_Dependencies_Should_Run_Strictly_
//! Downward` already enforces for this repository by hand.
//!
//! # What this rule is, and what it is not yet
//!
//! `OD-RULES-003` designed this: a declared architecture is data, the observed dependency
//! graph is a fact a capability provider establishes, and a rule composes the two into
//! findings. This is that rule. It is not yet composed into `nomos-check-orchestration`'s
//! `Run` — `nomos_lang_rust_cargo`'s own module doc says why (that provider performs I/O
//! `nomos-check-orchestration` is deliberately built without, and wiring it in means
//! touching `nomos-cli`, held live by a concurrent item at the time this rule was
//! written). This rule stands and is tested on its own, the same "additive and unwired"
//! shape `RuleRegistry` carried from `OD-RULES-004`.
//!
//! # `BANDS` is a second copy, deliberately, for now
//!
//! [`BANDS`] below is a copy of `tests/contract/tests/boundaries/bands.rs`'s own table.
//! `OD-RULES-003` names exactly this as the expected interim state: a declared
//! architecture generalized into a portable, non-Rust-source format is real work this
//! record does not schedule, and this workspace has exactly one declared architecture to
//! check today. Duplicating it as rule-authored data — the same shape
//! `nomos_rules::CONTRACT_RECORD` already is for `Check_Completeness_Mirrors` — costs a
//! table that can drift; generalizing the format now would be designing it from a
//! population of one, the mistake `OD-PACKAGE-006` and `OD-PACKAGE-008` both already
//! declined elsewhere. A second repository wanting this property is the trigger for that
//! generalization, not a hypothesis to build ahead of one.
//!
//! # Scope
//!
//! First-party edges only — [`nomos_cap_dependency`]'s provider already filters to
//! workspace members, so this rule never sees an external (registry) dependency to judge.
//! `bands.rs`'s `CONTRACTS_ALLOWLIST` and `PLATFORM_ADAPTER` exceptions are not
//! replicated: both are about reach *outside* this workspace's own band ordering, which
//! this capability does not observe at all, so there is nothing here for them to except.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_capability::Requirement;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const DEPENDENCY_DIRECTION: &str = "dependency-direction";

/// This workspace's own declared architecture, copied from
/// `tests/contract/tests/boundaries/bands.rs`'s `BANDS` at the time this rule was
/// written. See this module's own doc comment for why a copy and not a shared source.
const BANDS: &[(&str, u32)] = &[
    ("nomos-contracts", 0),
    ("nomos-model", 10),
    ("nomos-store", 12),
    ("nomos-platform", 15),
    ("nomos-platform-std", 16),
    ("nomos-workspace", 18),
    ("nomos-ledger", 20),
    ("nomos-capability", 21),
    ("nomos-analysis", 22),
    ("nomos-cap-syntax", 23),
    ("nomos-cap-dependency", 23),
    ("nomos-package", 24),
    ("nomos-lang-rust", 25),
    ("nomos-lang-rust-scan", 25),
    ("nomos-lang-rust-cargo", 25),
    ("nomos-lang-package", 26),
    ("nomos-spec-model", 11),
    ("nomos-spec-store", 12),
    ("nomos-spec-bundle", 13),
    ("nomos-spec-ingest", 13),
    ("nomos-spec-validate", 14),
    ("nomos-spec-project", 14),
    ("nomos-rules", 30),
    ("nomos-corrections", 35),
    ("nomos-work-orchestration", 40),
    ("nomos-check-orchestration", 40),
    ("nomos-spec-orchestration", 40),
    ("nomos-gate-orchestration", 40),
    ("nomos-cli", 90),
    ("nomos-surface-provenance", 91),
    ("nomos-contract-tests", 100),
    ("nomos-integration-tests", 100),
];

#[must_use]
fn Declared_Band(name: &str) -> Option<u32>
{
    return BANDS
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, band)| *band);
}

/// What this rule needs from `nomos.cap.dependency.edges` before it will believe an
/// answer.
///
/// The ceiling itself: nothing weaker than Cargo's own resolution could tell a first-party
/// path dependency apart from an unrelated registry crate of the same name, and a rule
/// judging architecture cannot afford that ambiguity. Soundness `Sound` because every edge
/// this rule acts on must really be a resolved one; completeness `Sound` because
/// `nomos-lang-rust-cargo`'s own guarantee states it honestly achieves that, unlike a
/// syntax provider bounded by what a macro might hide.
#[must_use]
pub(crate) fn Dependency_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
    );

    return Requirement::New(
        nomos_cap_dependency::Capability(),
        nomos_cap_dependency::CONTRACT_VERSION,
        guarantee,
    );
}

/// Judges every workspace member `sources` names against this workspace's own declared
/// band ordering.
///
/// One fact per source, the same shape [`crate::Check_Naming_Convention`] reads. A source
/// here is a workspace member, not a file — its `subject` is the member's own subject
/// (the same [`nomos_model::Subject_Of_Path`] the provider keys its fact under), and its
/// `text` is unread: this rule's whole judgment comes from the fact, never from
/// `source.text`.
#[must_use]
pub fn Check_Dependency_Direction(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, source)),
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// One member's decoded dependency payload, or a finding reporting why it could not be
/// read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<DependencyPayload, Finding>
{
    let need = Dependency_Requirement();
    let capability = nomos_cap_dependency::Capability();
    // The provider's semantic input is the encoded payload it wrote, not `source.text` —
    // this rule does not know that encoding and must not guess it to key a lookup. A
    // reader resolves a fact by subject and requirement, not by recomputing the
    // provider's own input digest, so an empty digest here names nothing this rule is
    // required to get right.
    let inputs = InputDigest::Of(&[]);

    let fact = match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => fact,
        Err(applicability) =>
        {
            return Err(Unread(
                source,
                applicability,
                &format!("no admitted provider answered for it ({})", applicability.Label()),
            ));
        }
    };

    if fact.payload.schema != nomos_cap_dependency::Payload_Schema()
    {
        return Err(Unread(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this member carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return nomos_cap_dependency::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread(source, Applicability::Unparseable, &refusal.to_string()));
}

fn Unread(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(DEPENDENCY_DIRECTION),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this member's dependency direction could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// Every edge `payload` declares that runs same-band or upward, as findings.
///
/// A pure function of an already-decoded payload, testable against hand-built fixtures —
/// no registry, no store, no reader — the same split [`crate::naming::Violations_In`]
/// draws for the same reason.
fn Violations_In(payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    let Some(band) = Declared_Band(&payload.package)
    else
    {
        // Undeclared entirely — a different defect from a wrong-direction edge, and
        // already `tests/contract`'s own `Test_Every_Member_Should_Declare_A_Band`'s
        // subject. Judging direction from an unknown starting band would be a guess this
        // rule is not entitled to make.
        return Vec::new();
    };

    let mut findings = Vec::new();
    for edge in &payload.edges
    {
        // A dev-dependency does not ship, so it is not part of the graph this judgment is
        // about — `tests/contract/src/workspace.rs`'s own `Is_Not_Dev` excludes it from
        // `graph.rs`'s identical downward-ordering check for exactly this reason, and
        // `nomos-spec-ingest`'s own `Cargo.toml` names the real case this rule would
        // otherwise misjudge: a band-13 crate's dev-only dependency on band-14's validator,
        // present only so its own test suite can exercise a preservation run.
        if edge.kind == DependencyKind::Dev
        {
            continue;
        }

        let Some(dependency_band) = Declared_Band(&edge.target)
        else
        {
            continue;
        };

        if dependency_band >= band
        {
            findings.push(Violation(source, &payload.package, band, edge, dependency_band));
        }
    }

    return findings;
}

fn Violation(
    source: &SourceFile,
    package: &str,
    band: u32,
    edge: &DependencyEdge,
    dependency_band: u32,
) -> Finding
{
    return Finding {
        rule: RuleId::New(DEPENDENCY_DIRECTION),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} (band {band}) depends on {} (band {dependency_band}). Dependencies \
             run strictly downward; equal or upward edges are how a layered architecture \
             becomes a graph nobody can reason about.",
            edge.target
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::Content_Digest;
    use nomos_contracts::SubjectId;

    fn Source(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

    fn Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge {
            target: target.to_owned(),
            kind: DependencyKind::Normal,
            optional: false,
        };
    }

    fn Dev_Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge {
            target: target.to_owned(),
            kind: DependencyKind::Dev,
            optional: false,
        };
    }

    mod judging
    {
        use super::*;

        #[test]
        fn Test_A_Strictly_Downward_Edge_Should_Produce_No_Finding()
        {
            let payload = DependencyPayload {
                package: "nomos-rules".to_owned(),
                edges: vec![Edge("nomos-cap-syntax")],
            };

            let findings = Violations_In(&payload, &Source("nomos-rules"));

            assert!(findings.is_empty(), "{findings:?}");
        }

        #[test]
        fn Test_An_Upward_Edge_Should_Produce_One_Finding()
        {
            let payload = DependencyPayload {
                package: "nomos-cap-syntax".to_owned(),
                edges: vec![Edge("nomos-rules")],
            };

            let findings = Violations_In(&payload, &Source("nomos-cap-syntax"));

            assert_eq!(findings.len(), 1, "{findings:?}");
            let found = findings.first().expect("asserted len 1 above");
            assert_eq!(found.subject_name, "nomos-cap-syntax");
            assert_eq!(found.gate, GateCategory::Advisory);
        }

        #[test]
        fn Test_A_Same_Band_Edge_Should_Produce_One_Finding()
        {
            let payload = DependencyPayload {
                package: "nomos-lang-rust".to_owned(),
                edges: vec![Edge("nomos-lang-rust-scan")],
            };

            let findings = Violations_In(&payload, &Source("nomos-lang-rust"));

            assert_eq!(
                findings.len(),
                1,
                "two providers of one capability must not be able to name each other: {findings:?}"
            );
        }

        #[test]
        fn Test_A_Dev_Dependency_Running_Upward_Should_Produce_No_Finding()
        {
            // The real case this guards: nomos-spec-ingest (13) dev-depends on
            // nomos-spec-validate (14) so its own test suite can run a preservation
            // check, and `Cargo.toml`'s own comment there says exactly why this must not
            // read as a violation.
            let payload = DependencyPayload {
                package: "nomos-spec-ingest".to_owned(),
                edges: vec![Dev_Edge("nomos-spec-validate")],
            };

            let findings = Violations_In(&payload, &Source("nomos-spec-ingest"));

            assert!(
                findings.is_empty(),
                "a dev-dependency does not ship and must not be judged as an architecture \
                 edge: {findings:?}"
            );
        }

        #[test]
        fn Test_A_Package_With_No_Declared_Band_Should_Produce_No_Finding()
        {
            let payload = DependencyPayload {
                package: "not-in-bands".to_owned(),
                edges: vec![Edge("nomos-rules")],
            };

            let findings = Violations_In(&payload, &Source("not-in-bands"));

            assert!(
                findings.is_empty(),
                "an undeclared band is a different defect, judged elsewhere: {findings:?}"
            );
        }

        #[test]
        fn Test_An_Edge_To_An_Undeclared_Target_Should_Produce_No_Finding()
        {
            let payload = DependencyPayload {
                package: "nomos-rules".to_owned(),
                edges: vec![Edge("not-in-bands")],
            };

            let findings = Violations_In(&payload, &Source("nomos-rules"));

            assert!(findings.is_empty(), "{findings:?}");
        }

        #[test]
        fn Test_A_Package_With_No_Edges_Should_Produce_No_Finding()
        {
            let payload = DependencyPayload {
                package: "nomos-contracts".to_owned(),
                edges: Vec::new(),
            };

            let findings = Violations_In(&payload, &Source("nomos-contracts"));

            assert!(findings.is_empty(), "{findings:?}");
        }
    }

    mod bands_table
    {
        use super::*;

        #[test]
        fn Test_Every_Entry_Should_Be_Findable_By_Name()
        {
            for (name, band) in BANDS
            {
                assert_eq!(Declared_Band(name), Some(*band), "{name}");
            }
        }

        #[test]
        fn Test_An_Unknown_Name_Should_Have_No_Declared_Band()
        {
            assert_eq!(Declared_Band("nomos-does-not-exist"), None);
        }
    }

    /// [`Check_Dependency_Direction`] itself, through a real registry, store and reader —
    /// the half [`Violations_In`]'s own tests do not reach.
    mod reading_a_fact
    {
        use super::*;
        use nomos_analysis::{
            Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader,
        };
        use nomos_capability::{ProviderOffer, Registry};
        use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, ProviderId, SnapshotId};

        const PROVIDER: &str = "nomos.test.dependency.resolves";

        fn Guarantee_At_Floor() -> Guarantee
        {
            return Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Project,
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

        fn Offering() -> (MemoryFactStore, Registry, ProviderOffer)
        {
            let mut registry = Registry::New();
            registry
                .Declare(nomos_cap_dependency::Capability_Contract())
                .expect("the dependency capability is declared once");

            let offer = ProviderOffer {
                provider: ProviderId::New(PROVIDER),
                capability: nomos_cap_dependency::Capability(),
                version: nomos_cap_dependency::CONTRACT_VERSION,
                guarantee: Guarantee_At_Floor(),
            };
            registry.Offer(offer.clone()).expect("within the ceiling");

            return (MemoryFactStore::New(), registry, offer);
        }

        fn Materialize(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DependencyPayload)
        {
            let context = Test_Context();
            let bytes = nomos_cap_dependency::Encode_Payload(payload);
            let key = FactKey {
                contract: nomos_cap_dependency::Capability(),
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
                    MaterializedFact {
                        identity: key.At(context.generation),
                        snapshot: context.snapshot,
                        evidence: EvidenceClass::Verified,
                        guarantee: offer.guarantee,
                        payload: FactPayload::New(nomos_cap_dependency::Payload_Schema(), bytes),
                    },
                    &[],
                )
                .expect("nothing here is backdated");
        }

        #[test]
        fn Test_A_Real_Fact_Should_Be_Read_And_Judged()
        {
            let source = Source("nomos-cap-syntax");
            let (mut store, registry, offer) = Offering();
            Materialize(
                &mut store,
                &source,
                &offer,
                &DependencyPayload {
                    package: "nomos-cap-syntax".to_owned(),
                    edges: vec![Edge("nomos-rules")],
                },
            );

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Dependency_Direction(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "nomos-cap-syntax");
        }

        #[test]
        fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
        {
            let source = Source("nomos-cap-syntax");
            let (store, registry, _offer) = Offering();

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Dependency_Direction(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        }
    }
}
