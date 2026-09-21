//! Running an ordered sequence of `WorkflowStepPlan` declarations against a real
//! `AgentExecutor`, `ModelBackend`, `nomos-check-orchestration::Run`,
//! `nomos-correction-orchestration::Run_Correction`, or `nomos-gate-orchestration::Run_Gate`.

mod platform;

pub use platform::Platform;

use std::path::Path;

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::RunContext;
use nomos_contracts::RunId;
use nomos_correction_orchestration::{CorrectionCommand, CorrectionEnvironment, Run_Correction};
use nomos_gate_orchestration::{GateEnvironment, GateRunOutcome, Run_Gate};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_workspace::BuildVariant;

use crate::{Body, DispatchError, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

/// Dispatches `plan` in order through `platform`.
///
/// Before dispatching a step, calls `step.declaration.Is_Coherent()`; a step that
/// declares itself incoherent is refused without its body ever dispatching —
/// `WorkflowStep::Is_Coherent`'s first real consumer anywhere in this workspace. The
/// first dispatch failure stops the run -- which now includes a [`Body::Gate`] step whose
/// own disposition was `Failed`, per `P40-WORKFLOW-GATE-BODY`'s own done_when. Either way,
/// every real outcome from the steps that ran before the stop is preserved in the order
/// they ran. An empty `plan` completes vacuously.
///
/// `variant` exists for a [`Body::Check`], [`Body::Correction`] or [`Body::Gate`] step --
/// what the *compiling* binary was built as, read through `env!` there and nowhere this
/// crate could read it correctly from, so a composition root supplies it rather than this
/// crate hard-coding one. `run` exists only for a [`Body::Gate`] step: `Run_Gate`'s own
/// `RunId` identifies one execution and is not derived from anything else about the run,
/// the same "supplied by the caller" contract that type's own doc states. A plan composing
/// more than one gate step would need a fresh `RunId` per step rather than the one this
/// signature threads through unchanged -- not a case `nomos_cli::workflow`'s own
/// single-step-per-invocation shape reaches yet.
#[must_use]
pub fn Run<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(plan: &[WorkflowStepPlan], platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant, run: RunId) -> WorkflowOutcome
{
    let mut completed = Vec::new();

    for (index, step) in plan.iter().enumerate()
    {
        if !step.declaration.Is_Coherent()
        {
            return WorkflowOutcome::Refused { completed, index };
        }

        match Dispatch_Body(&step.body, platform, variant, run)
        {
            Ok(outcome) => completed.push(outcome),
            Err(error) => return WorkflowOutcome::Failed { completed, index, error },
        }
    }

    return WorkflowOutcome::Completed { completed };
}

/// The root a [`Body::ClaudeCode`] step resolves `prohibited_changes` against: none.
///
/// Every other body that needs one carries its own — `check.root`, `correction.root` — and
/// a `TaskEnvelope` has no such field, so there is nothing here to pass. Naming the absence
/// rather than passing a dot is what keeps a step that declares paths to protect from
/// silently comparing some other checkout's: the executor refuses an undecidable root
/// instead of resolving it against whatever directory this process happens to be in. Giving
/// `Body::ClaudeCode` a root of its own, the way `Body::Check` already has, is that change's
/// own work and reaches this crate's surface snapshot.
const NO_ROOT: &str = "";

/// Runs `body`'s task through whichever real dispatch target it names, and reports which
/// one answered. The entire dispatch, not a stand-in for a shared trait — the same
/// restraint `nomos_cli::agent::Dispatch` already holds for a person's own single call.
///
/// `Body::Check` and `Body::Correction` never produce a [`DispatchError`]: both seams fold
/// their own failure taxonomy (an unreadable tree, a contradictory registry, no facts, a
/// refused correction) into their own outcome type rather than a separate error type, so
/// there is nothing here for `DispatchError` to name that `StepOutcome::Check` or
/// `StepOutcome::Correction` does not already carry. `Body::Gate` is the one exception:
/// a `GateRunOutcome::Failed` disposition becomes `DispatchError::Gate` rather than
/// `StepOutcome::Gate`, which is what makes a failing gate end the workflow instead of
/// merely being reported as a step that ran.
/// Runs an agent step, and keeps a failure ending the workflow the way it always did.
///
/// The three outcomes are not one. A backend that answered is a completed step. A backend
/// that was selected and then could not be started is a dispatch error, which is what a
/// failing agent step produced before the two per-backend bodies were collapsed into one --
/// collapsing them decided which backend answers, not whether a failure stops the run. And a
/// selection that reached no backend at all is a third thing, because nothing was dispatched
/// for a step to have failed at.
fn Dispatched_Agent<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    body: &crate::AgentBody, platform: &Platform<'_, Launcher, Fs, Env>,
) -> Result<StepOutcome, DispatchError>
{
    use nomos_agent_orchestration::AgentDispatchOutcome;

    let outcome = nomos_agent_orchestration::Run_Agent_Task(
        &body.task,
        &nomos_agent_orchestration::BackendSelection {
            profile: &body.profile,
            preferred: None,
            declared: platform.declared,
        },
        &nomos_agent_orchestration::AgentEnvironment { launcher: platform.launcher },
    );

    return match outcome
    {
        AgentDispatchOutcome::Unavailable(reason) => Err(DispatchError::AgentUnavailable(reason)),
        AgentDispatchOutcome::NotSelected(absence) => Err(DispatchError::AgentNotSelected(absence)),
        answered => Ok(StepOutcome::Agent(answered)),
    };
}

