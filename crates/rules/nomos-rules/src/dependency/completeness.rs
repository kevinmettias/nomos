//! Judging whether an already-decoded dependency payload's own package has a declared band.
//!
//! `violations.rs`'s `Violations_In` silently produces no findings for a package with no
//! entry in [`super::bands::BANDS`], naming the gap as a different defect —
//! `tests/contract`'s own `Test_Every_Member_Should_Declare_A_Band`'s subject, not this
//! rule's. This module is that subject, promoted from a contract test's hand assertion to a
//! Finding-producing judgment reachable through an ordinary `nomos check` run. A pure
//! function of an already-decoded payload, grouped apart from `reading.rs` for the same
//! reason [`super::violations`] is: testable against hand-built fixtures, no registry, no
//! store, no reader.

use crate::SourceFile;
use nomos_cap_dependency::DependencyPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// A finding when `payload`'s own package has no declared band — empty otherwise.
#[must_use]
pub(super) fn Violations_In(payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    use super::bands::Declared_Band;

    if Declared_Band(&payload.package).is_some()
    {
        return Vec::new();
    }

    return vec![Violation_For_Package(source, &payload.package)];
}

fn Violation_For_Package(source: &SourceFile, package: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(super::DEPENDENCY_COMPLETENESS),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} has no declared band. Every workspace member must declare where it \
             sits in this workspace's own layering before its dependency direction can be \
             judged against it."
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

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

    #[test]
    fn Test_A_Declared_Package_Should_Produce_No_Finding()
    {
        let payload = DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: Vec::new(),
        };

        let findings = Violations_In(&payload, &Source_File("nomos-rules"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Undeclared_Package_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload {
            package: "not-in-bands".to_owned(),
            edges: Vec::new(),
        };

        let findings = Violations_In(&payload, &Source_File("not-in-bands"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "not-in-bands");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Edges_Have_No_Effect_On_The_Coverage_Judgment()
    {
        // Coverage is about the declaring package alone; direction over its edges is
        // `violations.rs`'s own subject.
        let payload = DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: vec![nomos_cap_dependency::DependencyEdge {
                target: "not-in-bands".to_owned(),
                kind: nomos_cap_dependency::DependencyKind::Normal,
                optional: false,
            }],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-rules"));

        assert!(findings.is_empty(), "{findings:?}");
    }
}
