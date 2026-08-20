//! Turning what [`nomos_gate_orchestration::Run`] or [`nomos_gate_orchestration::Run_Gate`]
//! answered into text and an [`ExitCode`].

use super::ExitCode;
use nomos_capability::RegistryError;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_gate_orchestration::{
    BaselineDebt, Explanation, GateExplainResult, GateOutcome, GateRunOutcome, GateRunResult, Suppression,
};
use std::io::Write;
use std::path::Path;

/// Renders what `nomos_gate_orchestration::Run` answered for `plan`.
pub(super) fn Render_Plan(outcome: &GateOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
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

/// Renders what [`super::run::Run_Gate`] answered for `run`.
///
/// Matches on [`GateRunResult::check_outcome`] directly, the same shape
/// `check::report::Render` already uses, rather than switching on `disposition`: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, which
/// only `check_outcome` carries.
pub(super) fn Render_Run(result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Run_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Run_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Run_No_Facts(&result.root, *files, stderr),
        CheckOutcome::Judged { findings, .. } => Report_Judged(findings, result, stdout),
    };
}

/// The root does not exist, is not a directory, or its walk could not be ingested -- the
/// same message whether the caller was `run` or `explain`, since neither verb's own
/// question ever got asked.
fn Render_Check_Unreadable(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "cannot judge `{}`: not a directory, or its walk could not be ingested as a \
         workspace state",
        root.display()
    );

    return ExitCode::Contradictory;
}

/// The check layer beneath this gate run has its own composition contradictory.
fn Render_Run_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
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
fn Render_Run_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
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
fn Render_Run_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
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

/// Renders a judged run's findings and reduces its disposition to an [`ExitCode`] -- the
/// one arm of [`Render_Run`] that does real work, the same way `check::report::Render`
/// delegates its own `Judged` arm to a dedicated function rather than folding it into the
/// outer match.
fn Report_Judged(findings: &[Finding], result: &GateRunResult, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ = writeln!(stdout, "{}", finding.Describe());
    }

    let _ = writeln!(
        stdout,
        "\n{} finding(s), {} of which can fail a build, {} suppressed, {} baselined",
        findings.len(),
        result.blocking_findings.len(),
        result.suppressed_findings.len(),
        result.baselined_findings.len()
    );

    return Exit_Code_For(result.disposition);
}

/// Reduces a real run's disposition to the [`ExitCode`] it reports.
fn Exit_Code_For(disposition: GateRunOutcome) -> ExitCode
{
    return match disposition
    {
        GateRunOutcome::Failed => ExitCode::Violations,
        GateRunOutcome::Passed => ExitCode::Ok,
        GateRunOutcome::Indeterminate => unreachable!(
            "nomos_gate_orchestration::Disposition never returns Indeterminate; \
             Run_Gate only assigns it for a CheckOutcome that never reached Judged, \
             and this arm is Judged's own"
        ),
    };
}

/// Renders what [`nomos_gate_orchestration::Explain_Gate`] answered for `explain`.
///
/// The same `check_outcome`-first match [`Render_Run`] uses, for the same reason: the exit
/// code for every non-`Judged` variant is a property of *why* nothing was judged, not of
/// the query.
pub(super) fn Render_Explain(result: &GateExplainResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match &result.check_outcome
    {
        CheckOutcome::Unreadable => Render_Check_Unreadable(&result.root, stderr),
        CheckOutcome::Contradictory(error) => Render_Explain_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_Explain_No_Source(&result.root, stderr),
        CheckOutcome::NoFacts { files } => Render_Explain_No_Facts(&result.root, *files, stderr),
        CheckOutcome::Judged { .. } => Report_Explanation(&result.explanation, stdout),
    };
}

/// The check layer beneath this gate explain has its own composition contradictory.
fn Render_Explain_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "the check layer beneath this gate explain has its own composition \
         contradictory, so no fact it produced would have been offered by anybody: \
         {error}"
    );

    return ExitCode::Contradictory;
}

/// The walk found no source under `root`, so `explain`'s query cannot be answered.
fn Render_Explain_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "no Rust source found under `{}`, so nothing was judged and the query \
         cannot be answered.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Source was found under `root` but no syntax fact was materialized for any of it, so
/// `explain`'s query cannot be answered.
fn Render_Explain_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
{
    let _ = writeln!(
        stderr,
        "{files} file(s) were read under `{}` and no syntax fact was materialized \
         for any of them, so the query cannot be answered.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Renders `explain`'s answer and reduces it to an [`ExitCode`] -- `Violations` when the
/// named finding would block a real run, `Ok` otherwise (not found, or found but not
/// blocking), the same "the exit code mirrors what `run` would decide for this one
/// finding" reasoning `nomos_gate_orchestration::explain`'s own doc gives.
fn Report_Explanation(explanation: &Explanation, stdout: &mut impl Write) -> ExitCode
{
    return match explanation
    {
        Explanation::NotFound =>
        {
            let _ = writeln!(stdout, "not found");

            ExitCode::Ok
        }
        Explanation::Found { finding, would_block, suppressed_by, baselined_by } =>
        {
            let tolerance = Toleration { suppressed_by: suppressed_by.as_ref(), baselined_by: baselined_by.as_ref() };

            Report_Found(finding, *would_block, tolerance, stdout)
        }
    };
}

/// The suppression or baseline note [`Report_Found`] renders alongside a found
/// explanation's block status -- never both at once, since `Explain_Gate` checks baseline
/// only once suppression is ruled out, but grouped as a pair rather than two parameters:
/// what a found explanation was tolerated by is one fact, not two.
struct Toleration<'a>
{
    suppressed_by: Option<&'a Suppression>,
    baselined_by: Option<&'a BaselineDebt>,
}

/// Renders one found explanation's finding, block status, and suppression or baseline note
/// (if either applies), and reduces it to the [`ExitCode`] a real run would decide for this
/// one finding.
fn Report_Found(finding: &Finding, would_block: bool, tolerance: Toleration<'_>, stdout: &mut impl Write) -> ExitCode
{
    let _ = writeln!(stdout, "{}", finding.Describe());
    let _ = writeln!(stdout, "would block: {would_block}");
    if let Some(suppression) = tolerance.suppressed_by
    {
        let _ = writeln!(
            stdout,
            "suppressed by: {:?} — {} (owner: {})",
            suppression.disposition, suppression.rationale, suppression.owner
        );
    }
    if let Some(debt) = tolerance.baselined_by
    {
        let _ = writeln!(stdout, "baselined by: {}", debt.rationale);
    }

    return if would_block { ExitCode::Violations } else { ExitCode::Ok };
}
