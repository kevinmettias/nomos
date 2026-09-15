//! What a step's own outcome renders as, and the code it earns.

use super::ExitCode;
use nomos_workflow_orchestration::{StepOutcome, WorkflowOutcome};
use std::io::Write;

/// Renders `outcome` and reports the [`ExitCode`] it earns.
pub(super) fn Rendered(outcome: &WorkflowOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        WorkflowOutcome::Refused { .. } =>
        {
            let _ = writeln!(stderr, "the one step this command composed declared itself incoherent");
            ExitCode::Refused
        }
        WorkflowOutcome::Failed { error: nomos_workflow_orchestration::DispatchError::Gate(result), .. } =>
        {
            let _ = writeln!(stderr, "the gate step failed: {} blocking finding(s)", result.findings.blocking_findings.len());
            ExitCode::Refused
        }
        WorkflowOutcome::Failed { error, .. } =>
        {
            let _ = writeln!(stderr, "the step's dispatch failed: {error:?}");
            ExitCode::Unavailable
        }
        WorkflowOutcome::Completed { completed } => match completed.first()
        {
            Some(step) => Rendered_Step(step, stdout, stderr),
            None =>
            {
                let _ = writeln!(stderr, "completed with no step run, which this command never asks for");
                ExitCode::Usage
            }
        },
    };
}

/// The one step's own outcome, rendered.
fn Rendered_Step(step: &StepOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match step
    {
        StepOutcome::ClaudeCode(answer) =>
        {
            let _ = writeln!(stdout, "assumptions: {:?}", answer.result.assumptions);
            let _ = writeln!(stdout, "unresolved questions: {:?}", answer.result.unresolved_questions);
            ExitCode::Ok
        }
        StepOutcome::Ollama(answer) =>
        {
            let _ = writeln!(stdout, "{}", answer.response);
            ExitCode::Ok
        }
        StepOutcome::Check(check) => Rendered_Check(check, stdout, stderr),
        StepOutcome::Correction(correction) => Rendered_Correction(correction, stdout, stderr),
        StepOutcome::Gate(result) => Rendered_Gate(result, stdout),
    };
}

/// A `Body::Check` step's own [`nomos_check_orchestration::CheckOutcome`], rendered -- `Judged`
/// reports `Ok` regardless of its findings; see [`super`]'s own doc for why.
pub(super) fn Rendered_Check(outcome: &nomos_check_orchestration::CheckOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    use nomos_check_orchestration::CheckOutcome;

    return match outcome
    {
        CheckOutcome::Unreadable =>
        {
            let _ = writeln!(stderr, "the tree could not be read as a workspace state");
            ExitCode::Unavailable
        }
        CheckOutcome::Contradictory(error) =>
        {
            let _ = writeln!(stderr, "this build's own capability registry is self-contradictory: {error:?}");
            ExitCode::Unavailable
        }
        CheckOutcome::NoSource =>
        {
            let _ = writeln!(stderr, "no `.rs` or `.go` source found under the named root");
            ExitCode::Vacuous
        }
        CheckOutcome::NoFacts { files } =>
        {
            let _ = writeln!(stderr, "{files} file(s) were read but no syntax fact was materialized for any of them");
            ExitCode::Vacuous
        }
        CheckOutcome::Judged { findings, .. } =>
        {
            let _ = writeln!(stdout, "judged: {} finding(s)", findings.len());
            ExitCode::Ok
        }
    };
}

/// A `Body::Correction` step's own [`nomos_correction_orchestration::CorrectionOutcome`],
/// rendered -- the identical mapping `correct.rs`'s own render already uses, since this verb
/// reports the same seam's own outcome rather than a second reading of it.
///
/// One arm per disposition, one line each, so the table of what each outcome means is what a
/// reader sees rather than that table interleaved with the writing: seven of the nine write
/// exactly one line and go through [`Correction_Said`], the two dispositions that also print the
/// plan's own preview go through [`Correction_Reported`].
pub(super) fn Rendered_Correction(outcome: &nomos_correction_orchestration::CorrectionOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    use nomos_correction_orchestration::CorrectionOutcome;

    return match outcome
    {
        CorrectionOutcome::UnreadableRoot => Correction_Said(stderr, "the named root is not a directory", ExitCode::Unavailable),
        CorrectionOutcome::NoSourceFound => Correction_Said(stderr, "no `.rs` or `.go` source found under the named root", ExitCode::Vacuous),
        CorrectionOutcome::UnreadableWorkspaceState => Correction_Said(stderr, "the tree could not be read as a workspace state", ExitCode::Unavailable),
        CorrectionOutcome::ContradictoryRegistry(error) => Correction_Said(stderr, &format!("this build's own capability registry is self-contradictory: {error:?}"), ExitCode::Unavailable),
        CorrectionOutcome::NoFactsMaterialized(files) => Correction_Said(stderr, &format!("{files} file(s) were read but no syntax fact was materialized for any of them"), ExitCode::Vacuous),
        CorrectionOutcome::Clean => Correction_Said(stdout, "clean: no blocking correction claim under the named root", ExitCode::Ok),
        CorrectionOutcome::Refused(reason) => Correction_Said(stderr, reason, ExitCode::Refused),
        CorrectionOutcome::Staged { path, summary, preview } => Correction_Reported(stdout, preview, &format!("dry run: `{path}`: {summary}. Pass --commit to apply it.")),
        CorrectionOutcome::Committed { path, summary, preview, base, after_snapshot } => Correction_Reported(stdout, preview, &format!("committed: `{path}`: {summary} ({base} -> {after_snapshot})")),
    };
}

/// `line`, written to `stream`, and the code that earns -- the one-line dispositions of
/// [`Rendered_Correction`], factored out because seven of its nine arms write exactly one.
fn Correction_Said(stream: &mut impl Write, line: &str, code: ExitCode) -> ExitCode
{
    let _ = writeln!(stream, "{line}");

    return code;
}

/// The two two-line dispositions of [`Rendered_Correction`]: the plan's own preview, then the
/// line naming what happened to it.
fn Correction_Reported(stdout: &mut impl Write, preview: &[u8], line: &str) -> ExitCode
{
    let _ = writeln!(stdout, "{}", String::from_utf8_lossy(preview));
    let _ = writeln!(stdout, "{line}");

    return ExitCode::Ok;
}

/// A `Body::Gate` step's own [`nomos_gate_orchestration::GateRunResult`], rendered.
///
/// `GateRunOutcome::Failed` is not rendered here: `Dispatch` reports a failing gate as
/// `DispatchError::Gate` before this function ever sees a `StepOutcome::Gate` at all, so `Passed`
/// and `Indeterminate` are the only two dispositions a caller can reach through this path.
/// `Failed` is still matched, defensively, as `Vacuous` rather than assumed unreachable and
/// panicked on -- a total function over every value the type can hold, the same discipline every
/// other render in this module already keeps.
pub(super) fn Rendered_Gate(result: &nomos_gate_orchestration::GateRunResult, stdout: &mut impl Write) -> ExitCode
{
    use nomos_gate_orchestration::GateRunOutcome;

    return match result.disposition
    {
        GateRunOutcome::Passed =>
        {
            let _ = writeln!(stdout, "gate passed: {} finding(s), none blocking", result.findings.blocking_findings.len());
            ExitCode::Ok
        }
        GateRunOutcome::Indeterminate | GateRunOutcome::Failed =>
        {
            let _ = writeln!(stdout, "gate did not reach a judgment it could pass or fail");
            ExitCode::Vacuous
        }
    };
}
