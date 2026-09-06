//! Judging an already-decoded dependency payload against this workspace's declared write
//! doors.
//!
//! `OD-RULES-023` decided the mechanism: `nomos-store`'s own README row states "one write
//! door per authority," measured true today (`nomos-workspace` is the only crate that
//! depends on it) but unchecked by anything. [`WRITE_DOORS`] names, for each authority
//! crate this workspace has declared a sole write door for, exactly which crates may depend
//! on it directly -- the identical `Permits`-over-a-declared-table shape
//! [`super::zones::Permits`] already applies to zone crossings, aimed at authority instead
//! of direction. A pure function of an already-decoded payload, grouped apart from
//! `reading.rs` for the same reason [`super::violations`] and [`super::completeness`] are:
//! testable against hand-built fixtures, no registry, no store, no reader.

use crate::SourceFile;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Every crate this workspace has declared the sole write door for, paired with the crates
/// permitted to depend on it directly.
///
/// `OD-RULES-023` measured the only real case: `nomos-store`'s own design is a single write
/// door, and exactly one crate -- `nomos-workspace` -- depends on it anywhere in this
/// workspace today. A second authority earns a second row the same way a second same-zone
/// edge earned one in [`super::zones::SAME_ZONE_EDGES`]: named here, not inferred from a
/// crate compiling.
///
/// Mirrored by `Test_Every_Write_Door_Should_Be_A_Real_Dependency`, in
/// `tests/contract/tests/boundaries/graph.rs`: every crate named here must be a real, direct
/// `Cargo.toml` dependency of the authority it is paired with, or the exception permits an
/// edge nobody's code draws.
pub const WRITE_DOORS: &[(&str, &[&str])] = &[("nomos-store", &["nomos-workspace"])];

/// Every edge in `payload` that reaches a [`WRITE_DOORS`] authority from a package not
/// named as one of its doors, as findings.
#[must_use]
pub(super) fn Violations_In(payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    return payload
        .edges
        .iter()
        .filter_map(|edge| return Violation_For_Edge(source, &payload.package, edge))
        .collect();
}

/// `edge`, judged against [`WRITE_DOORS`], as a finding -- or `None` when the edge is out
/// of scope (a dev-dependency), its target names no declared authority, or `package` is one
/// of that authority's own named doors.
fn Violation_For_Edge(source: &SourceFile, package: &str, edge: &DependencyEdge) -> Option<Finding>
{
    if edge.kind == DependencyKind::Dev
    {
        return None;
    }

    let (authority, doors) = WRITE_DOORS.iter().find(|(authority, _)| return *authority == edge.target)?;
    if doors.contains(&package)
    {
        return None;
    }

    return Some(Violation_Finding(source, package, authority, doors));
}

fn Violation_Finding(source: &SourceFile, package: &str, authority: &str, doors: &[&str]) -> Finding
{
    return Finding {
        rule: RuleId::New(super::WRITE_AUTHORITY),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} depends on {authority}, an authority crate whose only declared \
             write doors are {doors:?}. A second door into an authority this workspace \
             built around a single one is the exact property `nomos-store`'s own design \
             exists to prevent."
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
    fn Test_A_Named_Door_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-workspace".to_owned(),
            edges: vec![Dependency_Edge("nomos-store")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-workspace"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Undeclared_Door_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-cli".to_owned(),
            edges: vec![Dependency_Edge("nomos-store")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-cli"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos-cli");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_An_Edge_To_A_Non_Authority_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: vec![Dependency_Edge("nomos-cap-syntax")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-rules"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_A_Dev_Dependency_On_An_Authority_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-cli".to_owned(),
            edges: vec![Dev_Edge("nomos-store")],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-cli"));

        assert!(
            findings.is_empty(),
            "a dev-dependency does not ship and must not be judged as a write door: {findings:?}"
        );
    }

    #[test]
    fn Test_A_Package_With_No_Edges_Should_Produce_No_Finding()
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
