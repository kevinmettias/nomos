//! Judging an already-decoded dependency payload against the repository's own declared
//! architecture.
//!
//! A pure function of an already-decoded payload, grouped apart from `reading.rs` so it
//! stays testable against hand-built fixtures — no registry, no store, no reader — the
//! same split [`crate::naming::violations`] draws for the same reason.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_architecture::{ArchitecturePayload, Depended, Depending};
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Judges every workspace member `sources` names against the architecture its own repository
/// declares.
///
/// One dependency fact per source, the same shape [`crate::Check_Naming_Convention`] reads, plus
/// one whole-workspace declaration read once for the run. A source here is a workspace member,
/// not a file — its `subject` is the member's own subject — and its `text` is unread: this
/// rule's whole judgment comes from the two facts, never from `source.text`.
///
/// Spelled here rather than in [`super`] because this is the direction half of the judgment and
/// [`Violations_In`] beside it is the whole of what it reaches: the caller belongs with the
/// thing it calls, and the three entry points sharing one file was a module boundary the
/// declarations were hiding.
#[must_use]
pub fn Check_Dependency_Direction(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match super::reading::Architecture_Of(sources, facts, super::DEPENDENCY_DIRECTION)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return super::Judged(sources, facts, super::DEPENDENCY_DIRECTION, &|payload, source| {
        return Violations_In(&architecture, payload, source);
    });
}

/// Every edge `payload` declares that reaches a component its own component may not, or a
/// peer in its own component with no named exception, as findings.
#[must_use]
pub(super) fn Violations_In(architecture: &ArchitecturePayload, payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    let Some(component) = architecture.Component_Of(&payload.package)
    else
    {
        // Unplaced entirely — a different defect from a wrong-direction edge, judged by
        // `super::completeness::Violations_In` instead. Judging direction from an unknown
        // starting component would be a guess this rule is not entitled to make.
        return Vec::new();
    };

    return payload
        .edges
        .iter()
        .filter_map(|edge| return Violation_For_Edge(architecture, source, Declaring { package: &payload.package, component }, edge))
        .collect();
}

/// The member whose own edge is being judged, and the component it was declared in — grouped
/// so [`Violation_For_Edge`] stays within this workspace's own parameter cap.
struct Declaring<'a>
{
    package: &'a str,
    component: &'a str,
}

