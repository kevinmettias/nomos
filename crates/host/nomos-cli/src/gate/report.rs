//! Turning what [`nomos_gate_orchestration::Run`] or [`super::run::Run_Gate`] answered into
//! text and an [`ExitCode`].

use super::run::GateRunResult;
use super::ExitCode;
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use nomos_gate_orchestration::{GateOutcome, GateRunOutcome};
use std::io::Write;

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
        CheckOutcome::Unreadable =>
        {
            let _ = writeln!(
                stderr,
                "cannot judge `{}`: not a directory, or its walk could not be ingested as a \
                 workspace state",
                result.root.display()
            );

            ExitCode::Contradictory
        }
        CheckOutcome::Contradictory(error) =>
        {
            let _ = writeln!(
                stderr,
                "the check layer beneath this gate run has its own composition \
                 contradictory, so no fact it produced would have been offered by anybody: \
                 {error}"
            );

            ExitCode::Contradictory
        }
        CheckOutcome::NoSource =>
        {
            let _ = writeln!(
                stderr,
                "no Rust source found under `{}`, so nothing was judged.\n\
                 A clean result here would mean only that the walk found nothing.",
                result.root.display()
            );

            ExitCode::Vacuous
        }
        CheckOutcome::NoFacts { files } =>
        {
            let _ = writeln!(
                stderr,
                "{files} file(s) were read under `{}` and no syntax fact was materialized \
                 for any of them, so no mirror claim could be resolved.\n\
                 A clean result here would mean only that the analysis never ran.",
                result.root.display()
            );

            ExitCode::Vacuous
        }
        CheckOutcome::Judged { findings, .. } => Report_Judged(findings, result, stdout),
    };
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
        "\n{} finding(s), {} of which can fail a build",
        findings.len(),
        result.blocking_findings.len()
    );

    return match result.disposition
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
