//! Translating one `nomos_contracts::Finding` into the judgement(s) it becomes -- the one
//! function this crate's own module doc calls "no analysis logic of its own": every field
//! below is read off `finding` or a table `nomos-rules` / `nomos-correction-orchestration`
//! already declares, never computed by judging anything.
//!
//! The target is `xvpe_diagnostics::SourceDiagnostic` rather than an editor's own wire type,
//! which is what lets every case below be asserted without a client anywhere near it.

use crate::location::Location;
use crate::severity::Severity_Of;
use crate::walk_outward::WalkOutward;
use nomos_cap_architecture::ArchitecturePayload;
use nomos_contracts::Finding;
use xvpe_diagnostics::SourceDiagnostic;

/// What this workspace calls itself in a diagnostic, so a reader looking at several
/// producers at once can tell which one spoke.
const PRODUCER: &str = "nomos";

/// `finding`, as the judgements an editor renders for it -- one per location, since a
/// location is where a diagnostic has to live and `Finding::locations` allows more than
/// one. A finding with no location produces none: a text-document diagnostic cannot exist
/// without a document position, and `docs/records/OD-HOST-010-*.md` names which composed
/// rules that excludes from ever surfacing as a buffer diagnostic (`nomos.cap.goals.policy`'s
/// own workspace-level rule among them) as an honest, structural limit rather than a bug
/// this function could fix.
#[must_use]
pub fn Diagnostics_For(architecture: &ArchitecturePayload, finding: &Finding) -> Vec<SourceDiagnostic>
{
    return finding
        .locations
        .iter()
        .map(|raw| return Diagnostic_For_Location(architecture, finding, raw))
        .collect();
}

/// One judgement for one of `finding`'s own locations.
fn Diagnostic_For_Location(architecture: &ArchitecturePayload, finding: &Finding, raw: &str) -> SourceDiagnostic
{
    let location = Location::Parse(raw);

    let judgement = SourceDiagnostic::At(location.path, Severity_Of(finding), finding.summary.clone())
        .Produced_By(PRODUCER, finding.rule.As_Str());

    let judgement = match location.line
    {
        Some(line) => judgement.On_Line(line),
        // Carried through unparsed rather than defaulted: `Location`'s own doc names the
        // rules whose locations are a package or a declaration rather than a file position,
        // and giving one of those line one would point an editor at a line it does not mean.
        None => judgement,
    };

    return Carrying_Walk_Outward(judgement, architecture, finding);
}

