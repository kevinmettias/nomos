//! Judging an already-decoded dependency payload against this workspace's declared zones.
//!
//! A pure function of an already-decoded payload, grouped apart from `reading.rs` so it
//! stays testable against hand-built fixtures — no registry, no store, no reader — the
//! same split [`crate::naming::violations`] draws for the same reason.

use super::zones::{Permits, Zone, Zone_Of, SAME_ZONE_EDGES};
use crate::SourceFile;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Every edge `payload` declares that reaches a zone its own zone may not, or a same-zone
/// peer with no named exception, as findings.
#[must_use]
pub(super) fn Violations_In(payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    let Some(zone) = Zone_Of(&payload.package)
    else
    {
        // Undeclared entirely — a different defect from a wrong-direction edge, judged by
        // `super::completeness::Violations_In` instead. Judging direction from an unknown
        // starting zone would be a guess this rule is not entitled to make.
        return Vec::new();
    };

    return payload
        .edges
        .iter()
        .filter_map(|edge| return Violation_For_Edge(source, &payload.package, zone, edge))
        .collect();
}

/// `edge`, judged against its declaring member's own `zone`, as a finding — or `None`
/// when the edge is out of scope (a dev-dependency), its target has no declared zone to
/// compare against, or the edge is permitted.
fn Violation_For_Edge(source: &SourceFile, package: &str, zone: Zone, edge: &DependencyEdge) -> Option<Finding>
{
    if Is_Dev_Dependency(edge)
    {
        return None;
    }

    let dependency_zone = Zone_Of(&edge.target)?;
    let violation = EdgeViolation {
        package,
        zone,
        edge,
        dependency_zone,
    };

    return Violation_If_Wrong_Direction(source, &violation);
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
    zone: Zone,
    edge: &'a DependencyEdge,
    dependency_zone: Zone,
}