/// `edge`, judged against its declaring member's own component, as a finding — or `None`
/// when the edge is out of scope (a dev-dependency), its target is not placed by this
/// declaration, or the edge is permitted.
fn Violation_For_Edge(architecture: &ArchitecturePayload, source: &SourceFile, declaring: Declaring<'_>, edge: &DependencyEdge) -> Option<Finding>
{
    if Is_Dev_Dependency(edge)
    {
        return None;
    }

    let dependency_component = architecture.Component_Of(&edge.target)?;
    let violation = EdgeViolation {
        package: declaring.package,
        component: declaring.component,
        edge,
        dependency_component,
    };

    return Violation_If_Wrong_Direction(architecture, source, &violation);
}

/// Whether `edge` is out of scope for this judgment.
///
/// A dev-dependency does not ship, so it is not part of the graph this judgment is about —
/// `tests/contract/src/workspace.rs`'s own `Is_Not_Dev` excludes it from `graph.rs`'s
/// identical downward-ordering check for exactly this reason, and `nomos-spec-ingest`'s own
/// `Cargo.toml` names the real case this rule would otherwise misjudge: a Specification
/// crate's dev-only dependency on its own zone's validator, present only so its own test
/// suite can exercise a preservation run.
fn Is_Dev_Dependency(edge: &DependencyEdge) -> bool
{
    return edge.kind == DependencyKind::Dev;
}

/// The specifics of one wrong-direction edge, grouped so [`Violation`] takes a type rather
/// than an unbounded parameter list.
struct EdgeViolation<'a>
{
    package: &'a str,
    component: &'a str,
    edge: &'a DependencyEdge,
    dependency_component: &'a str,
}

/// `violation` as a finding, unless its `dependency_component` is one `violation.component`
/// may reach — by the declaration's own permissions when the two components differ, or by a
/// named exception when they are the same component.
fn Violation_If_Wrong_Direction(architecture: &ArchitecturePayload, source: &SourceFile, violation: &EdgeViolation<'_>) -> Option<Finding>
{
    if violation.component == violation.dependency_component
    {
        if architecture.Excepts(Depending(violation.package), Depended(&violation.edge.target))
        {
            return None;
        }
    }
    else if architecture.Permits(Depending(violation.component), Depended(violation.dependency_component))
    {
        return None;
    }

    let finding = Violation_Finding(source, violation);
    return Some(finding);
}

fn Violation_Finding(source: &SourceFile, violation: &EdgeViolation<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(super::DEPENDENCY_DIRECTION),
        subject: source.subject,
        subject_name: violation.package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{} ({}) depends on {} ({}), an edge this repository's declared architecture \
             admits by neither a component permission nor a named exception. A dependency \
             graph nobody can reason about is what an unchecked edge like this one becomes.",
            violation.package, violation.component, violation.edge.target, violation.dependency_component
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_architecture::{Exception, Membership, Permission};
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
    fn Test_A_Permitted_Edge_Should_Produce_No_Finding()
    {
        for (package, target) in Permitted_Edges()
        {
            let payload = DependencyPayload { package: package.to_owned(), edges: vec![Dependency_Edge(target)] };

            let findings = Violations_In(&Declaration(), &payload, &Source_File(package));

            assert!(findings.is_empty(), "{package} -> {target}: {findings:?}");
        }
    }

    /// A member and an edge the declaration admits, for
    /// [`Test_A_Permitted_Edge_Should_Produce_No_Finding`] — named for the pairing rather than
    /// `Cases()`, since what varies is which declared component pair the edge crosses.
    fn Permitted_Edges() -> Vec<(&'static str, &'static str)>
    {
        return vec![("http", "billing"), ("postgres", "billing")];
    }

    #[test]
    fn Test_Violations_In_Should_Produce_One_Finding_For_An_Edge_The_Declaration_Does_Not_Admit()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("http")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "billing");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    /// The finding names the two components by the words the repository chose, which is the
    /// whole property this migration is for: nothing in this crate supplied `Domain` or `Api`.
    #[test]
    fn Test_A_Finding_Should_Name_The_Components_The_Repository_Declared()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("http")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));
        let found = findings.first().expect("the fixture declares one violation");

        assert!(found.summary.contains("billing (Domain)"), "{}", found.summary);
        assert!(found.summary.contains("http (Api)"), "{}", found.summary);
    }

    #[test]
    fn Test_Two_Peers_In_One_Component_With_No_Named_Exception_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("invoicing")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert_eq!(findings.len(), 1, "two peers of one component must not name each other: {findings:?}");
    }

    #[test]
    fn Test_A_Named_Exception_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("billing-core")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "a named exception must not be judged a violation: {findings:?}");
    }

    /// An exception is a directed fact about one real dependency, not a blanket exemption for
    /// the pair.
    #[test]
    fn Test_An_Unnamed_Reverse_Of_An_Exception_Should_Still_Produce_One_Finding()
    {
        let payload = DependencyPayload { package: "billing-core".to_owned(), edges: vec![Dependency_Edge("billing")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing-core"));

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_A_Dev_Dependency_Running_Upward_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dev_Edge("http")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(
            findings.is_empty(),
            "a dev-dependency does not ship and must not be judged as an architecture edge: {findings:?}"
        );
    }

    #[test]
    fn Test_A_Package_The_Declaration_Does_Not_Place_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "unplaced".to_owned(), edges: vec![Dependency_Edge("billing")] };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("unplaced"));

        assert!(findings.is_empty(), "an unplaced package is a different defect, judged elsewhere: {findings:?}");
    }

    #[test]
    fn Test_An_Edge_To_An_Unplaced_Target_Should_Produce_No_Finding()
    {
        for target in Unplaced_Targets()
        {
            let payload = DependencyPayload { package: "http".to_owned(), edges: vec![Dependency_Edge(target)] };

            let findings = Violations_In(&Declaration(), &payload, &Source_File("http"));

            assert!(findings.is_empty(), "target {target}: {findings:?}");
        }
    }

    /// Targets this declaration does not place — an edge whose target has no component to
    /// compare against is out of scope for direction, whatever it is called.
    fn Unplaced_Targets() -> Vec<&'static str>
    {
        return vec!["unplaced", "serde_json", "another-missing-crate"];
    }

    /// A repository declaring nothing judges nothing here, whatever its edges are. The empty
    /// declaration is the case `OD-RULES-029` measured the compiled table could not express.
    #[test]
    fn Test_A_Repository_That_Declared_Nothing_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: vec![Dependency_Edge("http")] };

        let findings = Violations_In(&ArchitecturePayload::default(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_A_Member_With_No_Edges_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A declaration in a vocabulary this workspace does not use, which is the point: every
    /// assertion above is about the mechanism, and none of them could be written this way if
    /// the components were still an enum in this crate.
    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Domain".to_owned(), "Infrastructure".to_owned(), "Api".to_owned()],
            membership: vec![
                Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "billing-core".to_owned(), component: "Domain".to_owned() },
                Membership { package: "invoicing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "postgres".to_owned(), component: "Infrastructure".to_owned() },
                Membership { package: "http".to_owned(), component: "Api".to_owned() },
            ],
            permissions: vec![
                Permission { from: "Api".to_owned(), to: "Domain".to_owned() },
                Permission { from: "Infrastructure".to_owned(), to: "Domain".to_owned() },
            ],
            exceptions: vec![Exception { from: "billing".to_owned(), to: "billing-core".to_owned() }],
            authorities: Vec::new(),
        };
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }
}
