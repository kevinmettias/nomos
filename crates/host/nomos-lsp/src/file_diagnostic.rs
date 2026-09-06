//! Translating one `nomos_contracts::Finding` into the `lsp_types::Diagnostic`(s) it
//! becomes -- the one function this crate's own module doc calls "no analysis logic of its
//! own": every field below is read off `finding` or a table `nomos-rules` /
//! `nomos-correction-orchestration` already declares, never computed by judging anything.

use crate::location::Location;
use crate::range::Range_For;
use crate::severity::Severity_Of;
use crate::walk_outward::WalkOutward;
use lsp_types::{Diagnostic, NumberOrString};
use nomos_contracts::Finding;

/// One diagnostic, addressed to the file it belongs in -- `lsp_types::Diagnostic` itself
/// carries no file, since LSP groups diagnostics by document at the transport layer
/// (`textDocument/publishDiagnostics`), not inside the diagnostic value.
#[derive(Clone, Debug, PartialEq)]
pub struct FileDiagnostic
{
    /// Repo-relative, forward slashes -- the same convention `Finding::locations` itself
    /// carries, so a caller resolves this against the same root the check walked.
    pub path: String,
    pub diagnostic: Diagnostic,
}

/// `finding`, as the diagnostics an editor renders for it -- one per location, since a
/// location is where LSP requires a diagnostic to live and `Finding::locations` allows more
/// than one. A finding with no location produces none: a text-document diagnostic cannot
/// exist without a document position, and `docs/records/OD-HOST-010-*.md` names which
/// composed rules that excludes from ever surfacing as a buffer diagnostic (`nomos.cap.
/// goals.policy`'s own workspace-level rule among them) as an honest, structural limit
/// rather than a bug this function could fix.
#[must_use]
pub fn Diagnostics_For(finding: &Finding) -> Vec<FileDiagnostic>
{
    return finding.locations.iter().map(|raw| return Diagnostic_For_Location(finding, raw)).collect();
}

/// One [`FileDiagnostic`] for one of `finding`'s own locations.
fn Diagnostic_For_Location(finding: &Finding, raw: &str) -> FileDiagnostic
{
    let location = Location::Parse(raw);
    let walked = WalkOutward::Of(finding);

    let diagnostic = Diagnostic {
        range: Range_For(location.line),
        severity: Some(Severity_Of(finding)),
        code: Some(NumberOrString::String(finding.rule.As_Str().to_owned())),
        code_description: None,
        source: Some("nomos".to_owned()),
        message: finding.summary.clone(),
        related_information: None,
        tags: None,
        data: serde_json::to_value(&walked).ok(),
    };

    return FileDiagnostic { path: location.path, diagnostic };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Diagnostics_For_Should_Produce_One_Diagnostic_Per_Location()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
            subject_name: "a.rs:3".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "trailing whitespace".to_owned(),
            locations: vec!["a.rs:3".to_owned(), "a.rs:9".to_owned()],
        };

        let diagnostics = Diagnostics_For(&finding);

        assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
        let first = diagnostics.first().expect("asserted len 2 above");
        let second = diagnostics.get(1).expect("asserted len 2 above");
        assert_eq!(first.path, "a.rs");
        assert_eq!(first.diagnostic.range.start.line, 2, "one-based line 3 is zero-based line 2");
        assert_eq!(second.diagnostic.range.start.line, 8);
    }

    #[test]
    fn Test_Diagnostics_For_Should_Report_No_Diagnostics_For_An_Unlocated_Finding()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::GOALS_AND_PARTS_LINE_UP),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([4; Digest128::BYTE_LENGTH])),
            subject_name: "OFFERINGS".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Advisory,
            summary: "a goal names no part".to_owned(),
            locations: Vec::new(),
        };

        assert!(Diagnostics_For(&finding).is_empty());
    }

    #[test]
    fn Test_Diagnostic_Should_Carry_The_Rule_As_Its_Code_And_The_Summary_As_Its_Message()
    {
        let finding = Finding {
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([5; Digest128::BYTE_LENGTH])),
            subject_name: "Table::All".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "declares no mirror".to_owned(),
            locations: vec!["crates/spec/nomos-spec-store/src/store.rs:12".to_owned()],
        };

        let diagnostics = Diagnostics_For(&finding);
        let only = diagnostics.first().expect("one location, one diagnostic");

        assert_eq!(only.path, "crates/spec/nomos-spec-store/src/store.rs");
        assert_eq!(only.diagnostic.message, "declares no mirror");
        assert_eq!(only.diagnostic.code, Some(NumberOrString::String(nomos_rules::COMPLETENESS_MIRROR.to_owned())));
        assert_eq!(only.diagnostic.severity, Some(lsp_types::DiagnosticSeverity::ERROR));

        let data = only.diagnostic.data.as_ref().expect("walk-outward data is attached");
        assert_eq!(At(data, "governing_rule", "rule"), Some(nomos_rules::COMPLETENESS_MIRROR));
        assert_eq!(At(data, "architectural_component", "crate_name"), Some("nomos-spec-store"));
        assert_eq!(At(data, "available_correction", "family"), Some(nomos_rules::COMPLETENESS_MIRROR));
    }

    /// `data.first.second`, read through `Value::get` rather than `Value`'s own `Index` --
    /// this workspace's `indexing_slicing` lint policy denies the bracket form everywhere,
    /// not only on a slice.
    fn At<'a>(data: &'a serde_json::Value, first: &str, second: &str) -> Option<&'a str>
    {
        return data.get(first)?.get(second)?.as_str();
    }
}
