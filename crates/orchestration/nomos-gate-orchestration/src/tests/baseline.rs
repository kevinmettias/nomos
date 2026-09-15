//! What a declared quantity does to a scope a baseline entry tolerates: the one place a
//! `rule`/`subject` scope holds more than one occurrence.

use super::{Command_At, Ran_Over, Repository_Root, Source, SourcePath, SourceText};
use crate::{BaselineAllowance, BaselineDebt, BaselinePolicy, GateCommand, GateRunOutcome, GateRunResult, RuleSelector};
use nomos_contracts::RuleId;
use nomos_rules::NO_SINGLE_LINE_FUNCTION_BODIES;

/// How many collapsed bodies an unbounded or exceeded fixture puts in one file.
///
/// Named because it is a fixture's own quantity rather than a number the code computes, and it
/// is read both as the run's input and as the population it must report back.
const SCOPED_BODIES: u32 = 5;

/// How many bodies the within-allowance fixture puts in one file, and accepts.
const TOLERATED_BODIES: u32 = 2;

/// How many occurrences the exceeded-scope fixtures' entry accepted.
const ACCEPTED_BODIES: u32 = 1;

/// What [`SCOPED_BODIES`] exceeds [`ACCEPTED_BODIES`] by, which is the arithmetic a reader acts
/// on and therefore has to be the same number twice rather than two numbers that agree today.
const EXCESS_BODIES: u32 = SCOPED_BODIES - ACCEPTED_BODIES;

/// Several collapsed function bodies in one file, so one rule reports many occurrences under
/// one `rule`/`subject` scope -- the shape a baseline quantity is about, and the one every
/// other fixture in this module deliberately does not have.
///
/// Built rather than written out so that no line of *this* file is itself a collapsed body:
/// `no-single-line-function-bodies` is a text rule and this repository judges its own sources.
fn Collapsed_Bodies(count: u32) -> nomos_rules::SourceFile
{
    let text: String = (0..count).map(|index| return format!("pub fn Thing_{index}() -> i32 {{ return {index}; }}\n")).collect();

    return Source(SourcePath("collapsed.rs"), SourceText(text.as_str()));
}

/// A baseline entry for `rule` at `collapsed.rs`, accepting `allowance`.
///
/// The declared path is spelled the way an author plausibly would and not the way
/// `Subject_Of_Path` normalizes it, so that a fixture reaching the report reads the same as a
/// real run's. `P109-D`: the entry's reach is decided by the subject either way, and this
/// spelling is only what a report names it by.
fn Baseline_Accepting(allowance: BaselineAllowance) -> BaselineDebt
{
    return BaselineDebt {
        rule: RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES),
        subject: nomos_model::Subject_Of_Path("collapsed.rs"),
        rationale: "adopted at the baseline".to_owned(),
        allowance,
        declared_path: Some("./collapsed.rs".to_owned()),
    };
}

/// Runs [`crate::Run_Gate`] over `count` collapsed bodies under a baseline accepting `allowance`.
fn Run_Over_Collapsed_Bodies(count: u32, allowance: BaselineAllowance) -> GateRunResult
{
    let command = GateCommand {
        baseline: BaselinePolicy { debt: vec![Baseline_Accepting(allowance)] },
        rules: RuleSelector { include: vec![RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES)] },
        ..Command_At(Repository_Root())
    };

    return Ran_Over(vec![Collapsed_Bodies(count)], &command);
}

/// A scope inside its allowance is tolerated exactly as it was before the quantity existed.
///
/// The assertion that keeps every other test here honest: a bound that also blocked the debt a
/// repository legitimately adopted would satisfy `OD-GATE-030`'s letter and destroy the verb.
#[test]
fn Test_A_Scope_Within_Its_Allowance_Should_Still_Be_Tolerated()
{
    let result = Run_Over_Collapsed_Bodies(TOLERATED_BODIES, BaselineAllowance::AtMost(TOLERATED_BODIES));

    assert_eq!(result.findings.baselined_findings.len(), TOLERATED_BODIES as usize, "{:?}", result.findings.baselined_findings);
    assert!(result.findings.baseline_exceeded_findings.is_empty(), "{:?}", result.findings.baseline_exceeded_findings);
    assert_eq!(result.disposition, GateRunOutcome::Passed);
}

