//! A workspace member's own dependencies must run strictly downward, the same property
//! `tests/contract/tests/boundaries/graph.rs`'s `Test_Dependencies_Should_Run_Strictly_
//! Downward` already enforces for this repository by hand.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! `OD-RULES-003` designed this: a declared architecture is data, the observed dependency
//! graph is a fact a capability provider establishes, and a rule composes the two into
//! findings. This is that rule. `run.rs`'s `Run` calls [`Check_Dependency_Direction`]
//! directly over the edges `Materialize_Dependencies` wrote, wired by
//! `P13-DEPENDENCY-WIRE-1` the same way `nomos_lang_rust_cargo`'s own provider was wired in
//! ahead of it — `nomos check` judges dependency direction as part of an ordinary run.
//!
//! # `BANDS` is a second copy, deliberately, for now
//!
//! [`bands::BANDS`] is a copy of `tests/contract/tests/boundaries/bands.rs`'s own table.
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
//!
//! # Split by responsibility
//!
//! [`bands`] holds this workspace's own declared architecture table and the lookup over it.
//! [`reading`] requires and decodes one member's dependency fact. [`violations`] judges an
//! already-decoded payload against the declared bands for *direction*; [`completeness`]
//! judges the same payload for *coverage* — whether its own package has a declared band at
//! all. This file keeps only what composes the pieces: the rules' own identifiers and
//! [`Check_Dependency_Direction`]/[`Check_Every_Member_Declares_A_Band`] themselves, plus
//! the end-to-end tests that exercise each through a real reader.
//!
//! # A second rule over the same fact
//!
//! `violations.rs`'s own `Violations_In` has always silently produced no findings for a
//! package with no declared band, naming the gap as a different defect —
//! `tests/contract/tests/boundaries/graph.rs`'s own `Test_Every_Member_Should_Declare_A_Band`
//! already enforces it by hand, for this repository alone.
//! [`Check_Every_Member_Declares_A_Band`] promotes that gap to a Finding-producing judgment
//! reachable through an ordinary `nomos check` run, over whatever workspace supplies the
//! fact — the same declared-architecture-vs-observed-fact shape `OD-RULES-003` designed for
//! direction, applied to coverage instead. No new capability and no new provider: it reads
//! the identical `nomos.cap.dependency.edges` fact and the identical [`bands::BANDS`] table
//! [`Check_Dependency_Direction`] already reads.

mod bands;
mod completeness;
mod reading;
mod violations;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;
use reading::Payload_Of;
use violations::Violations_In;

/// This rule's own identifier.
pub const DEPENDENCY_DIRECTION: &str = "dependency-direction";

/// [`Check_Every_Member_Declares_A_Band`]'s own identifier.
pub const DEPENDENCY_COMPLETENESS: &str = "dependency-completeness";

/// The record this implementation's contract is written in.
///
/// `PKG-014` requires every rule implementation be traceable to one contract version and
/// mechanically checked for consistency with it, and `OD-PACKAGE-001` measured what happens
/// without it: the requirement sits asserted in the record's own prose and nowhere in the
/// crate that implements it. `tests/contract/tests/rule_contract_citation.rs` reads
/// `OD-RULES-003`'s own front matter on every run and compares it against
/// [`DEPENDENCY_CONTRACT_RECORD_VERSION`], so an amendment this implementation has not caught
/// up to is a red test rather than silent drift.
///
/// The name carries the rule's prefix where `mirror.rs`'s [`crate::CONTRACT_RECORD`] does
/// not. That asymmetry is deliberate: the unprefixed pair was this crate's only citation when
/// it was written and is published, and renaming a published constant for symmetry is churn
/// with no defect behind it. A third cited rule is the point at which extracting a shared
/// citation shape stops being a generalization from two.
///
/// [`Check_Every_Member_Declares_A_Band`] cites this same record rather than a record of its
/// own: it is `OD-RULES-003`'s identical declared-architecture-vs-observed-fact design,
/// judging coverage instead of direction, not a second design decision needing a second
/// citation.
pub const DEPENDENCY_CONTRACT_RECORD: &str = "OD-RULES-003";

/// The version of [`DEPENDENCY_CONTRACT_RECORD`] this implementation was written against.
pub const DEPENDENCY_CONTRACT_RECORD_VERSION: u32 = 1;

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
        match Payload_Of(source, facts, DEPENDENCY_DIRECTION)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, source);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Judges whether every workspace member `sources` names has declared where it sits in
/// this workspace's own band ordering — the coverage half of architecture conformance,
/// left to `tests/contract`'s own `Test_Every_Member_Should_Declare_A_Band` until now.
///
/// Reads the identical fact and requirement [`Check_Dependency_Direction`] does, through
/// the same [`Payload_Of`], and files an unread subject under its own identifier rather
/// than direction's.
#[must_use]
pub fn Check_Every_Member_Declares_A_Band(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts, DEPENDENCY_COMPLETENESS)
        {
            Ok(payload) =>
            {
                let violations = completeness::Violations_In(&payload, source);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// [`Check_Dependency_Direction`] and [`Check_Every_Member_Declares_A_Band`] themselves,
/// through a real registry, store and reader — the half [`violations::tests`] and
/// [`completeness::tests`] do not reach.
#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore, Reader,
    };
    use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, Digest128, EvidenceClass, FactVariant, GenerationId,
        Guarantee, IncrementalGranularity, ProviderId, RuleId, SnapshotId, SubjectId,
    };
    use nomos_model::Content_Digest;

    const PROVIDER: &str = "nomos.test.dependency.resolves";

    fn Dependency_Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge {
            target: target.to_owned(),
            kind: DependencyKind::Normal,
            optional: false,
        };
    }

    #[test]
    fn Test_A_Real_Fact_Should_Be_Read_And_Judged()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload {
                package: "nomos-cap-syntax".to_owned(),
                edges: vec![Dependency_Edge("nomos-rules")],
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
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
    }

    #[test]
    fn Test_A_Declared_Package_Should_Produce_No_Completeness_Finding()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload {
                package: "nomos-cap-syntax".to_owned(),
                edges: Vec::new(),
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A fresh fact store, registry, and the one [`ProviderOffer`] declared into it — named
    /// so a call site reads `offering.store`, not a position it has to count.
    struct TestOffering
    {
        store: MemoryFactStore,
        registry: Registry,
        offer: ProviderOffer,
    }

    #[test]
    fn Test_An_Undeclared_Package_Should_Produce_One_Completeness_Finding()
    {
        let source = Source_File("not-in-bands");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload {
                package: "not-in-bands".to_owned(),
                edges: Vec::new(),
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(DEPENDENCY_COMPLETENESS));
    }

    #[test]
    fn Test_A_Completeness_Subject_With_No_Fact_Should_Be_Reported_Under_Its_Own_Rule()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(
            found.rule,
            RuleId::New(DEPENDENCY_COMPLETENESS),
            "an unread subject must be filed under whichever rule asked, not always direction's"
        );
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

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

    fn Offering() -> TestOffering
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

        return TestOffering { store: MemoryFactStore::New(), registry, offer };
    }

    fn Materialize_Dependency_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DependencyPayload)
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
}
