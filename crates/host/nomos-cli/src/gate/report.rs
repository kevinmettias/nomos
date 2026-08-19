//! Turning a [`GateOutcome`] into text and an [`ExitCode`].

use super::ExitCode;
use nomos_gate_orchestration::GateOutcome;
use std::io::Write;

/// Renders what `nomos_gate_orchestration::Run` answered.
pub(super) fn Render(outcome: &GateOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
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
