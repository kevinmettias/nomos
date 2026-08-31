//! The seam between `nomos_model` and `nomos_contracts`, driven through `nomos_model`'s own
//! public API — the same view a real consumer has.
//!
//! `Coverage` and `CoverageGap` are `nomos_model`'s own types, but every gap they carry is
//! filed against a `nomos_contracts::Applicability` and every subject it names is a
//! `nomos_contracts::SubjectId`. The inline `#[cfg(test)] mod tests` in
//! `src/evidence/coverage.rs` proves the bucket logic through this crate's own private
//! access; this file proves the same properties hold through the public surface a real
//! consumer has.

use nomos_contracts::{Applicability, SubjectId};
use nomos_model::{Content_Digest, Coverage, CoverageGap};

fn Subject_Named(name: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(name.as_bytes()));
}

fn Gap_For(subject: &str, reason: Applicability) -> CoverageGap
{
    return CoverageGap {
        subject: Subject_Named(subject),
        reason,
    };
}

/// The happy path: a `nomos_contracts::Applicability::MissingCapability` gap is debt and a
/// deliberate absence is not — the property a gate consults before it may report success.
#[test]
fn Test_Debt_Should_Exclude_Deliberate_Absences_Through_The_Public_Api()
{
    let coverage = Coverage {
        evaluated: 3,
        excluded: 2,
        gaps: vec![
            Gap_For("a.rs", Applicability::NotApplicable),
            Gap_For("b.rs", Applicability::ConfigurationDisabled),
            Gap_For("c.rs", Applicability::MissingCapability),
        ],
    };

    let debt: Vec<&CoverageGap> = coverage.Debt().collect();

    assert_eq!(debt.len(), 1);
    assert_eq!(debt.first().map(|gap| gap.reason), Some(Applicability::MissingCapability));
    assert!(!coverage.Is_Complete());
}

/// `Applicability::AgentRequired` is a gap and not debt — the lifecycle boundary
/// `CHK-003`'s seventh reporting category imposes across the two crates: `nomos_contracts`
/// names the reason, and `nomos_model` decides which bucket it belongs in.
#[test]
fn Test_Agent_Required_Should_Be_A_Gap_That_Is_Not_Debt_Through_The_Public_Api()
{
    let coverage = Coverage {
        evaluated: 3,
        excluded: 1,
        gaps: vec![Gap_For("a.rs", Applicability::AgentRequired)],
    };

    assert_eq!(coverage.Debt().count(), 0, "no provider is missing");
    assert_eq!(coverage.Agent_Required().count(), 1);
    assert!(!coverage.Is_Complete(), "a subject nobody judged must not read as examined");
}

/// The three buckets `nomos_contracts::Applicability` distinguishes stay disjoint on
/// `nomos_model`'s side too: a gap must not land in both `Debt` and `Agent_Required`.
#[test]
fn Test_The_Buckets_Should_Not_Overlap_Through_The_Public_Api()
{
    let coverage = Coverage {
        evaluated: 1,
        excluded: 0,
        gaps: vec![
            Gap_For("a.rs", Applicability::MissingCapability),
            Gap_For("b.rs", Applicability::AgentRequired),
            Gap_For("c.rs", Applicability::NotApplicable),
        ],
    };

    assert_eq!(coverage.Debt().count(), 1);
    assert_eq!(coverage.Agent_Required().count(), 1);
    assert!(coverage.Debt().all(|gap| !gap.reason.Requires_Agent()));
    assert!(coverage.Agent_Required().all(|gap| !gap.reason.Is_Coverage_Debt()));
}
