//! Running the shared gate step the predicate does not cover.

use super::{FileSystem, Clock, CrossProcessLock, FileLedger, ItemId, Path, FinishRefusal, Workflow_Path, GateUnknown, Derive_Step, LINT_STEP, ProcessLauncher, Runner, GateOutcome, Commanded, Ran_To_Completion};

/// The gate's lint step, read out of the workflow rather than written here.
///
/// Both failures are `GateUndetermined` rather than a licence to run the predicate alone:
/// a workflow nobody could read and a workflow with no such step both leave the question
/// "would this land" unanswered, and that is not the same as answering it yes.
pub(super) fn Gate_Argv<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
    item: &ItemId,
    working_directory: Option<&Path>,
) -> Result<Vec<String>, FinishRefusal>
{
    let workflow_path = Workflow_Path(working_directory.unwrap_or_else(|| return Path::new(".")));
    let workflow = ledger
        .Read_File(&workflow_path)
        .map_err(|cause| FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::Unreadable {
                path: workflow_path.display().to_string(),
                cause,
            },
        })?;

    return Derive_Step(&workflow, LINT_STEP).map_err(|cause| {
        return FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause,
        };
    });
}

/// Runs the gate's lint step, derived from the workflow rather than written here.
///
/// Separate from [`Finish`] because it is a separate question. [`Finish`] asks whether the
/// item's own predicate holds; this asks whether the work can land at all, and the second
/// question is the one every item's predicate was silently skipping.
///
/// # Errors
///
/// Returns [`FinishRefusal::GateUndetermined`] when what the gate checks cannot be
/// established — which is a refusal, not a licence to run the predicate alone — and
/// [`FinishRefusal::GateFailed`] when the step ran and the answer was no.
pub(super) fn Run_Gate_Step<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
    launcher: &impl ProcessLauncher,
    item: &ItemId,
    runner: Runner<'_>,
) -> Result<GateOutcome, FinishRefusal>
{
    let argv = Gate_Argv(ledger, item, runner.working_directory)?;
    let command = Commanded(argv.clone(), runner);
    let ran = Ran_To_Completion(launcher, &command, item)?;

    if ran.code != 0
    {
        return Err(FinishRefusal::GateFailed {
            item: item.clone(),
            argv,
            exit_code: ran.code,
            output_tail: ran.tail,
        });
    }

    return Ok(GateOutcome {
        argv,
        exit_code: ran.code,
    });
}
