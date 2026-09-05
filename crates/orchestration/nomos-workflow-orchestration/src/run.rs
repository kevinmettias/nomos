//! Running an ordered sequence of `WorkflowStepPlan` declarations against a real
//! `AgentExecutor`, `ModelBackend`, `nomos-check-orchestration::Run`, or
//! `nomos-correction-orchestration::Run_Correction`.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::RunContext;
use nomos_correction_orchestration::{CorrectionCommand, CorrectionEnvironment, Run_Correction};
use nomos_platform::{FileSystem, ProcessLauncher};
use nomos_workspace::BuildVariant;

use crate::{Body, DispatchError, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

/// Dispatches `plan` in order through `launcher`.
///
/// Before dispatching a step, calls `step.declaration.Is_Coherent()`; a step that
/// declares itself incoherent is refused without its body ever dispatching —
/// `WorkflowStep::Is_Coherent`'s first real consumer anywhere in this workspace. The
/// first dispatch failure stops the run. Either way, every real outcome from the steps
/// that ran before the stop is preserved in the order they ran. An empty `plan`
/// completes vacuously.
///
/// `filesystem` and `variant` exist only for a [`Body::Check`] step -- the identical two
/// pieces `nomos_check_orchestration::RunContext` needs beside a real `ProcessLauncher`
/// that neither an agent nor a model dispatch reads at all. Taken here rather than
/// constructed by this function for the same reason `nomos_check_orchestration::Run`
/// itself takes them from its own caller: `variant` is what the *compiling* binary was
/// built as, read through `env!` there and nowhere this crate could read it correctly
/// from, and `filesystem` is a platform choice a composition root makes once rather than
/// this crate hard-coding one.
#[must_use]
pub fn Run<P: ProcessLauncher, Fs: FileSystem>(plan: &[WorkflowStepPlan], launcher: &P, filesystem: &Fs, variant: &BuildVariant) -> WorkflowOutcome
{
    let mut completed = Vec::new();

    for (index, step) in plan.iter().enumerate()
    {
        if !step.declaration.Is_Coherent()
        {
            return WorkflowOutcome::Refused { completed, index };
        }

        match Dispatch(&step.body, launcher, filesystem, variant)
        {
            Ok(outcome) => completed.push(outcome),
            Err(error) => return WorkflowOutcome::Failed { completed, index, error },
        }
    }

    return WorkflowOutcome::Completed { completed };
}

/// Runs `body`'s task through whichever real dispatch target it names, and reports which
/// one answered. The entire dispatch, not a stand-in for a shared trait — the same
/// restraint `nomos_cli::agent::Dispatch` already holds for a person's own single call.
///
/// `Body::Check` and `Body::Correction` never produce a [`DispatchError`]: both seams fold
/// their own failure taxonomy (an unreadable tree, a contradictory registry, no facts, a
/// refused correction) into their own outcome type rather than a separate error type, so
/// there is nothing here for `DispatchError` to name that `StepOutcome::Check` or
/// `StepOutcome::Correction` does not already carry.
fn Dispatch<P: ProcessLauncher, Fs: FileSystem>(body: &Body, launcher: &P, filesystem: &Fs, variant: &BuildVariant) -> Result<StepOutcome, DispatchError>
{
    return match body
    {
        Body::ClaudeCode(task) => match nomos_agent_executor_claude_code::Execute_Task(task, launcher)
        {
            Ok(outcome) => Ok(StepOutcome::ClaudeCode(outcome)),
            Err(error) => Err(DispatchError::ClaudeCode(error)),
        },
        Body::Ollama(task) => match nomos_model_backend_ollama::Execute_Task(task, launcher)
        {
            Ok(outcome) => Ok(StepOutcome::Ollama(outcome)),
            Err(error) => Err(DispatchError::Ollama(error)),
        },
        Body::Check(check) => Ok(StepOutcome::Check(Dispatched_Check(check, launcher, filesystem, variant))),
        Body::Correction(correction) => Ok(StepOutcome::Correction(Dispatched_Correction(correction, launcher, filesystem, variant))),
    };
}

/// A [`Body::Check`]'s own dispatch: a fresh workspace and fact store per call, the same
/// "every caller today passes a fresh one" shape `nomos_check_orchestration::Run`'s own
/// doc names as what reproduces its pre-reuse behavior exactly. A workflow step dispatches
/// once; there is no second call here for a reused store to help.
fn Dispatched_Check<P: ProcessLauncher, Fs: FileSystem>(
    check: &crate::CheckBody, launcher: &P, filesystem: &Fs, variant: &BuildVariant,
) -> nomos_check_orchestration::CheckOutcome
{
    return nomos_check_orchestration::Run(
        &check.sources,
        RunContext {
            variant: variant.clone(),
            root: &check.root,
            launcher,
            filesystem,
            workspace: &mut None,
            store: &mut MemoryFactStore::New(),
        },
        &check.selected,
    );
}

/// A [`Body::Correction`]'s own dispatch: `correction.sources` is always `Some`, never
/// `None` -- this crate walks nothing itself, so `Run_Correction`'s own `UnreadableRoot`
/// case (its answer to a walk that never happened at all) is not reachable from a body a
/// caller already built with real, already-walked source, the same "already walked"
/// contract [`Dispatched_Check`] holds for `Body::Check`.
fn Dispatched_Correction<P: ProcessLauncher, Fs: FileSystem>(
    correction: &crate::CorrectionBody, launcher: &P, filesystem: &Fs, variant: &BuildVariant,
) -> nomos_correction_orchestration::CorrectionOutcome
{
    let command = CorrectionCommand { root: correction.root.clone(), commit: correction.commit };
    let environment = CorrectionEnvironment { variant: variant.clone(), launcher, filesystem };

    return Run_Correction(Some(correction.sources.clone()), environment, &command);
}
