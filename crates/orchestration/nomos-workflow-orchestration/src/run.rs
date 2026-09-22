//! Running an ordered sequence of `WorkflowStepPlan` declarations against a real
//! `AgentExecutor`, `ModelBackend`, `nomos-check-orchestration::Run`,
//! `nomos-correction-orchestration::Run_Correction`, or `nomos-gate-orchestration::Run_Gate`,
//! honoring the retry, timeout and compensation each step declares.

mod clocked_platform;
mod dispatching;
mod platform;

pub use clocked_platform::ClockedPlatform;
pub use platform::Platform;

use dispatching::Dispatching;

use std::time::Duration;

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::RunContext;
use nomos_contracts::{Compensation, RetryPolicy, RunId, Timeout};
use nomos_correction_orchestration::{CorrectionCommand, CorrectionEnvironment, CorrectionOutcome, Run_Correction};
use nomos_gate_orchestration::{GateEnvironment, GateRunOutcome, Run_Gate};
use nomos_platform::{Clock, Environment, FileSystem, ProgramLauncher};
use nomos_workspace::BuildVariant;

use crate::{Body, DispatchError, StepAttempt, StepCompensation, StepOutcome, StepTiming, WorkflowOutcome, WorkflowRun, WorkflowStepPlan};

/// Dispatches `plan` in order through `platform`.
///
/// Before dispatching a step, calls `step.declaration.Is_Coherent()`; a step that
/// declares itself incoherent is refused without its body ever dispatching —
/// `WorkflowStep::Is_Coherent`'s first real consumer anywhere in this workspace. The
/// first dispatch failure that the step's own `RetryPolicy` does not permit another
/// attempt at stops the run -- which now includes a [`Body::Gate`] step whose own
/// disposition was `Failed`, per `P40-WORKFLOW-GATE-BODY`'s own done_when. Either way,
/// every real outcome from the steps that ran before the stop is preserved in the order
/// they ran. An empty `plan` completes vacuously.
///
/// Reports [`WorkflowOutcome`] alone, which is exactly what it always reported. The
/// retry, timeout and compensation runtime `P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-
/// RUNTIME` added runs here too -- a declaration is honored whichever entry point ran it
/// -- but what honoring it *did* is the richer [`WorkflowRun`] that
/// [`Run_Unclocked`] and [`Run_With_Clock`] report and this function drops. Two callers,
/// `nomos_cli::workflow` and `nomos_api::workflow`, match this outcome's three variants
/// field by field with no wildcard, so widening it is a breaking change to two crates
/// that item may not edit; adding beside it is not.
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
    return Run_Unclocked(plan, platform, variant, run).outcome;
}

