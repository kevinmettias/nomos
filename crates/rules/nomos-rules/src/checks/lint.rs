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
use nomos_cap_lint::{DiagnosticsPayload, LintDiagnostic, LintLevel};
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
pub const LINT_CONTRACT_RECORD_VERSION: u32 = 3;

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
///
/// Narrowed to the provider that filed the member's fact when the source names one --
/// `OD-CAPABILITY-009`'s caller-side narrowing, because this family's offers partition by
/// subject: a Rust member's diagnostics are clippy's and a Go module's are `go vet`'s, and
/// `Registry::Resolve`, which never sees a subject, would otherwise pick one of the two for
/// both. The rule still names no provider; it narrows with the one the source carries.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = match &source.answered_by
    {
        Some(provider) => Lint_Requirement().Preferring(provider.clone()),
        None => Lint_Requirement(),
    };
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

/// One diagnostic, relayed as a `Finding` located at `file:line`.
///
/// The line is [`LintDiagnostic::line`], a plain `u32` the tool always reports, and
/// `path:line` is the spelling `concurrency_text.rs`, `formatting.rs` and `error_text.rs`
/// already file their own locations in and `nomos-lsp`'s `Location::Parse` reads, so an
/// editor gets the line this fact already carried rather than the whole file.
/// `OD-HOST-010` measured this rule dropping it.
///
/// The gate is the tool's own level, relayed: an `error` is `Blocking` and anything less is
/// `Advisory`. `OD-GATE-007` made `Cargo.toml`'s lint table the one statement of what blocks a
/// merge, and an error is exactly what that table and the compiler make fail -- a `deny` lint, or a
/// member that does not compile -- while a `warn` lint is a warning in both. Relaying every error as
/// advisory would let a workspace the gate's own lint step fails read as passing here.
fn Finding_For_Diagnostic(source: &SourceFile, diagnostic: &LintDiagnostic) -> Finding
{
    let gate = if diagnostic.level == LintLevel::Error { GateCategory::Blocking } else { GateCategory::Advisory };
    return Finding {
        address: None,
        rule: RuleId::New(LINT_DIAGNOSTICS),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate,
        summary: Summary_Of(diagnostic),
        locations: vec![format!("{}:{}", diagnostic.file, diagnostic.line)],
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
        address: None,
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
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
    use nomos_cap_lint::LintLevel;
    use nomos_capability::ProviderOffer;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    const PROVIDER: &str = "nomos.test.lint.resolves";

    /// Where in its file the fixture claims the one diagnostic sits. Nothing correlates it
    /// with a real line; it only has to be a line a rendered finding can carry.
    const FIRST_DIAGNOSTIC_LINE: u32 = 10;

    /// Where the first of [`Two_Member_Diagnostics`]' two diagnostics sits. Paired with
    /// [`SECOND_DIAGNOSTIC_LINE`], which differs, so the assertion that reads both can tell
    /// them apart by line as well as by file.
    const FIRST_MEMBER_DIAGNOSTIC_LINE: u32 = 1;

    /// The second member's diagnostic is filed lower in its own file than the first's, so
    /// the two are distinguishable if the relay ever mixed them up.
    const SECOND_DIAGNOSTIC_LINE: u32 = 2;

    /// The two diagnostics `Two_Member_Diagnostics` carries, and therefore the two findings
    /// a one-fact read of it must produce.
    const EXPECTED_DIAGNOSTIC_FINDINGS: usize = 2;

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
                    line: FIRST_DIAGNOSTIC_LINE,
                }],
            },
        );

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        Assert_One_Relayed_Diagnostic(&findings);
    }

    /// Every field a relayed lint finding must carry: the reporting member, the fact's own
    /// applicability, evidence and gate, the diagnostic's message, and the file and line it
    /// names.
    fn Assert_One_Relayed_Diagnostic(findings: &[Finding])
    {
        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos-cap-syntax");
        assert_eq!(found.applicability, Applicability::Supported);
        assert_eq!(found.evidence, EvidenceClass::Derived);
        assert_eq!(found.gate, GateCategory::Advisory);
        assert!(found.summary.contains("needless_return"), "{}", found.summary);
        assert_eq!(found.locations, vec![format!("crates/capabilities/nomos-cap-syntax/src/lib.rs:{FIRST_DIAGNOSTIC_LINE}")]);
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
        Materialize_Diagnostics_Fact(&mut store, &source, &offer, &Two_Member_Diagnostics());

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Lint_Diagnostics(&[source], &mut reader);

        assert_eq!(findings.len(), EXPECTED_DIAGNOSTIC_FINDINGS, "{findings:?}");

        // Not the order the fixture declares them in. `super::Relay_Findings` sorts every
        // relayed finding by `(subject_name, summary)` before returning, so the error --
        // whose summary begins "error:" -- precedes the warning whichever order the tool
        // reported them in. Asserting the sorted positions pins that determinism here as
        // well as the file and line each diagnostic carries.
        let located: Vec<&Vec<String>> = findings.iter().map(|finding| return &finding.locations).collect();
        assert_eq!(
            located,
            vec![
                &vec![format!("crates/rules/nomos-rules/src/lint.rs:{SECOND_DIAGNOSTIC_LINE}")],
                &vec![format!("crates/rules/nomos-rules/src/lib.rs:{FIRST_MEMBER_DIAGNOSTIC_LINE}")],
            ],
            "{findings:?}"
        );
    }

    /// One warning with a lint name and one error without one, on two different files of the
    /// same member — the fact that must become two findings rather than one.
    fn Two_Member_Diagnostics() -> DiagnosticsPayload
    {
        return DiagnosticsPayload {
            package: "nomos-rules".to_owned(),
            diagnostics: vec![
                LintDiagnostic {
                    level: LintLevel::Warning,
                    lint: Some("clippy::needless_return".to_owned()),
                    message: "unneeded `return` statement".to_owned(),
                    file: "crates/rules/nomos-rules/src/lib.rs".to_owned(),
                    line: FIRST_MEMBER_DIAGNOSTIC_LINE,
                },
                LintDiagnostic {
                    level: LintLevel::Error,
                    lint: None,
                    message: "mismatched types".to_owned(),
                    file: "crates/rules/nomos-rules/src/lint.rs".to_owned(),
                    line: SECOND_DIAGNOSTIC_LINE,
                },
            ],
        };
    }

    /// Two lint offers over disjoint members, each filing its own member's fact -- clippy's and
    /// `go vet`'s shape -- are both read when each source names the provider that answered it, and
    /// one is not when neither does: the registry, which never sees a subject, resolves one offer
    /// for both. That is the narrowing `OD-CAPABILITY-009` puts on the caller, and the control that
    /// shows it carries the read rather than decorating it.
    #[test]
    fn Test_Each_Member_Should_Be_Read_Under_The_Provider_That_Answered_It()
    {
        const SECOND: &str = "nomos.test.lint.second";
        let TestOffering { mut store, mut registry, offer } = Offering();
        let second = ProviderOffer { provider: nomos_contracts::ProviderId::New(SECOND), ..offer.clone() };
        registry.Offer(second.clone()).expect("a second provider offers against the declared contract");
        let mut rust = Source_File("rust-member");
        let mut go = Source_File("go-module");
        Materialize_Diagnostics_Fact(&mut store, &rust, &offer, &One_Diagnostic("rust-member"));
        Materialize_Diagnostics_Fact(&mut store, &go, &second, &One_Diagnostic("go-module"));

        let unnarrowed = Check_Lint_Diagnostics(&[rust.clone(), go.clone()], &mut Reader::On(&store, &registry, Test_Context()));
        rust.answered_by = Some(offer.provider.clone());
        go.answered_by = Some(second.provider.clone());
        let narrowed = Check_Lint_Diagnostics(&[rust, go], &mut Reader::On(&store, &registry, Test_Context()));

        assert_eq!(narrowed.len(), 2, "{narrowed:?}");
        assert!(narrowed.iter().all(|finding| return finding.applicability == Applicability::Supported), "{narrowed:?}");
        assert!(unnarrowed.iter().any(|finding| return finding.applicability.Is_Coverage_Debt()), "one member is unread without the narrowing: {unnarrowed:?}");
    }

    /// An `error` blocks and a `warning` does not -- the same line `Cargo.toml`'s lint table and the
    /// compiler draw for `cargo clippy` itself, so a member that does not compile, or a `deny` lint
    /// that fires, fails the gate here exactly as it fails the gate's own lint step.
    #[test]
    fn Test_An_Error_Should_Block_And_A_Warning_Should_Not()
    {
        let source = Source_File("broken-member");
        let TestOffering { mut store, registry, offer } = Offering();
        let diagnostic = |level, line| return LintDiagnostic { level, lint: None, message: "found".to_owned(), file: "src/lib.rs".to_owned(), line };
        let payload = DiagnosticsPayload { package: "broken-member".to_owned(), diagnostics: vec![diagnostic(LintLevel::Error, 1), diagnostic(LintLevel::Warning, 2)] };
        Materialize_Diagnostics_Fact(&mut store, &source, &offer, &payload);

        let findings = Check_Lint_Diagnostics(&[source], &mut Reader::On(&store, &registry, Test_Context()));

        let gates: Vec<(String, GateCategory)> = findings.iter().map(|finding| return (finding.locations.join(","), finding.gate)).collect();
        assert_eq!(gates, [("src/lib.rs:1".to_owned(), GateCategory::Blocking), ("src/lib.rs:2".to_owned(), GateCategory::Advisory)]);
    }

    fn One_Diagnostic(package: &str) -> DiagnosticsPayload
    {
        let diagnostic = LintDiagnostic { level: LintLevel::Warning, lint: Some("tool::lint".to_owned()), message: "found".to_owned(), file: format!("{package}/a"), line: FIRST_DIAGNOSTIC_LINE };
        return DiagnosticsPayload { package: package.to_owned(), diagnostics: vec![diagnostic] };
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
        return test_support::Offered_Registry(
            OfferedProvider {
                contract: nomos_cap_lint::Capability_Contract(),
                capability: nomos_cap_lint::Capability(),
                version: nomos_cap_lint::CONTRACT_VERSION,
                provider: PROVIDER,
                guarantee: Guarantee_At_Floor(),
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
    }

    fn Materialize_Diagnostics_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &DiagnosticsPayload)
    {
        let bytes = nomos_cap_lint::Encode_Payload(payload);
        test_support::Materialize_Fact(store, FactToFile { subject: source.subject, offer, semantic_inputs: InputDigest::Of(&[]), schema: nomos_cap_lint::Payload_Schema(), bytes }).expect("the fixture's store holds no fact under this key at a newer generation");
    }
}
