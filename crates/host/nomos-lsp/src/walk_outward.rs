//! Everything a diagnostic can be walked outward to, attached at `lsp_types::Diagnostic::
//! data` -- LSP's own standard extension point for exactly this: per-diagnostic data a
//! client may read back on a code-action or hover request without this server inventing a
//! second protocol beside LSP's.
//!
//! Three of the five targets `P42-LSP-PROJECTION`'s own `done_when` names have a real,
//! mechanical answer today and are carried here: the governing rule
//! ([`governing_rule::GoverningRule`]), the architectural component
//! ([`architectural_component::ArchitecturalComponent`]), and the narrow available
//! correction ([`available_correction::AvailableCorrection`]), alongside the evidence
//! strength and applicability that were already sitting on [`Finding`] directly. Two are
//! not: which corpus requirement (`AGT-007`, `CHK-003`, ...) a finding bears on, and which
//! specific fact backed this specific judgment. `docs/records/OD-HOST-010-*.md` is the full
//! account of why, cited here rather than restated.

mod architectural_component;
mod available_correction;
mod governing_rule;

pub use architectural_component::ArchitecturalComponent;
pub use available_correction::AvailableCorrection;
pub use governing_rule::GoverningRule;

use nomos_contracts::Finding;
use serde::Serialize;

/// Everything about `finding` an editor can walk outward to, beyond the finding itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WalkOutward
{
    /// Which rule produced this finding, and what its own descriptor declares it reads --
    /// `None` only when this build's `nomos-rules` describes no such rule, which a finding
    /// from this same build's own `Run` never produces.
    pub governing_rule: Option<GoverningRule>,
    /// Which crate and zone the finding's first location falls under, when it names one
    /// under `crates/`.
    pub architectural_component: Option<ArchitecturalComponent>,
    /// The correction family this finding's rule participates in, when
    /// `nomos-correction-orchestration` composes one today.
    pub available_correction: Option<AvailableCorrection>,
    /// `Finding::evidence`'s own stable label -- whether this claim rests on a mechanism
    /// or on somebody's word.
    pub evidence: &'static str,
    /// `Finding::applicability`'s own stable label -- whether a judgment was reached at
    /// all, before asking what it was.
    pub applicability: &'static str,
    /// `Finding::Can_Fail_A_Build`, carried through directly rather than left for a client
    /// to re-derive from `evidence` and `applicability` apart.
    pub can_fail_a_build: bool,
}

impl WalkOutward
{
    /// Builds every walk-outward answer this crate has for `finding`.
    ///
    /// `finding.locations.first()` is what [`ArchitecturalComponent::Of`] reads: a finding
    /// naming more than one location (rare; `crates/rules/nomos-rules/src/checks/dependency/
    /// write_authority.rs` and its siblings are the closest today, and those name a package
    /// rather than a `crates/...` path anyway) is resolved from its first location only,
    /// the same location [`crate::file_diagnostic::Diagnostics_For`]'s own primary
    /// diagnostic is built from.
    #[must_use]
    pub(crate) fn Of(finding: &Finding) -> Self
    {
        return Self {
            governing_rule: GoverningRule::Of(&finding.rule),
            architectural_component: finding.locations.first().and_then(|location| return ArchitecturalComponent::Of(location)),
            available_correction: AvailableCorrection::Of(&finding.rule),
            evidence: finding.evidence.Label(),
            applicability: finding.applicability.Label(),
            can_fail_a_build: finding.Can_Fail_A_Build(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Of_Should_Carry_Every_Real_Answer_For_A_Correctable_Rules_Zone_Finding()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            subject_name: "Table::All".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "declares no mirror".to_owned(),
            locations: vec!["crates/spec/nomos-spec-store/src/store.rs:12".to_owned()],
        };

        let walked = WalkOutward::Of(&finding);

        assert_eq!(walked.governing_rule.expect("completeness-mirror is described").rule, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(walked.architectural_component.expect("under crates/").crate_name, "nomos-spec-store");
        assert_eq!(walked.available_correction.expect("a known correction family").family, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(walked.evidence, "Derived");
        assert_eq!(walked.applicability, "Supported");
        assert!(walked.can_fail_a_build);
    }

    #[test]
    fn Test_Of_Should_Report_No_Correction_And_No_Component_For_An_Unlocated_Package_Finding()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
            subject_name: "nomos-rules".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "depends on a Provider-zone crate".to_owned(),
            locations: vec!["nomos-rules".to_owned()],
        };

        let walked = WalkOutward::Of(&finding);

        assert!(walked.governing_rule.is_some(), "dependency-direction is a real descriptor");
        assert!(walked.architectural_component.is_none(), "a bare package name is not a crates/ path");
        assert!(walked.available_correction.is_none(), "dependency-direction has no known correction family");
    }
}