/// [`Run`], reporting every attempt and every compensation, with no clock to measure a
/// declared timeout through.
///
/// A step declaring `Timeout::Seconds` is reported as [`StepTiming::Unmeasured`] rather
/// than as having honored its bound: a run that measured nothing must not answer the
/// question as though it had measured something. A caller that can supply a clock calls
/// [`Run_With_Clock`] and gets [`StepTiming::Honored`] or [`StepTiming::Exceeded`]
/// instead.
#[must_use]
pub fn Run_Unclocked<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(plan: &[WorkflowStepPlan], platform: &Platform<'_, Launcher, Fs, Env>, variant: &BuildVariant, run: RunId) -> WorkflowRun
{
    return Ran(plan, &Dispatching { platform, variant, run, read_clock: None });
}

/// [`Run_Unclocked`], measuring each bounded step's dispatch through the clock
/// `platform` carries.
///
/// The clock is read twice per attempt, immediately before and immediately after that
/// attempt's dispatch, and only the difference is kept -- so a clock that reports fixed
/// readings makes a timeout case reproduce exactly, which is what lets these cases be
/// tested at all rather than slept through.
#[must_use]
pub fn Run_With_Clock<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment, Clk: Clock>(plan: &[WorkflowStepPlan], platform: &ClockedPlatform<'_, Launcher, Fs, Env, Clk>, variant: &BuildVariant, run: RunId) -> WorkflowRun
{
    let read_clock = || return platform.clock.Now();

    return Ran(plan, &Dispatching { platform: platform.platform, variant, run, read_clock: Some(&read_clock) });
}

/// The one runner both reporting entry points share.
fn Ran<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(plan: &[WorkflowStepPlan], context: &Dispatching<'_, Launcher, Fs, Env>) -> WorkflowRun
{
    let mut completed = Vec::new();
    let mut attempts = Vec::new();

    for (index, step) in plan.iter().enumerate()
    {
        if !step.declaration.Is_Coherent()
        {
            return WorkflowRun { outcome: WorkflowOutcome::Refused { completed, index }, attempts, compensations: Vec::new() };
        }

        match Attempted_Step(step, context, index, &mut attempts)
        {
            Ok(outcome) => completed.push(outcome),
            Err(error) =>
            {
                let compensations = Compensated(plan, &completed, context);

                return WorkflowRun { outcome: WorkflowOutcome::Failed { completed, index, error }, attempts, compensations };
            }
        }
    }

    return WorkflowRun { outcome: WorkflowOutcome::Completed { completed }, attempts, compensations: Vec::new() };
}

/// The attempt a step's first dispatch is, counting from one.
const FIRST_ATTEMPT: u32 = 1;

/// How many total attempts `RetryPolicy::NoRetry` permits: the first one, and nothing
/// after it.
const ATTEMPTS_WITHOUT_RETRY: u32 = 1;

/// Dispatches one step, re-dispatching a failed attempt while its own `RetryPolicy` still
/// permits one, and appends every attempt to `attempts` in the order it ran.
///
/// `WF-012`'s own named failure -- repeating a non-idempotent effect with nothing to tell
/// two attempts apart -- is not guarded here, because it is already guarded earlier and
/// better: [`Ran`] refuses an incoherent declaration before this function is ever called,
/// and `WorkflowStep::Is_Coherent` refuses `RetryPolicy::Retry` on a side-effecting,
/// non-idempotent step that requires no deduplication token and declares no compensation.
/// So every step that reaches a second dispatch here reached it under one of those two
/// covers. Re-checking that would be a second authority for a rule the contract already
/// owns.
fn Attempted_Step<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    step: &WorkflowStepPlan, context: &Dispatching<'_, Launcher, Fs, Env>, index: usize, attempts: &mut Vec<StepAttempt>,
) -> Result<StepOutcome, DispatchError>
{
    let allowed = Attempts_Allowed(step.declaration.retry);
    let mut attempt = FIRST_ATTEMPT;

    loop
    {
        let (result, timing) = Timed_Dispatch(&step.body, context, step.declaration.timeout);
        attempts.push(StepAttempt { index, attempt, failure: result.as_ref().err().cloned(), timing });

        match result
        {
            Ok(outcome) => return Ok(outcome),
            Err(error) if attempt >= allowed => return Err(error),
            Err(_) => attempt = attempt.saturating_add(1),
        }
    }
}

/// How many total attempts `retry` permits, including the first.
const fn Attempts_Allowed(retry: RetryPolicy) -> u32
{
    return match retry
    {
        RetryPolicy::NoRetry => ATTEMPTS_WITHOUT_RETRY,
        RetryPolicy::Retry { max_attempts, .. } => max_attempts.get(),
    };
}

/// Dispatches `body` once, and reports what `timeout` measured over that dispatch.
///
/// The bound is measured, not enforced: the dispatch is waited on to its end and the
/// overrun reported afterward. Cutting a dispatch short needs a cancellation runtime,
/// which `OD-WORKFLOW-005` declines and this crate still does not build -- and
/// `CancellationBehavior`, the declaration that would say whether a given step even
/// permits being cut short, is still read by `Is_Coherent` alone.
fn Timed_Dispatch<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    body: &Body, context: &Dispatching<'_, Launcher, Fs, Env>, timeout: Timeout,
) -> (Result<StepOutcome, DispatchError>, StepTiming)
{
    let started = context.read_clock.map(|read| return read());
    let result = Dispatch_Body(body, context.platform, context.variant, context.run);
    let finished = context.read_clock.map(|read| return read());
    let elapsed = started.zip(finished).map(|(from, to)| return to.Since(from));

    return (result, Timing_Of(timeout, elapsed));
}

/// What `timeout` says about a dispatch that took `elapsed`, or that nothing measured it.
///
/// Strictly greater, because `Timeout::Seconds`'s own doc is that the step is no longer
/// waited on *after* that many seconds: a dispatch that took exactly its bound stayed
/// inside it.
fn Timing_Of(timeout: Timeout, elapsed: Option<Duration>) -> StepTiming
{
    let Timeout::Seconds(declared_seconds) = timeout
    else
    {
        return StepTiming::Unbounded;
    };
    let Some(elapsed) = elapsed
    else
    {
        return StepTiming::Unmeasured { declared_seconds };
    };

    let elapsed_seconds = elapsed.as_secs();

    if elapsed_seconds > u64::from(declared_seconds.get())
    {
        return StepTiming::Exceeded { declared_seconds, elapsed_seconds };
    }

    return StepTiming::Honored { declared_seconds, elapsed_seconds };
}

/// Compensates `completed` in the reverse of the order those steps ran, and reports what
/// each step's own `Compensation` declaration did, refused, or owes.
///
/// Reverse order because compensation unwinds: a later step's effect may stand on an
/// earlier step's, so undoing the earlier one first would leave the later one undone
/// against ground that had already moved.
fn Compensated<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    plan: &[WorkflowStepPlan], completed: &[StepOutcome], context: &Dispatching<'_, Launcher, Fs, Env>,
) -> Vec<StepCompensation>
{
    let mut compensations = Vec::new();

    for (index, outcome) in completed.iter().enumerate().rev()
    {
        let Some(step) = plan.get(index)
        else
        {
            continue;
        };

        if let Some(compensation) = Compensated_Step(step, outcome, index, context.platform.filesystem)
        {
            compensations.push(compensation);
        }
    }

    return compensations;
}

