//! A workspace member's own dependencies must run the way its repository says they may.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! `OD-RULES-003` designed this: a declared architecture is data, the observed dependency
//! graph is a fact a capability provider establishes, and a rule composes the two into
//! findings. This is that rule. `run_context.rs`'s `Run` calls [`Check_Dependency_Direction`]
//! directly over the edges `Materialize_Dependencies` wrote, wired by `P13-DEPENDENCY-WIRE-1`
//! the same way `nomos_lang_rust_cargo`'s own provider was wired in ahead of it — `nomos check`
//! judges dependency direction as part of an ordinary run.
//!
//! # Both halves are now facts, and that is the change
//!
//! `OD-RULES-003` named three prerequisites and left one unbuilt: "a place to author a declared
//! architecture as data rather than a Rust `const` table." Until it existed, this module held
//! that table — `ZONES`, a literal of sixty-six entries every one of which was a crate name of
//! this workspace's, plus a `Zone` enum whose variants were this workspace's own architectural
//! vocabulary and whose doc comments named this workspace's own crates. `OD-RULES-029` measured
//! what that cost: on any other repository nothing resolved, so `Check_Dependency_Direction`
//! silently judged nothing while `Check_Every_Member_Declares_A_Band` reported one finding per
//! member saying it had declared nothing — a declaration that repository was never asked for.
//!
//! Both halves now arrive through the reader. `nomos.cap.architecture.declaration` carries the
//! whole triple `OD-RULES-029` insisted travels together: the components a repository divides
//! itself into, the order over them, and the named exceptions that order cannot express. No
//! component name and no crate name of this workspace's remains here, and none of the three
//! rules below could tell you what a zone is.
//!
//! # Scope
//!
//! First-party edges only — [`nomos_cap_dependency`]'s provider already filters to workspace
//! members, so this rule never sees an external (registry) dependency to judge.
//!
//! # Split by responsibility
//!
//! [`reading`] requires and decodes both facts. [`violations`] judges an already-decoded
//! payload against the declaration for *direction*; [`completeness`] judges the same payload
//! for *coverage* — whether its own package is placed at all; [`write_authority`] judges it for
//! *authority* — whether an edge into a declared authority comes from one of its named doors.
//! This file keeps only what composes the pieces.
//!
//! # A second and third rule over the same two facts
//!
//! [`Check_Every_Member_Declares_A_Band`] promotes to a Finding what `violations` passes over:
//! a member the declaration does not place. `OD-RULES-023` decided the third belonged here
//! rather than in a new capability, and [`Check_Write_Authority`] cites that record rather than
//! `OD-RULES-003` because it is new decision content, not the same design applied again.

mod completeness;
mod reading;
mod violations;
mod write_authority;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;
use reading::{Architecture_Of, Payload_Of};

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
/// The name carries the rule's prefix where `mirror.rs`'s [`crate::CONTRACT_RECORD`] does not.
/// That asymmetry is deliberate: the unprefixed pair was this crate's only citation when it was
/// written and is published, and renaming a published constant for symmetry is churn with no
/// defect behind it.
///
/// [`Check_Every_Member_Declares_A_Band`] cites this same record rather than one of its own: it
/// is `OD-RULES-003`'s identical declared-architecture-vs-observed-fact design, judging coverage
/// instead of direction, not a second design decision needing a second citation.
pub const DEPENDENCY_CONTRACT_RECORD: &str = "OD-RULES-003";

/// The version of [`DEPENDENCY_CONTRACT_RECORD`] this implementation was written against.
pub const DEPENDENCY_CONTRACT_RECORD_VERSION: u32 = 1;

/// The record [`Check_Write_Authority`]'s own contract is written in.
///
/// A record of its own rather than a second citation of [`DEPENDENCY_CONTRACT_RECORD`]:
/// `OD-RULES-023` is new decision content -- a declared allow-list checked against the existing
/// dependency-edges fact -- not `OD-RULES-003`'s design applied again the way
/// [`Check_Every_Member_Declares_A_Band`] is. `tests/contract/tests/rule_contract_citation.rs`
/// reads `OD-RULES-023`'s own front matter on every run and compares it against
/// [`WRITE_AUTHORITY_CONTRACT_RECORD_VERSION`].
pub const WRITE_AUTHORITY_CONTRACT_RECORD: &str = "OD-RULES-023";

/// The version of [`WRITE_AUTHORITY_CONTRACT_RECORD`] this implementation was written against.
pub const WRITE_AUTHORITY_CONTRACT_RECORD_VERSION: u32 = 1;

