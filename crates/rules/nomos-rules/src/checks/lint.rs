//! One workspace member's own diagnostics from an external lint tool, relayed as
//! `Finding`s.
//!
//! `OD-RULES-010` decided a `ToolProvider`'s output is a fact a native rule judges, not a
//! `Finding` the tool emits directly. This is that rule for `nomos.cap.lint.diagnostics`:
//! since a lint tool's own diagnostic is already a verdict, this rule's own judgment is a
//! 1:1 relay rather than a second opinion — read the fact, emit one `Finding` per
//! diagnostic it carries, at `EvidenceClass::Derived`, the same `Require`-then-decode-then-
//! emit shape [`crate::Check_Dependency_Direction`] already uses, minus that rule's own
//! separate `violations` judgment step: there is no band or convention for this rule to
//! compare a diagnostic against, because `cargo clippy` already decided.
//!
//! # Composed into `nomos-check-orchestration::Run`
//!
//! Behind `OD-GATE-017`'s own per-call rule subset, the same `Wants(selected, ...)` gate
//! `DEPENDENCY_DIRECTION` already sits behind — wiring this rule in is
//! `nomos-check-orchestration`'s own territory, not this crate's.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_lint::{DiagnosticsPayload, LintDiagnostic};
use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory,
    Guarantee, IncrementalGranularity, RuleId,
};

/// This rule's own identifier.
pub const LINT_DIAGNOSTICS: &str = "lint-diagnostics";

/// The record this implementation's contract is written in -- `OD-RULES-010` decided a
/// `ToolProvider`'s output is a fact a native rule judges, this rule's first real instance.
/// `tests/contract/tests/rule_contract_citation.rs` reads `OD-RULES-010`'s own front matter
/// on every run and compares it against [`LINT_CONTRACT_RECORD_VERSION`], the same
/// `nomos_rules::DEPENDENCY_CONTRACT_RECORD` shape, so an amendment this implementation has
/// not caught up to is a red test rather than silent drift.
pub const LINT_CONTRACT_RECORD: &str = "OD-RULES-010";

/// The version of [`LINT_CONTRACT_RECORD`] this implementation was written against.
pub const LINT_CONTRACT_RECORD_VERSION: u32 = 2;

/// Judges every workspace member `sources` names against `cargo clippy`'s own reported
/// diagnostics for it.
///
/// One fact per source, the same shape [`crate::Check_Dependency_Direction`] reads — a
/// source here is a workspace member, not a file, the same asymmetry that rule's own doc
/// states for the identical reason: its `subject` is the member's own subject, the one the
/// provider keyed its fact under, and its `text` is unread.
#[must_use]
pub fn Check_Lint_Diagnostics(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return super::Relay_Findings(sources, facts, Payload_Of, Findings_Of);
}