/// What compensating one completed step did, or `None` when its own declaration asked for
/// nothing at all.
fn Compensated_Step<Fs: FileSystem>(step: &WorkflowStepPlan, outcome: &StepOutcome, index: usize, filesystem: &Fs) -> Option<StepCompensation>
{
    return match step.declaration.compensation
    {
        Compensation::None => None,
        Compensation::ExternallyCompensated => Some(StepCompensation::Owed { index }),
        Compensation::SelfCompensating => Some(Self_Compensated(&step.body, outcome, index, filesystem)),
    };
}

/// Why a body that is not a [`Body::Correction`] cannot honor
/// `Compensation::SelfCompensating`.
const NO_COMPENSATING_MODE: &str = "this step's body supports no compensating mode: an agent, check or gate body re-invoked \
                                    in a compensating mode is the same dispatch again, not the reverse of it";

/// Runs the compensating mode the step's own body actually supports.
///
/// [`Body::Correction`] is the one of the four with a mode this crate can reach, and it
/// reaches it through its own carried source rather than through
/// `nomos_corrections::CommittedPlan::Rollback`: `Run_Correction` builds that value
/// internally and hands back `CorrectionOutcome::Committed`, which carries rendered
/// strings and no `CommittedPlan` at all, so the receiver rollback needs never crosses the
/// seam. What does cross it is the body's own already-walked `sources` -- the file's
/// content before the correction wrote it -- which is the exact reverse of the one write a
/// committed correction performs.
///
/// Two things that reverse does not have, named rather than implied: it does not assert
/// the workspace has not moved since the commit, which `CommittedPlan::Rollback` does
/// through `Assert_Not_Moved`, so a path a third party edited between the commit and the
/// failure is overwritten rather than refused; and it restores only the one path the
/// outcome names, because that is the only path a correction run reports having written.
fn Self_Compensated<Fs: FileSystem>(body: &Body, outcome: &StepOutcome, index: usize, filesystem: &Fs) -> StepCompensation
{
    return match (body, outcome)
    {
        (Body::Correction(correction), StepOutcome::Correction(CorrectionOutcome::Committed { path, .. })) =>
        {
            Restored_Correction(correction, path, index, filesystem)
        }
        (Body::Correction(_), _) => StepCompensation::Compensated { index, restored: Vec::new() },
        _ => StepCompensation::Refused { index, reason: NO_COMPENSATING_MODE.to_owned() },
    };
}

/// Why a correction step's own compensating mode could not put a path back.
const UNRESTORABLE: &str = "the correction step's own compensating mode could not put back";

/// Why a correction body carries no content for the path its own run committed.
const NO_CARRIED_SOURCE: &str = "the correction body carries no source for the path its run committed, so there is nothing to put back for";

/// Puts `path` back to the content `correction` carried before its own run wrote it.
fn Restored_Correction<Fs: FileSystem>(correction: &crate::CorrectionBody, path: &str, index: usize, filesystem: &Fs) -> StepCompensation
{
    let Some(source) = correction.sources.iter().find(|source| return source.path == path)
    else
    {
        return StepCompensation::Refused { index, reason: format!("{NO_CARRIED_SOURCE} `{path}`") };
    };

    return match filesystem.Replace_Atomically(&correction.root.join(path), &source.text)
    {
        Ok(()) => StepCompensation::Compensated { index, restored: vec![path.to_owned()] },
        Err(error) => StepCompensation::Refused { index, reason: format!("{UNRESTORABLE} `{path}`: {error:?}") },
    };
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