/// Judges every workspace member `sources` names against the architecture its own repository
/// declares.
///
/// One dependency fact per source, the same shape [`crate::Check_Naming_Convention`] reads, plus
/// one whole-workspace declaration read once for the run. A source here is a workspace member,
/// not a file — its `subject` is the member's own subject — and its `text` is unread: this
/// rule's whole judgment comes from the two facts, never from `source.text`.
#[must_use]
pub fn Check_Dependency_Direction(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match Architecture_Of(sources, facts, DEPENDENCY_DIRECTION)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return Judged(sources, facts, DEPENDENCY_DIRECTION, &|payload, source| {
        return violations::Violations_In(&architecture, payload, source);
    });
}

/// Judges whether every workspace member `sources` names is placed by the architecture its own
/// repository declares — the coverage half of architecture conformance.
///
/// Reads the identical two facts [`Check_Dependency_Direction`] does, through the same readers,
/// and files an unread subject under its own identifier rather than direction's.
#[must_use]
pub fn Check_Every_Member_Declares_A_Band(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match Architecture_Of(sources, facts, DEPENDENCY_COMPLETENESS)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return Judged(sources, facts, DEPENDENCY_COMPLETENESS, &|payload, source| {
        return completeness::Violations_In(&architecture, payload, source);
    });
}

/// Judges whether every workspace member `sources` names that depends on a package its
/// repository declares an authority is one of the doors named for it -- the authority half of
/// architecture conformance `OD-RULES-023` decided.
#[must_use]
pub fn Check_Write_Authority(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match Architecture_Of(sources, facts, WRITE_AUTHORITY)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return Judged(sources, facts, WRITE_AUTHORITY, &|payload, source| {
        return write_authority::Violations_In(&architecture, payload, source);
    });
}

/// Every member's own dependency payload, judged by `judge`, with an unread one reported under
/// `rule` — the shape all three rules above share once the declaration is in hand.
fn Judged(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    rule: &'static str,
    judge: &dyn Fn(&nomos_cap_dependency::DependencyPayload, &SourceFile) -> Vec<Finding>,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts, rule)
        {
            Ok(payload) => findings.extend(judge(&payload, source)),
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));

    return findings;
}


