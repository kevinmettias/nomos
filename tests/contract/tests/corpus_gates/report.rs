use crate::{GATED_TOTAL, GATES, VARIABLES};
use std::path::PathBuf;

/// Says how much of this suite did not run, whatever the answer is.
///
/// Prints rather than fails, and the distinction is the whole design. The corpora are not
/// in this repository and most machines will never have them; a test that failed for their
/// absence would be a gate that can never be green, which is a gate everybody learns to
/// ignore. So this reports.
///
/// It reports on the way through in both directions — a configured corpus prints zero
/// skipped, so the line is present in every log and its absence is itself visible. CI runs
/// this with `--nocapture` under its own step name, which is what puts the number beside the
/// green tick rather than behind it.
#[test]
fn Test_A_Run_Should_Report_What_It_Did_Not_Check()
{
    let (skipped, lines) = Per_Variable_Report();
    let unreachable = Assertions_No_Corpus_Reaches();

    eprintln!(
        "corpus gates: {unreachable} of {GATED_TOTAL} corpus-backed assertions did not run\n{}",
        lines.join("\n")
    );

    assert!(
        skipped >= unreachable,
        "a file cannot be skipped for more variables than it reads"
    );
}

/// One line per corpus variable, and how many assertions the unset ones carry.
pub(crate) fn Per_Variable_Report() -> (usize, Vec<String>)
{
    let mut skipped = 0_usize;
    let mut lines = Vec::new();
    for variable in VARIABLES
    {
        let carried = Assertions_Carried_By(variable);
        let Some(value) = std::env::var_os(variable)
        else
        {
            skipped = skipped.saturating_add(carried);
            lines.push(format!("  {variable}: unset — up to {carried} assertion(s) skipped"));
            continue;
        };
        let at = PathBuf::from(value);

        lines.push(format!("  {variable}: {} — {carried} assertion(s) reachable", at.display()));
    }

    return (skipped, lines);
}

/// The assertions the table says the files reading this variable gate.
pub(crate) fn Assertions_Carried_By(variable: &str) -> usize
{
    return GATES
        .iter()
        .filter(|gate| return gate.variables.contains(&variable))
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
}

/// Not the sum of the per-variable figures: a file gated on two variables is counted against
/// each, and is skipped once.
pub(crate) fn Assertions_No_Corpus_Reaches() -> usize
{
    return GATES
        .iter()
        .filter(|gate| {
            return gate
                .variables
                .iter()
                .any(|variable| return std::env::var_os(variable).is_none());
        })
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
}
