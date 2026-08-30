//! Judging an already-decoded dependency payload against this workspace's declared bands.
//!
//! A pure function of an already-decoded payload, grouped apart from `reading.rs` so it
//! stays testable against hand-built fixtures — no registry, no store, no reader — the
//! same split [`crate::naming::violations`] draws for the same reason.

use super::bands::Declared_Band;
use crate::SourceFile;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Every edge `payload` declares that runs same-band or upward, as findings.
#[must_use]
pub(super) fn Violations_In(payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    let Some(band) = Declared_Band(&payload.package)
    else
    {
        // Undeclared entirely — a different defect from a wrong-direction edge, judged by
        // `super::completeness::Violations_In` instead. Judging direction from an unknown
        // starting band would be a guess this rule is not entitled to make.
        return Vec::new();
    };

    return payload
        .edges
        .iter()
        .filter_map(|edge| return Violation_For_Edge(source, &payload.package, band, edge))
        .collect();
}

/// `edge`, judged against its declaring member's own `band`, as a finding — or `None`
/// when the edge is out of scope (a dev-dependency) or its target has no declared band to
/// compare against.
fn Violation_For_Edge(source: &SourceFile, package: &str, band: u32, edge: &DependencyEdge) -> Option<Finding>
{
    if Is_Dev_Dependency(edge)
    {
        return None;
    }

    let dependency_band = Declared_Band(&edge.target)?;
    let violation = EdgeViolation {
        package,
        band,
        edge,
        dependency_band,
    };

    return Violation_If_Wrong_Direction(source, &violation);
}

/// Whether `edge` is out of scope for this judgment.
///
/// A dev-dependency does not ship, so it is not part of the graph this judgment is about —
/// `tests/contract/src/workspace.rs`'s own `Is_Not_Dev` excludes it from `graph.rs`'s
/// identical downward-ordering check for exactly this reason, and `nomos-spec-ingest`'s own
/// `Cargo.toml` names the real case this rule would otherwise misjudge: a band-13 crate's
/// dev-only dependency on band-14's validator, present only so its own test suite can
/// exercise a preservation run.
fn Is_Dev_Dependency(edge: &DependencyEdge) -> bool
{
    return edge.kind == DependencyKind::Dev;
}

/// The specifics of one wrong-direction edge, grouped so [`Violation`] takes a type rather
/// than an unbounded parameter list.
struct EdgeViolation<'a>
{
    package: &'a str,
    band: u32,
    edge: &'a DependencyEdge,
    dependency_band: u32,
}

/// `violation` as a finding, when its `dependency_band` really does run same-band or
/// upward from its declaring member's own `band`.
fn Violation_If_Wrong_Direction(source: &SourceFile, violation: &EdgeViolation<'_>) -> Option<Finding>
{
    if violation.dependency_band < violation.band
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
            "{} (band {}) depends on {} (band {}). Dependencies run strictly downward; \
             equal or upward edges are how a layered architecture becomes a graph nobody \
             can reason about.",
            violation.package, violation.band, violation.edge.target, violation.dependency_band
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

    /// A member and a downward edge it declares, for [`Test_A_Strictly_Downward_Edge_Should_Produce_No_Finding`] —
    /// named for the pairing rather than `Cases()`, since what varies is which real band
    /// gap the edge crosses.
    fn Downward_Edges() -> Vec<(&'static str, &'static str)>
    {
        return vec![
            ("nomos-rules", "nomos-cap-syntax"),
            ("nomos-check-orchestration", "nomos-rules"),
            ("nomos-cli", "nomos-gate-orchestration"),
        ];
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
    fn Test_A_Same_Band_Edge_Should_Produce_One_Finding()
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
            "an undeclared band is a different defect, judged elsewhere: {findings:?}"
        );
    }

    /// Target names no `BANDS` entry declares, for
    /// [`Test_An_Edge_To_An_Undeclared_Target_Should_Produce_No_Finding`] — an edge whose
    /// target has no declared band is out of scope for direction, whatever it is called.
    fn Undeclared_Targets() -> Vec<&'static str>
    {
        return vec!["not-in-bands", "totally-unknown-crate", "another-missing-crate"];
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
