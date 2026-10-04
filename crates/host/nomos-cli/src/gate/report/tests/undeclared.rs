//! The values a run's rules read that the repository never declared, named in `run`'s report after
//! the population block and apart from it and from the verdict -- `OD-RULES-011` version 3
//! decision 3, in the report a CI log shows.

use super::super::ExitCode;
use super::populations::{Populations_Of, Rendered_Beside};
use nomos_check_orchestration::UndeclaredValues;
use nomos_contracts::RuleId;
use nomos_rules::{UndeclaredOutcome, UndeclaredValue, NESTING_DEPTH, PARAMETER_COUNT};

/// A rule whose population was empty, so the report has a population block for the list to sit
/// apart from.
const EMPTY_RULE: &str = "go-only-rule";

/// What the run below names for [`NESTING_DEPTH`]: the limit no `nomos-limits.json` declared, and
/// the value it was judged against instead.
fn Nesting_Depth_Undeclared() -> UndeclaredValue
{
    return UndeclaredValue {
        family: "limits",
        declared_in: "nomos-limits.json",
        key: "nesting-depth-max",
        language: None,
        outcome: UndeclaredOutcome::JudgedAgainst { value: "3".to_owned() },
    };
}

/// [`NESTING_DEPTH`] read a value nobody declared, and [`PARAMETER_COUNT`] read one the repository
/// declared, which is why it names nothing -- exactly what the run records for each.
fn One_Undeclared_And_One_Declared() -> UndeclaredValues
{
    let mut undeclared = UndeclaredValues::New();
    undeclared.Note(RuleId::New(NESTING_DEPTH), vec![Nesting_Depth_Undeclared()]);
    undeclared.Note(RuleId::New(PARAMETER_COUNT), Vec::new());

    return undeclared;
}

/// Of two rules, the one that read a value nobody declared is named last, after the population
/// block, with where the value would be declared and what the rule judged against; the one that
/// read a declared value is not named at all. The report up to the list -- the findings, the counts
/// and the population block among it -- and the exit code are what they are with nothing named.
#[test]
fn Test_Render_Run_Should_Name_Only_The_Undeclared_Value_Beside_An_Unchanged_Verdict()
{
    let (reported, reported_code) = Rendered_Beside(Populations_Of(&[(EMPTY_RULE, 0)]), One_Undeclared_And_One_Declared());
    let (unreported, unreported_code) = Rendered_Beside(Populations_Of(&[(EMPTY_RULE, 0)]), UndeclaredValues::New());

    assert!(
        reported.ends_with(
            "\n1 rule(s) judged nothing, because no source was in their population:\n  go-only-rule\n\
             \n1 value(s) the rules read were never declared by this repository:\n  \
             nesting-depth: limits `nesting-depth-max` in nomos-limits.json, judged against 3\n"
        ),
        "{reported}"
    );
    assert!(!reported.contains(PARAMETER_COUNT), "{reported}");
    assert!(reported.starts_with(&unreported), "the report up to the list is unchanged:\n{reported}\n---\n{unreported}");
    assert_eq!(reported_code, ExitCode::Violations);
    assert_eq!(reported_code, unreported_code);
}

/// A run in which every value its rules read was declared prints nothing new: a rule that read
/// only declared values is not listed.
#[test]
fn Test_Render_Run_Should_Print_Nothing_New_When_Every_Value_Was_Declared()
{
    let mut declared = UndeclaredValues::New();
    declared.Note(RuleId::New(PARAMETER_COUNT), Vec::new());

    assert_eq!(Rendered_Beside(Populations_Of(&[]), declared), Rendered_Beside(Populations_Of(&[]), UndeclaredValues::New()));
}