/// The defect, closed. One occurrence was accepted and five are present, so the run fails.
///
/// Before `OD-GATE-030` v2 this reported five baselined and exited clean, which attributed
/// four violations written after adoption to debt that existed before it.
#[test]
fn Test_A_Scope_Above_Its_Allowance_Should_Fail_The_Run()
{
    let result = Run_Over_Collapsed_Bodies(SCOPED_BODIES, BaselineAllowance::AtMost(ACCEPTED_BODIES));

    assert_eq!(result.disposition, GateRunOutcome::Failed, "{:?}", result.findings);
}

/// The clause that shapes this more than the bound does: an exceeded scope moves **whole**.
///
/// `OD-GATE-030` refuses attribution inside an exceeded population. With five present and one
/// accepted, four provably post-date adoption and which four is unknown, so leaving any one of
/// them in `baselined_findings` would pick a historical occurrence out of five candidates on no
/// evidence -- and would let the next reformatting commit pick a different one.
#[test]
fn Test_An_Exceeded_Scope_Should_Move_Whole_Rather_Than_Naming_Which_Occurrences_Are_New()
{
    let result = Run_Over_Collapsed_Bodies(SCOPED_BODIES, BaselineAllowance::AtMost(ACCEPTED_BODIES));

    assert!(
        result.findings.baselined_findings.is_empty(),
        "no occurrence in an exceeded scope may be reported as the adopted one: {:?}",
        result.findings.baselined_findings
    );
    assert_eq!(
        result.findings.baseline_exceeded_findings.len(),
        SCOPED_BODIES as usize,
        "{:?}",
        result.findings.baseline_exceeded_findings
    );
    assert!(
        result.findings.blocking_findings.is_empty(),
        "an exceeded tolerance is not the same answer as a rule nobody addressed: {:?}",
        result.findings.blocking_findings
    );
}

/// The arithmetic a reader acts on, reported per scope.
#[test]
fn Test_An_Exceeded_Scope_Should_Report_What_It_Accepted_And_What_It_Found()
{
    let result = Run_Over_Collapsed_Bodies(SCOPED_BODIES, BaselineAllowance::AtMost(ACCEPTED_BODIES));

    let population = result.findings.baseline_populations.first().expect("one baselined scope");
    assert_eq!(population.rule, RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES));
    assert_eq!(population.subject, nomos_model::Subject_Of_Path("collapsed.rs"));
    assert_eq!(population.allowed, BaselineAllowance::AtMost(ACCEPTED_BODIES));
    assert_eq!(population.observed, SCOPED_BODIES);
    assert_eq!(population.Excess(), EXCESS_BODIES);
}

/// An entry authored before the quantity existed tolerates whatever its scope holds.
///
/// `OD-GATE-030` v2 requires this: reading a count-less entry as bounded would begin blocking
/// builds over debt a repository did adopt, on a number nobody wrote. The population is still
/// reported, so what was left unbounded is visible rather than silent.
#[test]
fn Test_An_Unbounded_Entry_Should_Tolerate_Whatever_Its_Scope_Holds()
{
    let result = Run_Over_Collapsed_Bodies(SCOPED_BODIES, BaselineAllowance::Unbounded);

    assert_eq!(result.findings.baselined_findings.len(), SCOPED_BODIES as usize, "{:?}", result.findings.baselined_findings);
    assert!(result.findings.baseline_exceeded_findings.is_empty());
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    let population = result.findings.baseline_populations.first().expect("one baselined scope");
    assert_eq!(population.allowed, BaselineAllowance::Unbounded);
    assert_eq!(population.observed, SCOPED_BODIES);
    assert!(!population.Is_Exceeded());
}
