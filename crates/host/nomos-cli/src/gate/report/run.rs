//! What the `plan` and `run` verbs render, and the [`ExitCode`] a judged tree reduces to.

use super::super::ExitCode;
use super::baselines::Report_Exceeded_Baselines;
use nomos_capability::RegistryError;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_gate_orchestration::{GateOutcome, GateRunOutcome, GateRunResult, NoVerdict};
use std::io::Write;
use std::path::Path;

/// Renders what `nomos_gate_orchestration::Run` answered for `plan`.
pub(in crate::gate) fn Render_Plan(outcome: &GateOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        GateOutcome::Planned(plan) =>
        {
            let _ = writeln!(stdout, "rules: {}", plan.rules.len());
            for offer in &plan.rules
            {
                let _ = writeln!(
                    stdout,
                    "  {} ({} v{})",
                    offer.rule, offer.contract_record, offer.contract_record_version
                );
            }

            ExitCode::Ok
        }
        GateOutcome::Contradictory(error) =>
        {
            let _ = writeln!(stderr, "this gate's rule registry is self-contradictory: {error:?}");

            ExitCode::Contradictory
        }
    };
}

/// Renders what [`nomos_gate_orchestration::Run_Gate`] answered for `run`.
///
/// Matches on [`GateRunResult::check_outcome`] directly, the same shape
/// `check::report::Render` already uses, rather than switching on `disposition`: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, which
/// only `check_outcome` carries.
pub(in crate::gate) fn Render_Run(result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Run_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Run_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Run_No_Facts(&result.root, *files, stderr),
        CheckOutcome::Judged { findings, .. } => Report_Judged(findings, result, stdout, stderr),
    };
}

/// Renders a judged run's findings and reduces its disposition to an [`ExitCode`] -- the
/// one arm of [`Render_Run`] that does real work, the same way `check::report::Render`
/// delegates its own `Judged` arm to a dedicated function rather than folding it into the
/// outer match.
fn Report_Judged(findings: &[Finding], result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    Report_Judged_Findings(findings, result, stdout);
    Report_Exceeded_Baselines(&result.findings.baseline_populations, stdout);
    Report_Unmatched_Policy(result, stdout);

    return Exit_Code_For(result, stderr);
}

/// One line per finding, then the counts a reader checks the report against: how many were
/// found, how many of those can fail a build, and how each of the other three were reduced.
fn Report_Judged_Findings(findings: &[Finding], result: &GateRunResult, stdout: &mut impl Write)
{
    let _ = writeln!(stdout, "run: {}", result.run);

    for finding in findings
    {
        let _ = writeln!(stdout, "{}", finding.Describe());
    }

    let _ = writeln!(
        stdout,
        "\n{} finding(s), {} of which can fail a build, {} calibrated, {} suppressed, {} baselined",
        findings.len(),
        // Both, because both failed the build. Counted together and explained apart: the
        // report below says which of them is a tolerance that ran out of room.
        result.findings.blocking_findings.len().saturating_add(result.findings.baseline_exceeded_findings.len()),
        result.findings.calibrated_findings.len(),
        result.findings.suppressed_findings.len(),
        result.findings.baselined_findings.len()
    );
}

/// Names every declared policy entry that matched no finding in this run.
///
/// `OD-GATE-024`: an entry that matches nothing is reported rather than silently ignored,
/// because an author who wrote one cannot otherwise tell a mis-spelling from a finding that
/// has since been fixed. It does not change the exit code — a policy legitimately outlives
/// the finding it was written for, and a repository whose debt was paid must not fail its
/// own gate for having paid it.
///
/// Silent when every entry matched, and when none was declared: a header over an empty list
/// on every clean run is the noise that teaches a reader to skip the line that matters.
fn Report_Unmatched_Policy(result: &GateRunResult, stdout: &mut impl Write)
{
    if result.unmatched_policy.is_empty()
    {
        return;
    }

    let _ = writeln!(stdout, "\ndeclared policy that matched nothing:");
    for entry in &result.unmatched_policy
    {
        let _ = writeln!(stdout, "  {entry}");
    }
}