/// `violation` as a finding, unless its `dependency_zone` is one `violation.zone` may
/// reach — by [`Permits`] when the two zones differ, or by a named
/// [`SAME_ZONE_EDGES`] pair when they are the same zone.
fn Violation_If_Wrong_Direction(source: &SourceFile, violation: &EdgeViolation<'_>) -> Option<Finding>
{
    if violation.zone == violation.dependency_zone
    {
        if Same_Zone_Edge_Declared(violation.package, &violation.edge.target)
        {
            return None;
        }
    }
    else if Permits(violation.zone, violation.dependency_zone)
    {
        return None;
    }

    let finding = Violation_Finding(source, violation);
    return Some(finding);
}

/// Whether `(package, target)` is one of the same-zone edges this workspace names.
fn Same_Zone_Edge_Declared(package: &str, target: &str) -> bool
{
    return SAME_ZONE_EDGES
        .iter()
        .any(|(from, to)| *from == package && *to == target);
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
            "{} ({}) depends on {} ({}), an edge no zone permission or named same-zone \
             exception allows. A dependency graph nobody can reason about is what an \
             unchecked edge like this one becomes.",
            violation.package, violation.zone, violation.edge.target, violation.dependency_zone
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    fn Dependency_Edge(target: &str) -> DependencyEdge
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

    #[test]
    fn Test_A_Strictly_Downward_Edge_Should_Produce_No_Finding()
    {
        for (package, target) in Downward_Edges()
        {
            let payload = DependencyPayload {
                package: package.to_owned(),
                edges: vec![Dependency_Edge(target)],
            };

            let findings = Violations_In(&payload, &Source_File(package));

            assert!(findings.is_empty(), "{package} -> {target}: {findings:?}");
        }
    }

    /// A member and a downward edge it declares, for [`Test_A_Strictly_Downward_Edge_Should_Produce_No_Finding`] —
    /// named for the pairing rather than `Cases()`, since what varies is which real zone
    /// pair the edge crosses.
    fn Downward_Edges() -> Vec<(&'static str, &'static str)>
    {
        return vec![
            ("nomos-rules", "nomos-cap-syntax"),
            ("nomos-check-orchestration", "nomos-rules"),
            ("nomos-cli", "nomos-gate-orchestration"),
        ];
    }

    #[test]
    fn Test_Violations_In_Should_Produce_One_Finding_For_An_Upward_Edge()
    {
        let payload = DependencyPayload {
            package: "nomos-cap-syntax".to_owned(),
            edges: vec![Dependency_Edge("nomos-rules")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-cap-syntax"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos-cap-syntax");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_A_Same_Zone_Edge_With_No_Named_Exception_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-lang-rust".to_owned(),
            edges: vec![Dependency_Edge("nomos-lang-rust-scan")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-lang-rust"));

        assert_eq!(
            findings.len(),
            1,
            "two providers of one capability must not be able to name each other: {findings:?}"
        );
    }

    #[test]
    fn Test_A_Named_Same_Zone_Edge_Should_Produce_No_Finding()
    {
        // The real case OD-RULES-020 measured: nomos-gate-orchestration depends on
        // nomos-check-orchestration, both Application Service, and the edge is real and
        // named in SAME_ZONE_EDGES rather than forbidden as an unnamed peer edge would be.
        let payload = DependencyPayload {
            package: "nomos-gate-orchestration".to_owned(),
            edges: vec![Dependency_Edge("nomos-check-orchestration")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-gate-orchestration"));

        assert!(findings.is_empty(), "a named same-zone edge must not be judged a violation: {findings:?}");
    }

    #[test]
    fn Test_An_Unnamed_Reverse_Of_A_Same_Zone_Edge_Should_Still_Produce_One_Finding()
    {
        // SAME_ZONE_EDGES names nomos-gate-orchestration -> nomos-check-orchestration, not
        // the reverse; a same-zone edge is a directed fact about one real dependency, not
        // a blanket exemption for the pair.
        let payload = DependencyPayload {
            package: "nomos-check-orchestration".to_owned(),
            edges: vec![Dependency_Edge("nomos-gate-orchestration")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-check-orchestration"));

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_A_Dev_Dependency_Running_Upward_Should_Produce_No_Finding()
    {
        // The real case this guards: nomos-spec-ingest dev-depends on nomos-spec-validate
        // (both Specification) so its own test suite can run a preservation check, and
        // `Cargo.toml`'s own comment there says exactly why this must not read as a
        // violation even though the pair has no named SAME_ZONE_EDGES entry.
        let payload = DependencyPayload {
            package: "nomos-spec-ingest".to_owned(),
            edges: vec![Dev_Edge("nomos-spec-validate")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-spec-ingest"));

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
            edges: vec![Dependency_Edge("nomos-rules")],
        };

        let findings = Violations_In(&payload, &Source_File("not-in-bands"));

        assert!(
            findings.is_empty(),
            "an undeclared zone is a different defect, judged elsewhere: {findings:?}"
        );
    }

    #[test]
    fn Test_An_Edge_To_An_Undeclared_Target_Should_Produce_No_Finding()
    {
        for target in Undeclared_Targets()
        {
            let payload = DependencyPayload {
                package: "nomos-rules".to_owned(),
                edges: vec![Dependency_Edge(target)],
            };

            let findings = Violations_In(&payload, &Source_File("nomos-rules"));

            assert!(findings.is_empty(), "target {target}: {findings:?}");
        }
    }

    /// Target names no `ZONES` entry declares, for
    /// [`Test_An_Edge_To_An_Undeclared_Target_Should_Produce_No_Finding`] — an edge whose
    /// target has no declared zone is out of scope for direction, whatever it is called.
    fn Undeclared_Targets() -> Vec<&'static str>
    {
        return vec!["not-in-bands", "totally-unknown-crate", "another-missing-crate"];
    }

    #[test]
    fn Test_New_Should_Build_A_Source_File_For_A_Package_With_No_Edges()
    {
        let payload = DependencyPayload {
            package: "nomos-contracts".to_owned(),
            edges: Vec::new(),
        };

        let findings = Violations_In(&payload, &Source_File("nomos-contracts"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }
}
