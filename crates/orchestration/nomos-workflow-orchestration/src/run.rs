//! Running an ordered sequence of `WorkflowStepPlan` declarations against a real
//! `AgentExecutor`.

use nomos_platform::ProcessLauncher;

use crate::{Body, DispatchError, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

/// Dispatches `plan` in order through `launcher`.
///
/// Before dispatching a step, calls `step.declaration.Is_Coherent()`; a step that
/// declares itself incoherent is refused without its body ever dispatching —
/// `WorkflowStep::Is_Coherent`'s first real consumer anywhere in this workspace. The
/// first dispatch failure stops the run. Either way, every real outcome from the steps
/// that ran before the stop is preserved in the order they ran. An empty `plan`
/// completes vacuously.
#[must_use]
pub fn Run<P: ProcessLauncher>(plan: &[WorkflowStepPlan], launcher: &P) -> WorkflowOutcome
{
    let mut completed = Vec::new();

    for (index, step) in plan.iter().enumerate()
    {
        if !step.declaration.Is_Coherent()
        {
            return WorkflowOutcome::Refused { completed, index };
        }

        match Dispatch(&step.body, launcher)
        {
            Ok(outcome) => completed.push(outcome),
            Err(error) => return WorkflowOutcome::Failed { completed, index, error },
        }
    }

    return WorkflowOutcome::Completed { completed };
}

/// Runs `body`'s task through whichever real `AgentExecutor` it names, and reports
/// which of the two answered. The entire dispatch, not a stand-in for a shared trait —
/// the same restraint `nomos_cli::agent::Dispatch` already holds for a person's own
/// single call.
fn Dispatch<P: ProcessLauncher>(body: &Body, launcher: &P) -> Result<StepOutcome, DispatchError>
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
    };
}