/// Reduces a real run's disposition to the [`ExitCode`] it reports.
///
/// Every arm is reachable from the `Judged` arm this is called under, `Indeterminate`
/// included: `Run_Gate` assigns that disposition *after* a full judgment in two deliberate
/// cases, which [`Render_Run_No_Verdict`] names. This function asserted the opposite and
/// aborted the process on both until the cases were measured.
///
/// Takes the whole result rather than the disposition alone, because the cause and the
/// disposition are one answer: reading `Indeterminate` without the `NoVerdict` beside it is
/// exactly the half-answer this repository had before the result carried one.
fn Exit_Code_For(result: &GateRunResult, stderr: &mut impl Write) -> ExitCode
{
    return match result.disposition
    {
        GateRunOutcome::Failed => ExitCode::Violations,
        GateRunOutcome::Passed => ExitCode::Ok,
        GateRunOutcome::Indeterminate => Render_Run_No_Verdict(result.no_verdict.as_ref(), stderr),
    };
}

/// The tree was judged, the findings reported above are all of them, and no verdict was
/// reached. Says which of the three mechanisms produced that.
///
/// Each wants a different reaction, which is the whole reason `GateRunResult` carries the
/// cause rather than only the disposition. Two are a broken `nomos-gate.json` and send a
/// reader to that file with the reader's own message about it -- for a mis-spelled key,
/// the key. The third is `OD-GATE-016`'s coverage floor doing exactly what the repository
/// asked it to, where there is no fault to find and a reader sent looking for one would
/// waste the trip.
///
/// `None` is not reachable from a real `Run_Gate` today, which fills the cause on every
/// path that produces this disposition after judging. It is still answered rather than
/// asserted away: an unreachable claim about this exact arm is what aborted the process
/// before, and the honest rendering of a missing reason is to say the reason is missing.
fn Render_Run_No_Verdict(cause: Option<&NoVerdict>, stderr: &mut impl Write) -> ExitCode
{
    let _ = match cause
    {
        Some(NoVerdict::UnreadablePolicy(detail)) => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict: the `nomos-gate.json` under its \
             root could not be read, so there were no declared rules to reduce the findings \
             above by.\n  {detail}"
        ),
        Some(NoVerdict::MalformedPolicy(detail)) => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict: the `nomos-gate.json` under its \
             root is not a policy this reader accepts, so there were no declared rules to \
             reduce the findings above by.\n  {detail}"
        ),
        Some(NoVerdict::IncompleteCoverage) => writeln!(
            stderr,
            "\nthis run judged the tree, found nothing that can fail a build, and is still not \
             a pass: its declared coverage floor is `require-completeness` and some rules \
             could not look.\n\
             Nothing is wrong with the tree or with the policy. A clean result here would \
             mean only that the rules which did run found nothing."
        ),
        None => writeln!(
            stderr,
            "\nthis run judged the tree and reached no verdict, and did not record why."
        ),
    };

    return ExitCode::Contradictory;
}

/// The check layer beneath this gate run has its own composition contradictory.
pub(super) fn Render_Run_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "the check layer beneath this gate run has its own composition \
         contradictory, so no fact it produced would have been offered by anybody: \
         {error}"
    );

    return ExitCode::Contradictory;
}

/// The walk found no source under `root`, so `run` judged nothing.
pub(super) fn Render_Run_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "no Rust source found under `{}`, so nothing was judged.\n\
         A clean result here would mean only that the walk found nothing.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Source was found under `root` but no syntax fact was materialized for any of it.
pub(super) fn Render_Run_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "{files} file(s) were read under `{}` and no syntax fact was materialized \
         for any of them, so no mirror claim could be resolved.\n\
         A clean result here would mean only that the analysis never ran.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// The root does not exist, is not a directory, or its walk could not be ingested -- the
/// same message whether the caller was `run` or `explain`, since neither verb's own
/// question ever got asked.
pub(super) fn Render_Check_Unreadable(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "cannot judge `{}`: not a directory, or its walk could not be ingested as a \
         workspace state",
        root.display()
    );

    return ExitCode::Contradictory;
}
