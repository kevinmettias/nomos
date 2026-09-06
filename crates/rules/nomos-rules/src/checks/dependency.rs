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
//! # `zones` is the one declaration, not a second copy
//!
//! `OD-RULES-020` decided a total order over band numbers claims precedence between crates
//! that have none, and `OD-RULES-020`'s own migration item made [`zones::ZONES`] this
//! workspace's one declared architecture: `tests/contract/tests/boundaries/bands.rs` and
//! `graph.rs` read it through this crate's own public surface rather than keeping their
//! own copy, and `README.md`'s own table is checked against the identical declaration.
//! What `OD-RULES-003` named as the expected interim state — one Rust-source table,
//! generalizing the format only once a second repository wants this property — is
//! unchanged by closing the copy; only the number of times that one table is typed out by
//! hand changed, from three to one.
//!
//! # Scope
//!
//! First-party edges only — [`nomos_cap_dependency`]'s provider already filters to
//! workspace members, so this rule never sees an external (registry) dependency to judge.
//! `graph.rs`'s `CONTRACTS_ALLOWLIST` and `PLATFORM_ADAPTER` exceptions are not
//! replicated: both are about reach *outside* this workspace's own zones, which this
//! capability does not observe at all, so there is nothing here for them to except.
//!
//! # Split by responsibility
//!
//! [`zones`] holds this workspace's own declared architecture table and the lookup over
//! it. [`reading`] requires and decodes one member's dependency fact. [`violations`]
//! judges an already-decoded payload against the declared zones for *direction*;
//! [`completeness`] judges the same payload for *coverage* — whether its own package has a
//! declared zone at all; [`write_authority`] judges the same payload for *authority* —
//! whether an edge into a declared write door crate comes from one of the doors named for
//! it. This file keeps only what composes the pieces: the rules' own identifiers and
//! [`Check_Dependency_Direction`]/[`Check_Every_Member_Declares_A_Band`]/
//! [`Check_Write_Authority`] themselves, plus the end-to-end tests that exercise each
//! through a real reader.
//!
//! # A second rule over the same fact
//!
//! `violations.rs`'s own `Violations_In` has always silently produced no findings for a
//! package with no declared zone, naming the gap as a different defect —
//! `tests/contract/tests/boundaries/graph.rs`'s own `Test_Every_Member_Should_Declare_A_Band`
//! already enforces it by hand, for this repository alone.
//! [`Check_Every_Member_Declares_A_Band`] promotes that gap to a Finding-producing judgment
//! reachable through an ordinary `nomos check` run, over whatever workspace supplies the
//! fact — the same declared-architecture-vs-observed-fact shape `OD-RULES-003` designed for
//! direction, applied to coverage instead. No new capability and no new provider: it reads
//! the identical `nomos.cap.dependency.edges` fact and the identical [`zones::ZONES`] table
//! [`Check_Dependency_Direction`] already reads.
//!
//! # A third rule over the same fact
//!
//! `OD-RULES-023` decided a third judgment belonged here rather than in a new capability:
//! `nomos-store`'s own "one write door per authority" design was true but unchecked, and
//! [`write_authority::WRITE_DOORS`] is an allow-list over the identical
//! `nomos.cap.dependency.edges` fact, checked the same way [`zones::Permits`] already checks
//! a crossing — named authority instead of named direction. [`Check_Write_Authority`] cites
//! `OD-RULES-023` rather than `OD-RULES-003`: unlike coverage, this is new decision content,
//! not the same design applied again.

mod completeness;
mod reading;
mod violations;
mod write_authority;
mod zones;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;
use reading::Payload_Of;
use violations::Violations_In;

pub use write_authority::WRITE_DOORS;
pub use zones::{Permits, Zone, Zone_Of, ALL as ZONE_LIST, SAME_ZONE_EDGES, ZONES};

/// This rule's own identifier.
pub const DEPENDENCY_DIRECTION: &str = "dependency-direction";

/// [`Check_Every_Member_Declares_A_Band`]'s own identifier.
pub const DEPENDENCY_COMPLETENESS: &str = "dependency-completeness";

/// [`Check_Write_Authority`]'s own identifier.
pub const WRITE_AUTHORITY: &str = "dependency-write-authority";

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