/// `judgement` with the walk-outward document `finding` carries under `architecture` attached,
/// or `judgement` unchanged when there is none to attach.
///
/// A serialization failure is not reachable over `WalkOutward`, which is a plain derived
/// `Serialize` over owned data: a finding still reaches the reader without its walk-outward
/// extras rather than being dropped for the sake of them.
fn Carrying_Walk_Outward(judgement: SourceDiagnostic, architecture: &ArchitecturePayload, finding: &Finding) -> SourceDiagnostic
{
    let extras = WalkOutward::Of(architecture, finding);
    return match serde_json::to_string(&extras)
    {
        Ok(walked) => judgement.Carrying(walked),
        Err(_) => judgement,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};
    use xvpe_diagnostics::DiagnosticSeverity;

    /// The one byte every fixture finding's subject digest is filled with. Nothing here
    /// asserts on it -- it exists so each fixture carries a real `SubjectId` rather than one
    /// that was never derived.
    const FIXTURE_DIGEST_BYTE: u8 = 7;

    /// The lines the two-location fixture reports, one-based and in the order it reports
    /// them -- the count and both line numbers every assertion in that test reads back.
    const REPORTED_LINES: &[u32] = &[3, 9];

    #[test]
    fn Test_Diagnostics_For_Should_Produce_One_Diagnostic_Per_Location()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "a.rs:3".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "trailing whitespace".to_owned(),
            locations: REPORTED_LINES.iter().map(|line| return format!("a.rs:{line}")).collect(),
        };

        let diagnostics = Diagnostics_For(&Declaring(), &finding);

        assert_eq!(diagnostics.len(), REPORTED_LINES.len(), "{diagnostics:?}");
        let first = diagnostics.first().expect("asserted one diagnostic per reported line above");
        let second = diagnostics.get(1).expect("asserted one diagnostic per reported line above");
        assert_eq!(first.path, "a.rs");
        // One-based, as every rule in this workspace reports it. Turning that into a
        // zero-based editor position is the engine's, at projection time.
        assert_eq!(first.line, REPORTED_LINES.first().copied());
        assert_eq!(second.line, REPORTED_LINES.get(1).copied());
    }

    #[test]
    fn Test_Diagnostics_For_Should_Report_No_Diagnostics_For_An_Unlocated_Finding()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::GOALS_AND_PARTS_LINE_UP),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "OFFERINGS".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Advisory,
            summary: "a goal names no part".to_owned(),
            locations: Vec::new(),
        };

        assert!(Diagnostics_For(&Declaring(), &finding).is_empty());
    }

    #[test]
    fn Test_A_Location_Naming_No_Line_Should_Carry_None_Rather_Than_A_Guess()
    {
        let only = Only_Diagnostic(Mirror_Finding { subject_name: "a-package", location: "crates/spec/nomos-spec-store" });

        assert_eq!(only.line, None, "a location with no line must not be given one");
        assert_eq!(only.path, "crates/spec/nomos-spec-store");
    }

    #[test]
    fn Test_Diagnostic_Should_Carry_The_Rule_As_Its_Code_And_The_Summary_As_Its_Message()
    {
        let only = Only_Diagnostic(Mirror_Finding {
            subject_name: "Table::All",
            location: "crates/spec/nomos-spec-store/src/store.rs:12",
        });

        assert_eq!(only.path, "crates/spec/nomos-spec-store/src/store.rs");
        assert_eq!(only.message, "declares no mirror");
        assert_eq!(only.code, nomos_rules::COMPLETENESS_MIRROR);
        assert_eq!(only.source, PRODUCER);
        assert_eq!(only.severity, DiagnosticSeverity::Error);

        let carried = only.detail.as_ref().expect("walk-outward data is attached");
        let data: serde_json::Value = serde_json::from_str(carried).expect("walk-outward data is a document");
        assert_eq!(At(&data, &["governing_rule", "rule"]), Some(nomos_rules::COMPLETENESS_MIRROR));
        assert_eq!(At(&data, &["architectural_component", "crate_name"]), Some("nomos-spec-store"));
        assert_eq!(At(&data, &["available_correction", "family"]), Some(nomos_rules::COMPLETENESS_MIRROR));
    }

    /// The two free-form facts a fixture `COMPLETENESS_MIRROR` finding carries: what the rule
    /// called its subject, and the one location it reported it at.
    struct Mirror_Finding<'a>
    {
        subject_name: &'a str,
        location: &'a str,
    }

    /// The string at the end of `path` in `data`, read through `Value::get` rather than
    /// `Value`'s own `Index` -- this workspace's `indexing_slicing` lint policy denies the
    /// bracket form everywhere, not only on a slice.
    fn At<'a>(data: &'a serde_json::Value, path: &[&str]) -> Option<&'a str>
    {
        let mut cursor = data;
        for step in path
        {
            cursor = cursor.get(step)?;
        }

        return cursor.as_str();
    }

    /// The one diagnostic the fixture finding `fixture` describes produces over [`Declaring`].
    /// Every fixture here locates itself exactly once, so anything else is the fixture having
    /// regressed rather than a state a caller should read.
    fn Only_Diagnostic(fixture: Mirror_Finding<'_>) -> SourceDiagnostic
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([FIXTURE_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: fixture.subject_name.to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "declares no mirror".to_owned(),
            locations: vec![fixture.location.to_owned()],
        };

        let diagnostics = Diagnostics_For(&Declaring(), &finding);
        return diagnostics.into_iter().next().expect("the fixture locates itself exactly once");
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