fn Dispatch_Body<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(body: &Body, platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant, run: RunId) -> Result<StepOutcome, DispatchError>
{
    return match body
    {
        // The step declares a profile and the declared set decides what answers it. This
        // used to be two arms, one per backend, which made the variant a step was written as
        // its own backend choice -- nothing resolved, and a step could not say "whatever
        // answers to this family" at all. A backend that could not be selected arrives as
        // `AgentDispatchOutcome::NotSelected` rather than as a dispatch error, because
        // nothing was dispatched: there is no step failure to report, only an absence.
        Body::Agent(body) => Dispatched_Agent(body, platform),
        Body::Check(check) =>
        {
            let judged = Dispatched_Check(check, platform, variant);
            Ok(StepOutcome::Check(judged))
        }
        Body::Correction(correction) =>
        {
            let corrected = Dispatched_Correction(correction, platform, variant);
            Ok(StepOutcome::Correction(corrected))
        }
        Body::Gate(gate) => match Dispatched_Gate(gate, platform, variant, run)
        {
            result if result.disposition == GateRunOutcome::Failed => Err(DispatchError::Gate(result)),
            result => Ok(StepOutcome::Gate(result)),
        },
    };
}

/// A [`Body::Check`]'s own dispatch: a fresh workspace and fact store per call, the same
/// "every caller today passes a fresh one" shape `nomos_check_orchestration::Run`'s own
/// doc names as what reproduces its pre-reuse behavior exactly. A workflow step dispatches
/// once; there is no second call here for a reused store to help.
fn Dispatched_Check<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    check: &crate::CheckBody, platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant,
) -> nomos_check_orchestration::CheckOutcome
{
    return nomos_check_orchestration::Run(
        &check.sources,
        RunContext {
            variant: variant.clone(),
            root: &check.root,
            launcher: platform.launcher,
            filesystem: platform.filesystem,
            environment: platform.environment,
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
fn Dispatched_Correction<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    correction: &crate::CorrectionBody, platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant,
) -> nomos_correction_orchestration::CorrectionOutcome
{
    let command = CorrectionCommand { root: correction.root.clone(), commit: correction.commit.Is_A_Commit() };
    let environment = CorrectionEnvironment { variant: variant.clone(), launcher: platform.launcher, filesystem: platform.filesystem, environment: platform.environment };

    return Run_Correction(Some(correction.sources.clone()), environment, &command);
}

/// A [`Body::Gate`]'s own dispatch: `gate.sources` is always `Some`, never `None`, the
/// identical "already walked" contract [`Dispatched_Correction`] holds for
/// `Body::Correction`.
fn Dispatched_Gate<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    gate: &crate::GateBody, platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant, run: RunId,
) -> nomos_gate_orchestration::GateRunResult
{
    let environment =
        GateEnvironment { variant: variant.clone(), launcher: platform.launcher, filesystem: platform.filesystem, environment: platform.environment, now: platform.now };

    return Run_Gate(Some(gate.sources.clone()), environment, &gate.command, run);
}
