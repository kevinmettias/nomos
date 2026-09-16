//! Judging an already-decoded dependency payload against the repository's own declared write
//! authorities.
//!
//! `OD-RULES-023` decided the mechanism: `nomos-store`'s own README row states "one write door
//! per authority," measured true but unchecked by anything. An authority names, for each crate
//! a repository has declared a sole write door for, exactly which crates may depend on it
//! directly -- the identical shape the component permissions apply to a crossing, aimed at
//! authority instead of direction.
//!
//! What changed since that record is where the table lives. It was a `WRITE_DOORS` constant in
//! this file, naming `nomos-store` and `nomos-workspace` as string literals, which made this
//! rule structurally unable to fire for any repository but this one -- the same defect
//! `OD-RULES-029` measured for the component table beside it, and it travels with that
//! declaration for the same reason. Nothing about what `OD-RULES-023` decided is reopened: a
//! declared allow-list over the existing `nomos.cap.dependency.edges` fact is still exactly
//! what this is.
//!
//! A pure function of an already-decoded payload, grouped apart from `reading.rs` for the same
//! reason [`super::violations`] and [`super::completeness`] are: testable against hand-built
//! fixtures, no registry, no store, no reader.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_architecture::ArchitecturePayload;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Judges whether every workspace member `sources` names that depends on a package its
/// repository declares an authority is one of the doors named for it -- the authority half of
/// architecture conformance `OD-RULES-023` decided.
///
/// Spelled here rather than in [`super`] because this is the authority half of the judgment and
/// [`Violations_In`] beside it is the whole of what it reaches.
#[must_use]
pub fn Check_Write_Authority(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match super::reading::Architecture_Of(sources, facts, super::WRITE_AUTHORITY)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return super::Judged(sources, facts, super::WRITE_AUTHORITY, &|payload, source| {
        return Violations_In(&architecture, payload, source);
    });
}

/// Every edge in `payload` that reaches a declared authority from a package the declaration
/// does not name as one of its doors, as findings.
#[must_use]
pub(super) fn Violations_In(architecture: &ArchitecturePayload, payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    return payload
        .edges
        .iter()
        .filter_map(|edge| return Violation_For_Edge(architecture, source, &payload.package, edge))
        .collect();
}

/// `edge`, judged against the declared authorities, as a finding -- or `None` when the edge is
/// out of scope (a dev-dependency), its target is no declared authority, or `package` is one of
/// that authority's own named doors.
fn Violation_For_Edge(architecture: &ArchitecturePayload, source: &SourceFile, package: &str, edge: &DependencyEdge) -> Option<Finding>
{
    if edge.kind == DependencyKind::Dev
    {
        return None;
    }

    let doors = architecture.Doors_Into(&edge.target)?;
    if doors.iter().any(|door| return door == package)
    {
        return None;
    }

    return Some(Violation_Finding(source, DeclaringPackage(package), &edge.target, doors));
}

/// The crate an offending edge is declared in, as a value rather than a bare `&str`: the
/// authority it depends on is named beside it, and two bare `&str`s in adjacent positions are
/// transposable at a call site with nothing to catch it.
#[derive(Clone, Copy)]
struct DeclaringPackage<'a>(&'a str);

fn Violation_Finding(source: &SourceFile, declaring: DeclaringPackage<'_>, authority: &str, doors: &[String]) -> Finding
{
    let package = declaring.0;

    return Finding {
        rule: RuleId::New(super::WRITE_AUTHORITY),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} depends on {authority}, which this repository declares an authority \
             whose only doors are {doors:?}. A second door into an authority a repository \
             built around a single one is the exact property that declaration exists to \
             prevent."
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use super::super::test_support;
    use nomos_cap_architecture::Authority;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    fn Dependency_Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge { target: target.to_owned(), kind: DependencyKind::Normal, optional: false };
    }

    fn Dev_Edge(target: &str) -> DependencyEdge
    {
        return DependencyEdge { target: target.to_owned(), kind: DependencyKind::Dev, optional: false };
    }

    #[test]
    fn Test_A_Named_Door_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("ledger-store")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Undeclared_Door_Should_Produce_One_Finding()
    {
        let found = test_support::Sole_Violation(Violations_In, &Declaration(), "http", "ledger-store");

        assert_eq!(found.rule, RuleId::New(super::super::WRITE_AUTHORITY));
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("ledger-store"), "the finding must name the authority it refuses: {}", found.summary);
    }

    /// An authority the repository declares with no doors at all is reachable by nobody, which
    /// the encoding can express and the compiled table could not.
    #[test]
    fn Test_An_Authority_With_No_Doors_Should_Refuse_Every_Depender()
    {
        let sealed = ArchitecturePayload {
            authorities: vec![Authority { package: "sealed".to_owned(), doors: Vec::new() }],
            ..ArchitecturePayload::default()
        };
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("sealed")] };

        let findings = Violations_In(&sealed, &payload, &Source_File("billing"));

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_An_Edge_To_A_Non_Authority_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "http".to_owned(), edges: vec![Dependency_Edge("billing")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("http"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_A_Dev_Dependency_On_An_Authority_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "http".to_owned(), edges: vec![Dev_Edge("ledger-store")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("http"));

        assert!(
            findings.is_empty(),
            "a dev-dependency does not ship and must not be judged as a write door: {findings:?}"
        );
    }

    #[test]
    fn Test_A_Package_With_No_Edges_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A repository declaring no authorities has nothing here to judge, which is how this rule
    /// stops reporting against every repository that is not this one.
    #[test]
    fn Test_A_Repository_Declaring_No_Authorities_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "http".to_owned(), edges: vec![Dependency_Edge("ledger-store")] };

        let findings = Violations_In(&ArchitecturePayload::default(), &payload, &Source_File("http"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            authorities: vec![Authority { package: "ledger-store".to_owned(), doors: vec!["billing".to_owned()] }],
            ..ArchitecturePayload::default()
        };
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }
}
