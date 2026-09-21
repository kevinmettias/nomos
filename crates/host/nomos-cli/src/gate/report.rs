//! Turning what [`nomos_gate_orchestration::Run`] or [`nomos_gate_orchestration::Run_Gate`]
//! answered into text and an [`ExitCode`].
//!
//! The `compare` verb's rendering is here, beside the submodule declarations; what each other
//! verb renders lives in the submodule named for it, so a reader looking for `explain`'s
//! wording does not read `run`'s to find it.

use super::ExitCode;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RunId;
use nomos_gate_orchestration::{
    CollidingOccurrences, Comparability, DispositionChange, GateCompareResult, GateRunResult, JudgmentDifference,
};
use std::io::Write;

mod admits;
mod baselines;
mod explain;
mod run;
mod steps;

#[cfg(test)]
mod tests;

pub(super) use admits::Render_Admits;
pub(super) use explain::Render_Explain;
pub(super) use run::{Render_Plan, Render_Run};
pub(super) use steps::Render_Steps;

use run::{Render_Check_Unreadable, Render_Run_Contradictory, Render_Run_No_Facts, Render_Run_No_Source};

/// A collision names two findings -- the one that was indexed and the one it collided with --
/// so the number of findings a refusal accounts for is twice the number of collisions.
const FINDINGS_PER_COLLISION: usize = 2;

/// Renders what [`nomos_gate_orchestration::Compare_Gate_Runs`] answered for `compare`.
///
/// # Why a difference is never a verdict
///
/// `compare` is `0` whenever both sides were judged, however much moved between them. A
/// finding added between two trees is a fact about the difference; whether that fact should
/// fail a build is a policy question, and the policy that would answer it — which additions
/// are tolerable, against which baseline — is `GateCommand`'s own
/// `suppressions`/`baseline`/`adoption`, which no flag authors yet. Reading `Violations`
/// onto an addition here would be inventing that policy at the exit code, where nobody
/// declared it.
///
/// A side that was never judged is different in kind, and keeps the code `run` already
/// gives that reason: there is no difference to report, rather than one that happens to be
/// empty.
pub(super) fn Render_Compare(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> ExitCode
{
    if let Some(code) = Unjudged_Side(baseline, "--root", stderr)
    {
        return code;
    }
    if let Some(code) = Unjudged_Side(candidate, "--against", stderr)
    {
        return code;
    }

    let compared = match nomos_gate_orchestration::Compare_Gate_Runs(baseline, candidate)
    {
        Ok(compared) => compared,
        Err(refused) => return Refuse_Colliding_Runs(&refused, stderr),
    };

    Report_Compared_Runs(baseline, candidate, &compared, stdout);

    return ExitCode::Ok;
}

/// The exit code for a side of a `compare` that never reached a judgment, and the reason
/// why, named by the flag that selected it.
///
/// `None` when the side was judged and there is nothing to report about it. Reuses
/// [`Render_Run`]'s own refusals rather than restating them, so `compare` cannot drift into
/// describing an unreadable tree differently from `run`.
fn Unjudged_Side(result: &GateRunResult, flag: &str, stderr: &mut impl Write) -> Option<ExitCode>
{
    if matches!(result.check_outcome, CheckOutcome::Judged { .. })
    {
        return None;
    }

    let _ = writeln!(stderr, "{flag} was not judged, so there is no difference to report:");

    return Some(match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Run_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Run_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Run_No_Facts(&result.root, *files, stderr),
        // Guarded by the matches! above, which returns before reaching here.
        CheckOutcome::Judged { .. } => ExitCode::Ok,
    });
}

/// Reports a `compare` the analysis refused: two findings in one run share an occurrence
/// identity, so indexing them would silently drop one.
///
/// Not `Violations`: nothing about the tree was judged badly, and nothing about the difference
/// was judged at all. This is the analysis refusing to answer, which is the distinction
/// [`Unjudged`] above already draws for a side that could not be read.
fn Refuse_Colliding_Runs(refused: &CollidingOccurrences, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "cannot compare: run {} produced {} finding(s) that share an occurrence identity \
         with another finding in the same run, so indexing them would silently drop one.\n\
         This is a defect in the identity material rather than something to work around: \
         two findings agreeing on rule, subject, summary and locations are one occurrence \
         to every consumer. The colliding findings are:",
        refused.run,
        refused.collisions.len().saturating_mul(FINDINGS_PER_COLLISION)
    );

    for collision in &refused.collisions
    {
        let _ = writeln!(stderr, "  {}", collision.first.Describe());
        let _ = writeln!(stderr, "  {}", collision.second.Describe());
    }

    return ExitCode::Contradictory;
}

