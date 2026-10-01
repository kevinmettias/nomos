//! Everything a diagnostic can be walked outward to, carried on
//! `xvpe_diagnostics::SourceDiagnostic::detail` and projected by the engine onto LSP's own
//! standard per-diagnostic extension point -- data a client may read back on a code-action
//! or hover request without this server inventing a second protocol beside LSP's.
//!
//! All five targets `P42-LSP-PROJECTION`'s own `done_when` names are carried here: the
//! governing rule ([`governing_rule::GoverningRule`]), the architectural component
//! ([`architectural_component::ArchitecturalComponent`]), the narrow available correction
//! ([`available_correction::AvailableCorrection`]), the facts the finding's rule read
//! ([`supporting_facts::SupportingFacts`]), and the corpus requirements the finding bears on
//! ([`requirement_link::RequirementLink`]) -- alongside the evidence strength and applicability
//! that were already sitting on [`Finding`] directly. `OD-HOST-010` built the first three and
//! named the last two undecided; `OD-HOST-016` and `OD-HOST-015` decided them, and each is
//! read here from the mechanism that record's own building item shipped.
//!
//! **Nothing in this module judges anything, and the two newest answers are where that is
//! easiest to lose.** A requirement link is a line an assessment declared, filtered for this
//! finding's rule; a supporting fact is a trail the run carried back on its outcome. Deriving
//! either here -- a requirement from a finding's location, a fact from a key rebuilt outside
//! the rule that held it -- is the analyzer `P42-LSP-PROJECTION` named as the shape this crate
//! must not become, and `OD-HOST-015`'s third decision and `OD-HOST-016`'s sixth refused those
//! two derivations by name.
//!
//! Each answer keeps its own vocabulary for not knowing, and the two are not the same
//! vocabulary. `supporting_facts` is four words, never an absence, because three different
//! silences would otherwise read as a judgment made without evidence. `requirements` is a
//! plain list that is empty when nobody declared a line -- which is what absence means
//! everywhere in that registry, and never a claim that no rule bears.

mod architectural_component;
mod available_correction;
mod fact_read;
mod governing_rule;
mod requirement_link;
mod supporting_facts;
mod walk_context;