/// The record [`Check_Write_Authority`]'s own contract is written in.
///
/// A record of its own rather than a second citation of [`DEPENDENCY_CONTRACT_RECORD`]:
/// `OD-RULES-023` is new decision content -- a declared allow-list checked against the
/// existing dependency-edges fact -- not `OD-RULES-003`'s design applied again the way
/// [`Check_Every_Member_Declares_A_Band`] is. `tests/contract/tests/rule_contract_
/// citation.rs` reads `OD-RULES-023`'s own front matter on every run and compares it
/// against [`WRITE_AUTHORITY_CONTRACT_RECORD_VERSION`].
pub const WRITE_AUTHORITY_CONTRACT_RECORD: &str = "OD-RULES-023";

/// The version of [`WRITE_AUTHORITY_CONTRACT_RECORD`] this implementation was written
/// against.
pub const WRITE_AUTHORITY_CONTRACT_RECORD_VERSION: u32 = 1;

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

/// Judges whether every workspace member `sources` names, that depends on a crate
/// [`write_authority::WRITE_DOORS`] declares an authority, is one of the doors named for
/// it -- the authority half of architecture conformance `OD-RULES-023` decided.
///
/// Reads the identical fact and requirement [`Check_Dependency_Direction`] does, through
/// the same [`Payload_Of`], and files an unread subject under its own identifier rather
/// than direction's.
#[must_use]
pub fn Check_Write_Authority(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts, WRITE_AUTHORITY)
        {
            Ok(payload) =>
            {
                let violations = write_authority::Violations_In(&payload, source);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// [`Check_Dependency_Direction`], [`Check_Every_Member_Declares_A_Band`] and
/// [`Check_Write_Authority`] themselves,
/// through a real registry, store and reader — the half [`violations::tests`] and
/// [`completeness::tests`] do not reach.
#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
    use nomos_capability::ProviderOffer;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, RuleId, SubjectId};
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

    /// Also [`reading::Dependency_Requirement`]'s own shape: the registered guarantee is
    /// built to exactly meet that floor, so a real fact only reaches this rule because the
    /// floor admits it.
    #[test]
    fn Test_Check_Dependency_Direction_Should_Read_And_Judge_A_Real_Fact_Whose_Guarantee_Meets_Dependency_Requirement()
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

    /// Also [`reading::Dependency_Requirement`]'s own shape: a provider is registered
    /// against exactly that floor, so the subject is reported unread for want of a
    /// materialized fact rather than for want of an admitted provider.
    #[test]
    fn Test_Dependency_Requirement_Should_Be_Registered_Yet_Report_A_Subject_With_No_Fact()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
    }

    #[test]
    fn Test_Check_Every_Member_Declares_A_Band_Should_Produce_No_Finding_For_A_Declared_Package()
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
    fn Test_Payload_Of_Should_Report_An_Unread_Completeness_Subject_Under_Its_Own_Rule()
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

    #[test]
    fn Test_Check_Write_Authority_Should_Produce_No_Finding_For_A_Named_Door()
    {
        let source = Source_File("nomos-workspace");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload {
                package: "nomos-workspace".to_owned(),
                edges: vec![Dependency_Edge("nomos-store")],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Write_Authority_Should_Produce_One_Finding_For_An_Undeclared_Door()
    {
        let source = Source_File("nomos-cli");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Dependency_Fact(
            &mut store,
            &source,
            &offer,
            &DependencyPayload {
                package: "nomos-cli".to_owned(),
                edges: vec![Dependency_Edge("nomos-store")],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(WRITE_AUTHORITY));
    }

    #[test]
    fn Test_Payload_Of_Should_Report_An_Unread_Write_Authority_Subject_Under_Its_Own_Rule()
    {
        let source = Source_File("nomos-workspace");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(
            found.rule,
            RuleId::New(WRITE_AUTHORITY),
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

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_dependency::Capability_Contract(),
            nomos_cap_dependency::Capability(),
            nomos_cap_dependency::CONTRACT_VERSION,
            PROVIDER,
            Guarantee_At_Floor(),
        );
    }

    fn Materialize_Dependency_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DependencyPayload)
    {
        let bytes = nomos_cap_dependency::Encode_Payload(payload);
        test_support::Materialize(store, source.subject, offer, InputDigest::Of(&[]), nomos_cap_dependency::Payload_Schema(), bytes);
    }
}