/// The three rules themselves, through a real registry, store and reader — the half
/// [`violations::tests`], [`completeness::tests`] and [`write_authority::tests`] do not reach.
///
/// Every fixture here declares an architecture in a vocabulary this workspace does not use.
/// That is the point rather than a flourish: these assertions could not have been written this
/// way while the components were an enum in this crate, and their reading naturally is the
/// evidence that nothing here knows what a zone is.
#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context};
    use nomos_cap_architecture::ArchitecturePayload;
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_cap_architecture::{Authority, Membership, Permission};
    use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
    use nomos_contracts::{ProviderId, RuleId, SubjectId};
    use nomos_model::Content_Digest;

    const DEPENDENCY_PROVIDER: &str = "nomos.test.dependency.resolves";
    const ARCHITECTURE_PROVIDER: &str = "nomos.test.architecture.declares";

    /// A registry declaring both capabilities these rules read, and a store to materialize into.
    ///
    /// `test_support::Offering` builds a registry around one contract, and these rules need two,
    /// so the declare-then-offer pairing is spelled out here rather than that helper widened for
    /// a single caller.
    struct Fixture
    {
        store: MemoryFactStore,
        registry: Registry,
        dependency: ProviderOffer,
        architecture: ProviderOffer,
    }

    fn Fixture() -> Fixture
    {
        let mut registry = Registry::New();

        let dependency = ProviderOffer {
            provider: ProviderId::New(DEPENDENCY_PROVIDER),
            capability: nomos_cap_dependency::Capability(),
            version: nomos_cap_dependency::CONTRACT_VERSION,
            guarantee: reading::Dependency_Requirement().minimum,
        };
        registry
            .Declare_And_Offer(nomos_cap_dependency::Capability_Contract(), dependency.clone())
            .expect("declared and offered within the ceiling");

        let architecture = ProviderOffer {
            provider: ProviderId::New(ARCHITECTURE_PROVIDER),
            capability: nomos_cap_architecture::Capability(),
            version: nomos_cap_architecture::CONTRACT_VERSION,
            guarantee: nomos_cap_architecture::Ceiling(),
        };
        registry
            .Declare_And_Offer(nomos_cap_architecture::Capability_Contract(), architecture.clone())
            .expect("declared and offered within the ceiling");

        return Fixture { store: MemoryFactStore::New(), registry, dependency, architecture };
    }

    impl Fixture
    {
        /// Files `declared` as the one whole-workspace architecture fact.
        fn Declaring(&mut self, declared: &ArchitecturePayload)
        {
            let bytes = nomos_cap_architecture::Encode_Payload(declared);
            test_support::Materialize(
                &mut self.store,
                nomos_model::Subject_Of_Path(""),
                &self.architecture.clone(),
                InputDigest::Of(&[]),
                nomos_cap_architecture::Payload_Schema(),
                bytes,
            );
        }

        /// Files one member's own dependency edges.
        fn Depending(&mut self, source: &SourceFile, payload: &DependencyPayload)
        {
            let bytes = nomos_cap_dependency::Encode_Payload(payload);
            test_support::Materialize(
                &mut self.store,
                source.subject,
                &self.dependency.clone(),
                InputDigest::Of(&[]),
                nomos_cap_dependency::Payload_Schema(),
                bytes,
            );
        }
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

    fn Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge { target: target.to_owned(), kind: DependencyKind::Normal, optional: false };
    }

    /// A declaration whose components are none of this workspace's.
    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Domain".to_owned(), "Api".to_owned()],
            membership: vec![
                Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "http".to_owned(), component: "Api".to_owned() },
            ],
            permissions: vec![Permission { from: "Api".to_owned(), to: "Domain".to_owned() }],
            exceptions: Vec::new(),
            authorities: vec![Authority { package: "ledger-store".to_owned(), doors: vec!["billing".to_owned()] }],
        };
    }

    #[test]
    fn Test_Check_Dependency_Direction_Should_Read_Both_Real_Facts_And_Judge_An_Inadmissible_Edge()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "billing".to_owned(), edges: vec![Edge("http")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "billing");
    }

    #[test]
    fn Test_Check_Dependency_Direction_Should_Produce_No_Finding_For_An_Edge_The_Declaration_Admits()
    {
        let source = Source_File("http");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "http".to_owned(), edges: vec![Edge("billing")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The property this whole migration is for, as one assertion: replace the repository's
    /// architecture wholesale and the same rule, unchanged, enforces the new one. Here `Api` is
    /// forbidden from reaching `Domain` instead of permitted, and the verdict flips.
    #[test]
    fn Test_The_Same_Rule_Should_Enforce_A_Different_Declaration_Over_The_Same_Edges()
    {
        let source = Source_File("http");
        let mut fixture = Fixture();
        let reversed = ArchitecturePayload {
            permissions: vec![Permission { from: "Domain".to_owned(), to: "Api".to_owned() }],
            ..Declaration()
        };
        fixture.Declaring(&reversed);
        fixture.Depending(&source, &DependencyPayload { package: "http".to_owned(), edges: vec![Edge("billing")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "the edge the previous declaration admitted is refused by this one: {findings:?}");
    }

    /// An unreadable declaration is reported once, not silently treated as "declares nothing"
    /// and not reported per member. `OD-RULES-003` turns on telling those apart.
    #[test]
    fn Test_An_Unreadable_Declaration_Should_Report_One_Finding_Rather_Than_Judge_Nothing()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Depending(&source, &DependencyPayload { package: "billing".to_owned(), edges: vec![Edge("http")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(DEPENDENCY_DIRECTION));
        assert!(found.summary.contains("declared architecture could not be read"), "{}", found.summary);
    }

    #[test]
    fn Test_Dependency_Requirement_Should_Be_Registered_Yet_Report_A_Subject_With_No_Fact()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Dependency_Direction(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
    }

    #[test]
    fn Test_Check_Every_Member_Declares_A_Band_Should_Produce_No_Finding_For_A_Placed_Member()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "billing".to_owned(), edges: Vec::new() });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Unplaced_Member_Should_Produce_One_Completeness_Finding()
    {
        let source = Source_File("unplaced");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "unplaced".to_owned(), edges: Vec::new() });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(DEPENDENCY_COMPLETENESS));
    }

    #[test]
    fn Test_Payload_Of_Should_Report_An_Unread_Completeness_Subject_Under_Its_Own_Rule()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Every_Member_Declares_A_Band(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(DEPENDENCY_COMPLETENESS),
            "an unread subject must be filed under whichever rule asked, not always direction's"
        );
    }

    #[test]
    fn Test_Check_Write_Authority_Should_Produce_No_Finding_For_A_Named_Door()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "billing".to_owned(), edges: vec![Edge("ledger-store")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Write_Authority_Should_Produce_One_Finding_For_An_Undeclared_Door()
    {
        let source = Source_File("http");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());
        fixture.Depending(&source, &DependencyPayload { package: "http".to_owned(), edges: vec![Edge("ledger-store")] });

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(WRITE_AUTHORITY));
    }

    #[test]
    fn Test_Payload_Of_Should_Report_An_Unread_Write_Authority_Subject_Under_Its_Own_Rule()
    {
        let source = Source_File("billing");
        let mut fixture = Fixture();
        fixture.Declaring(&Declaration());

        let mut reader = Reader::On(&fixture.store, &fixture.registry, Test_Context());
        let findings = Check_Write_Authority(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(WRITE_AUTHORITY),
            "an unread subject must be filed under whichever rule asked, not always direction's"
        );
    }
}
