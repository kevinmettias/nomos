//! How loudly one finding should speak, translated from the two vocabularies
//! `nomos_contracts` already keeps apart: what a violation *deserves*
//! ([`nomos_contracts::GateCategory`]) and whether the rule actually reached its subject
//! ([`nomos_contracts::Applicability`]).
//!
//! No judgment happens here. [`Finding::Can_Fail_A_Build`] already decided which findings
//! are real, gated defects; this only chooses which of the engine's four severities renders
//! that decision, and the finer distinction `GateCategory` still carries for a finding that
//! cannot fail a build.
//!
//! This mapping is the half of a language server that is genuinely this workspace's. What a
//! severity *means* is a property of this workspace's own rule categories, and no shared
//! vocabulary could know it -- which is exactly why `xvpe-diagnostics` declines to.

use xvpe_diagnostics::DiagnosticSeverity;
use nomos_contracts::{Finding, GateCategory};

/// The severity an editor should render `finding` at.
///
/// A finding that can fail a build is `Error`, unconditionally -- that is exactly what
/// [`Finding::Can_Fail_A_Build`] means. Below that line, [`GateCategory`] still
/// distinguishes a finding a gate reports but discards ([`GateCategory::Advisory`],
/// `Warning`) from one no mechanical enforcer is claimed for ([`GateCategory::Review`],
/// `Information`) from one declared enforced and never invoked
/// ([`GateCategory::Unreachable`]) or one the rule never reached at all -- both `Hint`,
/// the weakest signal LSP has, because neither is a claim about the code.
#[must_use]
pub(crate) fn Severity_Of(finding: &Finding) -> DiagnosticSeverity
{
    if finding.Can_Fail_A_Build()
    {
        return DiagnosticSeverity::Error;
    }

    if !finding.applicability.Is_Evaluated()
    {
        return DiagnosticSeverity::Hint;
    }

    return match finding.gate
    {
        GateCategory::Blocking | GateCategory::Advisory => DiagnosticSeverity::Warning,
        GateCategory::Review => DiagnosticSeverity::Information,
        GateCategory::Unreachable => DiagnosticSeverity::Hint,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, EvidenceClass, RuleId, SubjectId};

    #[test]
    fn Test_Severity_Of_Should_Be_Error_When_The_Finding_Can_Fail_A_Build()
    {
        let finding = Example_Finding(Applicability::Supported, GateCategory::Blocking);

        assert_eq!(Severity_Of(&finding), DiagnosticSeverity::Error);
    }

    #[test]
    fn Test_Severity_Of_Should_Be_Warning_For_An_Evaluated_Advisory_Finding()
    {
        let finding = Example_Finding(Applicability::Supported, GateCategory::Advisory);

        assert_eq!(Severity_Of(&finding), DiagnosticSeverity::Warning);
    }

    #[test]
    fn Test_Severity_Of_Should_Be_Information_For_An_Evaluated_Review_Finding()
    {
        let finding = Example_Finding(Applicability::Supported, GateCategory::Review);

        assert_eq!(Severity_Of(&finding), DiagnosticSeverity::Information);
    }

    #[test]
    fn Test_Severity_Of_Should_Be_Hint_When_The_Rule_Never_Reached_Its_Subject()
    {
        let finding = Example_Finding(Applicability::MissingCapability, GateCategory::Blocking);

        assert_eq!(Severity_Of(&finding), DiagnosticSeverity::Hint);
    }

    #[test]
    fn Test_Severity_Of_Should_Be_Hint_For_An_Evaluated_Unreachable_Finding()
    {
        let finding = Example_Finding(Applicability::Supported, GateCategory::Unreachable);

        assert_eq!(Severity_Of(&finding), DiagnosticSeverity::Hint);
    }

    /// The one byte every fixture digest below is filled with, so a real `SubjectId` is
    /// distinguishable from one that was never derived, without the value itself pretending
    /// to mean anything.
    const FIXTURE_DIGEST_BYTE: u8 = 7;

    fn Example_Finding(applicability: Applicability, gate: GateCategory) -> Finding
    {
        return Finding {
            address: None,
            rule: RuleId::New("test-rule"),
            subject: SubjectId::From_Digest(nomos_contracts::Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; nomos_contracts::Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability,
            evidence: EvidenceClass::Derived,
            gate,
            summary: "example".to_owned(),
            locations: vec!["a.rs:1".to_owned()],
        };
    }
}