/// One member's decoded diagnostics payload, or a finding reporting why it could not be
/// read.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<DiagnosticsPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this member's lint-diagnostics fact, turning an inadmissible answer into an
/// [`Unread`] finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Lint_Requirement();
    let capability = nomos_cap_lint::Capability();
    let inputs = InputDigest::Of(&[]);

    return match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread_Finding(
            source,
            applicability,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// What this rule needs from `nomos.cap.lint.diagnostics` before it will believe an
/// answer — the ceiling itself, the same "nothing weaker could be trusted" reasoning
/// `crate::dependency::reading::Dependency_Requirement` gives: a diagnostic this rule
/// cannot attribute to real, resolved code is not one it should relay as if `cargo
/// clippy` itself vouched for it.
#[must_use]
fn Lint_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );

    return Requirement::New(nomos_cap_lint::Capability(), nomos_cap_lint::CONTRACT_VERSION, guarantee);
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_lint::Payload_Schema()
    {
        return Err(Unread_Finding(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this member carries payload schema `{}`, which this build \
                 does not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`DiagnosticsPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<DiagnosticsPayload, Finding>
{
    return nomos_cap_lint::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread_Finding(source, Applicability::Unparseable, &refusal.to_string()));
}

/// `payload`'s own diagnostics, each relayed as a `Finding` — never judged a second time,
/// the whole point `OD-RULES-010` decided: `cargo clippy` already reached the verdict this
/// rule reports.
fn Findings_Of(source: &SourceFile, payload: &DiagnosticsPayload) -> Vec<Finding>
{
    return payload.diagnostics.iter().map(|diagnostic| return Finding_For_Diagnostic(source, diagnostic)).collect();
}

fn Finding_For_Diagnostic(source: &SourceFile, diagnostic: &LintDiagnostic) -> Finding
{
    return Finding {
        rule: RuleId::New(LINT_DIAGNOSTICS),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: Summary_Of(diagnostic),
        locations: vec![diagnostic.file.clone()],
    };
}

/// `diagnostic`, rendered as one line — the tool's own level and message, with its lint
/// identity named when it has one, since `cargo clippy` does not always attach a `code` to
/// what it reports.
fn Summary_Of(diagnostic: &LintDiagnostic) -> String
{
    return match &diagnostic.lint
    {
        Some(lint) => format!("{} [{lint}]: {} ({}:{})", diagnostic.level, diagnostic.message, diagnostic.file, diagnostic.line),
        None => format!("{}: {} ({}:{})", diagnostic.level, diagnostic.message, diagnostic.file, diagnostic.line),
    };
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(LINT_DIAGNOSTICS),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this member's lint diagnostics could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_cap_lint::LintLevel;
    use nomos_capability::ProviderOffer;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    const PROVIDER: &str = "nomos.test.lint.resolves";

    /// Also the [`super::Relay_Findings`] shape itself: a real fact read and its
    /// diagnostic relayed 1:1, never judged a second time.
    #[test]
    fn Test_Relay_Findings_Should_Read_A_Real_Fact_And_Relay_Its_Diagnostic()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Diagnostics_Fact(
            &mut store,
            &source,
            &offer,
            &DiagnosticsPayload {
                package: "nomos-cap-syntax".to_owned(),
                diagnostics: vec![LintDiagnostic {
                    level: LintLevel::Warning,
                    lint: Some("clippy::needless_return".to_owned()),
                    message: "unneeded `return` statement".to_owned(),
                    file: "crates/capabilities/nomos-cap-syntax/src/lib.rs".to_owned(),
                    line: 10,
                }],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos-cap-syntax");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Derived);
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("needless_return"), "{}", found.summary);
        assert_eq!(found.locations, vec!["crates/capabilities/nomos-cap-syntax/src/lib.rs".to_owned()]);
    }

    #[test]
    fn Test_Check_Lint_Diagnostics_Should_Produce_No_Finding_For_A_Clean_Members_Fact()
    {
        let source = Source_File("nomos-contracts");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Diagnostics_Fact(
            &mut store,
            &source,
            &offer,
            &DiagnosticsPayload { package: "nomos-contracts".to_owned(), diagnostics: Vec::new() },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        assert!(findings.is_empty(), "a clean report must not manufacture a finding: {findings:?}");
    }

    /// Also [`Test_Context`]'s own shape: the fixed context every fact and every reader
    /// this suite builds resolves under.
    #[test]
    fn Test_Test_Context_Should_Be_The_Context_A_Real_Reader_Resolves_Facts_Under()
    {
        let source = Source_File("nomos-cap-syntax");
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Multiple_Diagnostics_On_One_Member_Should_Each_Become_A_Finding()
    {
        let source = Source_File("nomos-rules");
        let TestOffering { mut store, registry, offer } = Offering();
        Materialize_Diagnostics_Fact(
            &mut store,
            &source,
            &offer,
            &DiagnosticsPayload {
                package: "nomos-rules".to_owned(),
                diagnostics: vec![
                    LintDiagnostic {
                        level: LintLevel::Warning,
                        lint: Some("clippy::needless_return".to_owned()),
                        message: "unneeded `return` statement".to_owned(),
                        file: "crates/rules/nomos-rules/src/lib.rs".to_owned(),
                        line: 1,
                    },
                    LintDiagnostic {
                        level: LintLevel::Error,
                        lint: None,
                        message: "mismatched types".to_owned(),
                        file: "crates/rules/nomos-rules/src/lint.rs".to_owned(),
                        line: 2,
                    },
                ],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        assert_eq!(findings.len(), 2, "{findings:?}");
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::Project,
        );
    }

    fn Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_lint::Capability_Contract(),
            nomos_cap_lint::Capability(),
            nomos_cap_lint::CONTRACT_VERSION,
            PROVIDER,
            Guarantee_At_Floor(),
        );
    }

    fn Materialize_Diagnostics_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DiagnosticsPayload)
    {
        let bytes = nomos_cap_lint::Encode_Payload(payload);
        test_support::Materialize(store, source.subject, offer, InputDigest::Of(&[]), nomos_cap_lint::Payload_Schema(), bytes);
    }
}
