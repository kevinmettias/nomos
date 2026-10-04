//! The rules a run judged over an empty population, named in `run`'s report after the verdict
//! and apart from it -- `OD-ANALYSIS-012` version 3, in the report a CI log shows.

use super::super::{ExitCode, Render_Run};
use super::{Empty_Findings, Example_Finding};
use nomos_check_orchestration::{CheckOutcome, Claim, Examined, Populations, SupportingFactTrail};
use nomos_contracts::{Digest128, GateCategory, RuleId, RunId};
use nomos_gate_orchestration::{GateRunOutcome, GateRunResult};
use std::path::PathBuf;

/// A rule the populations below report as having judged nothing.
const EMPTY_RULE: &str = "go-only-rule";

/// A rule the populations below report as having judged [`JUDGED_SOURCES`] sources.
const JUDGED_RULE: &str = "rust-only-rule";

/// The byte the fixed run id is filled with. Any byte would do.
const RUN_FILL: u8 = 4;

/// How many sources [`JUDGED_RULE`] was judged over.
const JUDGED_SOURCES: usize = 3;

/// Each `(rule, size)` noted in order.
fn Populations_Of(judged: &[(&str, usize)]) -> Populations
{
    let mut populations = Populations::New();
    for (rule, size) in judged
    {
        populations.Note(RuleId::New(*rule), *size);
    }

    return populations;
}

/// What `run` prints and exits with for a run that blocked on one finding, over `populations`.
///
/// The finding, the claim and the disposition are fixed here, so two reports rendered by this
/// differ in their populations and in nothing else.
fn Rendered_Over(populations: Populations) -> (String, ExitCode)
{
    let finding = Example_Finding(GateCategory::Blocking);
    let mut whole = Empty_Findings();
    whole.blocking_findings.push(finding.clone());
    let result = GateRunResult {
        // These tests are about rendering, and say nothing about what judged the run.
        provenance: None,
        // These tests are about rendering, and say nothing about which layer stated the
        // policy this run was judged under.
        policy: None,
        no_verdict: None,
        unmatched_policy: Vec::new(),
        // Fixed rather than fresh, so two reports differ in their populations alone.
        run: RunId::From_Digest(Digest128::From_Bytes([RUN_FILL; Digest128::BYTE_LENGTH])),
        root: PathBuf::from("."),
        check_outcome: CheckOutcome::Judged {
            findings: vec![finding],
            examined: Examined { files: JUDGED_SOURCES, facts: JUDGED_SOURCES },
            claim: Claim::Complete,
            supporting_facts: SupportingFactTrail::New(),
            populations,
            undeclared: nomos_check_orchestration::UndeclaredValues::New(),
        },
        findings: whole,
        disposition: GateRunOutcome::Failed,
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Render_Run(&result, &mut stdout, &mut stderr);

    return (String::from_utf8(stdout).expect("Render_Run writes only str into the buffer"), code);
}

/// Of two rules, the one whose population was empty is named after the verdict and the one that
/// judged three sources is not; the report up to there, and the exit code, are what they are
/// with no population reported at all.
#[test]
fn Test_Render_Run_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Verdict()
{
    let (reported, reported_code) = Rendered_Over(Populations_Of(&[(EMPTY_RULE, 0), (JUDGED_RULE, JUDGED_SOURCES)]));
    let (unreported, unreported_code) = Rendered_Over(Populations::New());

    assert!(reported.ends_with("\n1 rule(s) judged nothing, because no source was in their population:\n  go-only-rule\n"), "{reported}");
    assert!(!reported.contains(JUDGED_RULE), "{reported}");
    assert!(reported.starts_with(&unreported), "the report up to the list is unchanged:\n{reported}\n---\n{unreported}");
    assert_eq!(reported_code, ExitCode::Violations);
    assert_eq!(reported_code, unreported_code);
}

/// A run in which every selected rule judged something prints nothing new.
#[test]
fn Test_Render_Run_Should_Print_Nothing_New_When_Every_Population_Was_Judged()
{
    assert_eq!(Rendered_Over(Populations_Of(&[(JUDGED_RULE, JUDGED_SOURCES)])), Rendered_Over(Populations::New()));
}