/// The two runs' identities, then what a reader may conclude from the difference below, then
/// the difference itself.
fn Report_Compared_Runs(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
    compared: &GateCompareResult,
    stdout: &mut impl Write,
)
{
    let _ = writeln!(
        stdout,
        "baseline: {} ({})\ncandidate: {} ({})",
        compared.baseline,
        baseline.root.display(),
        compared.candidate,
        candidate.root.display()
    );

    Report_Comparability(&compared.comparability, stdout);
    Report_Moved_Findings(compared, stdout);
}

/// Says what a reader may conclude from the difference below, before they read it.
///
/// Before rather than after, because a reader who takes the diff at face value has already
/// been misled by the time they reach a footnote. `OD-GATE-031` requires that a difference be
/// attributed to the repository only when the rest of the judgment was compatible or its
/// differences are stated, and a statement nobody reaches in time is not one.
///
/// Silent when the two runs were judged alike, which is the ordinary case and the one with
/// nothing to say. A caveat printed on every comparison is how a reader learns to skip the
/// line that matters.
fn Report_Comparability(comparability: &Comparability, stdout: &mut impl Write)
{
    match comparability
    {
        Comparability::Compatible => (),
        Comparability::CompatibleWith(differences) => Report_Stated_Differences(differences, stdout),
        Comparability::Incomparable(runs) => Report_Incomparable(runs, stdout),
    }
}

/// The two runs both said what judged them, and something other than the source differed.
///
/// The difference is named and the conclusion is left to the reader, which is the whole
/// point: this is not a warning that something is wrong -- comparing two policies on purpose
/// is a real thing to want -- it is the fact that makes the diff below readable.
fn Report_Stated_Differences(differences: &[JudgmentDifference], stdout: &mut impl Write)
{
    let _ = writeln!(stdout, "\nthese two runs were not judged alike:");
    for difference in differences
    {
        let _ = writeln!(stdout, "  {}", Difference_Sentence(*difference));
    }

    let _ = writeln!(
        stdout,
        "A finding that moved may have moved for one of those reasons rather than because the \
         code did."
    );
}

/// At least one side does not say what judged it.
///
/// Reported, and the diff still printed. `P106` settled the same question one verb over:
/// refusing the verdict is not refusing the answer, and throwing away what the caller asked
/// for in order to say something about it is the worse trade.
fn Report_Incomparable(runs: &[RunId], stdout: &mut impl Write)
{
    let _ = writeln!(stdout, "\nthese two runs cannot be told apart from their instruments:");
    for run in runs
    {
        let _ = writeln!(stdout, "  run {run} does not record what judged it");
    }

    let _ = writeln!(
        stdout,
        "Nothing below can be attributed to the repository rather than to a difference in \
         policy, selection or build, because there is no evidence either way -- and the \
         absence of evidence is not evidence that they matched."
    );
}

/// Every finding the two runs do not agree on, one line each -- added, then removed, then
/// changed in disposition -- and the count of each.
fn Report_Moved_Findings(compared: &GateCompareResult, stdout: &mut impl Write)
{
    for finding in &compared.added
    {
        let _ = writeln!(stdout, "+ {}", finding.Describe());
    }
    for finding in &compared.removed
    {
        let _ = writeln!(stdout, "- {}", finding.Describe());
    }
    for change in &compared.changed
    {
        Report_One_Change(change, stdout);
    }

    let _ = writeln!(
        stdout,
        "\n{} added, {} removed, {} changed disposition",
        compared.added.len(),
        compared.removed.len(),
        compared.changed.len()
    );
}

/// One changed finding, as a line: the rule, its subject, where the change sits when the
/// finding carries geometry, and the disposition it moved between.
fn Report_One_Change(change: &DispositionChange, stdout: &mut impl Write)
{
    // `locations` is appended because occurrence scope made the rule-and-subject pair
    // non-unique: one subject can now contribute several changes, and without the geometry
    // they would print as identical lines. Omitted when a finding carries none, rather than
    // printing an empty bracket that says nothing.
    let where_it_is = if change.locations.is_empty()
    {
        String::new()
    }
    else
    {
        format!(" [{}]", change.locations.join(", "))
    };

    let _ = writeln!(
        stdout,
        "~ {} {}{}: {:?} -> {:?}",
        change.rule, change.subject_name, where_it_is, change.before, change.after
    );
}

/// One judgment difference, as a reader would need it said.
///
/// Written out rather than derived from `Debug`, for the reason every other rendering in this
/// file is: a variant name is an identifier and this is a sentence to a person.
const fn Difference_Sentence(difference: JudgmentDifference) -> &'static str
{
    return match difference
    {
        JudgmentDifference::Policy => "they judged under different declared policies",
        JudgmentDifference::Selection => "they were allowed to look at different things",
        JudgmentDifference::Instrument => "they were judged by different builds, or by different rule sets",
    };
}