pub use architectural_component::ArchitecturalComponent;
pub use available_correction::AvailableCorrection;
pub use fact_read::FactRead;
pub use governing_rule::GoverningRule;
pub use requirement_link::RequirementLink;
pub use supporting_facts::SupportingFacts;
pub use walk_context::WalkContext;

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
    /// Which crate and declared component the finding's first location falls under, when it
    /// names one under `crates/`.
    pub architectural_component: Option<ArchitecturalComponent>,
    /// The correction family this finding's rule participates in, when
    /// `nomos-correction-orchestration` composes one today.
    pub available_correction: Option<AvailableCorrection>,
    /// Which facts the rule that produced this finding read in the call that produced it.
    ///
    /// Not an `Option`, deliberately. Every one of the four shapes
    /// [`SupportingFacts::answer`] can take is an answer, including the two that mean the run
    /// has no trail to show; a `None` beside them would be a fifth silence saying less than
    /// any of them. Read beside [`Self::governing_rule`]'s own `required_facts`, this is the
    /// first time a reader of a nomos diagnostic sees a rule's declared families and its
    /// observed reads in one payload.
    pub supporting_facts: SupportingFacts,
    /// The corpus requirements whose committed assessments declare this finding's rule.
    ///
    /// **Empty means nobody declared a `rule` line, never that no rule bears** -- see
    /// [`RequirementLink::Declared_For`], which is where that reading is stated and where it
    /// comes from.
    pub requirements: Vec<RequirementLink>,
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
    /// Builds every walk-outward answer this crate has for `finding`, from what `context`
    /// already holds.
    ///
    /// `finding.locations.first()` is what [`ArchitecturalComponent::Of`] reads: a finding
    /// naming more than one location (rare; `crates/rules/nomos-rules/src/checks/dependency/
    /// write_authority.rs` and its siblings are the closest today, and those name a package
    /// rather than a `crates/...` path anyway) is resolved from its first location only,
    /// the same location [`crate::file_diagnostic::Diagnostics_For`]'s own primary
    /// diagnostic is built from.
    ///
    /// The other four answers are resolved from `finding.rule` and never from a location.
    /// For the requirement link that is a decision rather than a convenience:
    /// `OD-HOST-015` refused reading a location as a requirement link, because a `site` is
    /// where a requirement is satisfied and a finding's location is where a rule fired.
    #[must_use]
    pub(crate) fn Of(context: &WalkContext<'_>, finding: &Finding) -> Self
    {
        return Self {
            governing_rule: GoverningRule::Of(&finding.rule),
            architectural_component: finding
                .locations
                .first()
                .and_then(|location| return ArchitecturalComponent::Of(context.architecture, location)),
            available_correction: AvailableCorrection::Of(&finding.rule),
            supporting_facts: SupportingFacts::Of(context.trail, &finding.rule),
            requirements: RequirementLink::Declared_For(context.assessments, &finding.rule),
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
    use nomos_cap_architecture::ArchitecturePayload;
    use nomos_cap_requirement_trace::{Assessment, Site, Verdict};
    use nomos_check_orchestration::SupportingFactTrail;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};

    /// The bytes the two fixture subject digests below are filled with. Nothing asserts on
    /// either -- they exist so each fixture carries a real `SubjectId` rather than one that was
    /// never derived -- and they differ only so the two fixtures do not share a subject.
    const FIRST_FIXTURE_DIGEST_BYTE: u8 = 1;
    const SECOND_FIXTURE_DIGEST_BYTE: u8 = 2;

    #[test]
    fn Test_Of_Should_Carry_Every_Real_Answer_For_A_Correctable_Rules_Zone_Finding()
    {
        let finding = Finding {
            address: None,
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([FIRST_FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "Table::All".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "declares no mirror".to_owned(),
            locations: vec!["crates/spec/nomos-spec-store/src/store.rs:12".to_owned()],
        };
        let declaration = Declaring();
        let trail = SupportingFactTrail::New();
        let assessments = [Assessed_Against(nomos_rules::COMPLETENESS_MIRROR)];

        let walked = WalkOutward::Of(&Context(&declaration, &trail, &assessments), &finding);

        assert_eq!(walked.governing_rule.expect("completeness-mirror is described").rule, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(walked.architectural_component.expect("under crates/").crate_name, "nomos-spec-store");
        assert_eq!(walked.available_correction.expect("a known correction family").family, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(walked.supporting_facts.answer, "Unrecorded", "a fact-reading rule this trail holds nothing for");
        assert_eq!(walked.requirements.len(), 1, "{:?}", walked.requirements);
        assert_eq!(walked.requirements.first().expect("asserted one link above").requirement, "CHK-003");
        assert_eq!(walked.evidence, "Derived");
        assert_eq!(walked.applicability, "Supported");
        assert!(walked.can_fail_a_build);
    }

    #[test]
    fn Test_Of_Should_Report_No_Correction_And_No_Component_For_An_Unlocated_Package_Finding()
    {
        let finding = Finding {
            address: None,
            rule: RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([SECOND_FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "nomos-rules".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "depends on a Provider-zone crate".to_owned(),
            locations: vec!["nomos-rules".to_owned()],
        };
        let declaration = Declaring();
        let trail = SupportingFactTrail::New();

        let walked = WalkOutward::Of(&Context(&declaration, &trail, &[]), &finding);

        assert!(walked.governing_rule.is_some(), "dependency-direction is a real descriptor");
        assert!(walked.architectural_component.is_none(), "a bare package name is not a crates/ path");
        assert!(walked.available_correction.is_none(), "dependency-direction has no known correction family");
    }

    /// The two silences a reader is most able to confuse, over one finding: a repository that
    /// declared no requirement link at all, and a rule that judges source text.
    ///
    /// They are deliberately asserted together, because the honest answers are different shapes
    /// -- an empty list on one side and a word on the other -- and a walk that answered both
    /// with the same emptiness would still produce a payload an editor could read.
    #[test]
    fn Test_Of_Should_Answer_Both_Absences_In_Their_Own_Vocabulary()
    {
        let finding = Finding {
            address: None,
            rule: RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([FIRST_FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "a.rs:1".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "trailing whitespace".to_owned(),
            locations: vec!["a.rs:1".to_owned()],
        };
        let declaration = ArchitecturePayload::default();
        let trail = SupportingFactTrail::New();
        let assessments = [Assessed_Against(nomos_rules::COMPLETENESS_MIRROR)];

        let walked = WalkOutward::Of(&Context(&declaration, &trail, &assessments), &finding);

        assert_eq!(
            walked.supporting_facts.answer, "NotFactBacked",
            "no-trailing-whitespace judges source text, so no fact was involved and none could have been"
        );
        assert!(
            walked.requirements.is_empty(),
            "no entry declares this rule, which says nobody declared a line and not that no rule bears: {:?}",
            walked.requirements
        );
    }

    /// The three inputs a walk reads, gathered the way `Diagnose` gathers them once per batch.
    fn Context<'a>(
        architecture: &'a ArchitecturePayload,
        trail: &'a SupportingFactTrail,
        assessments: &'a [Assessment],
    ) -> WalkContext<'a>
    {
        return WalkContext { architecture, trail, assessments };
    }

    /// A committed `Met` entry for `CHK-003` declaring `rule`, in the shape that registry's own
    /// reader accepts: one verdict, one site, and the rule line `OD-HOST-015` added.
    fn Assessed_Against(rule: &str) -> Assessment
    {
        return Assessment {
            requirement: "CHK-003".to_owned(),
            verdict: Verdict::Met,
            record: None,
            sites: vec![Site { path: "crates/rules/nomos-rules/src/lib.rs".to_owned(), symbol: "DESCRIPTORS".to_owned() }],
            gaps: Vec::new(),
            rules: vec![RuleId::New(rule)],
        };
    }

    /// A declaration placing the one crate these fixtures name, so a location under `crates/`
    /// resolves to something. The component is this repository's own word because the finding
    /// is this repository's own file; nothing in this module supplied it.
    fn Declaring() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Specification".to_owned()],
            membership: vec![nomos_cap_architecture::Membership {
                package: "nomos-spec-store".to_owned(),
                component: "Specification".to_owned(),
            }],
            ..ArchitecturePayload::default()
        };
    }
}
