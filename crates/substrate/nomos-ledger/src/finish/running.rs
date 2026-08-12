//! Running a command to a verdict, and refusing one that did not reach one.

use super::{ItemId, FinishRefusal, Path, LedgerDocument, VerificationPredicate, ClaimRefusal, ProcessLauncher, Command, Tail_Of, ExitOutcome};

/// A predicate that ran and said no.
///
/// Its tail is carried into the refusal rather than only its code, because "not finished"
/// is not actionable and "not finished, here is what the run printed" is.
pub(super) fn Refuse_Nonzero(item: &ItemId, code: i32, tail: &str) -> Result<(), FinishRefusal>
{
    if code == 0
    {
        return Ok(());
    }

    return Err(FinishRefusal::PredicateFailed {
        item: item.clone(),
        exit_code: code,
        output_tail: tail.to_owned(),
    });
}

/// How a command is to be run: from where, and for how long at most.
///
/// The timeout is the item's own, so the gate step is bounded by the same patience the
/// predicate is. A gate that hung forever would refuse the item for a reason nobody could
/// distinguish from work that never finished.
#[derive(Clone, Copy)]
pub(super) struct Runner<'a>
{
    pub(super) working_directory: Option<&'a Path>,
    pub(super) timeout: std::time::Duration,
}

/// The item's predicate, if it has one that can actually be run.
///
/// Prose in `done_when` is not a predicate and a predicate with no program is not one
/// either. Both would let an item be finished on a check nobody performed.
pub(super) fn Runnable_Predicate<'a>(
    document: &'a LedgerDocument,
    item: &ItemId,
) -> Result<&'a VerificationPredicate, FinishRefusal>
{
    let predicate = Predicate_Of(document, item)?;
    if !predicate.Is_Runnable()
    {
        return Err(FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause: "the predicate has no program to run".to_owned(),
        });
    }

    return Ok(predicate);
}

/// The predicate an item declares, if the item exists and declares one.
pub(super) fn Predicate_Of<'a>(
    document: &'a LedgerDocument,
    item: &ItemId,
) -> Result<&'a VerificationPredicate, FinishRefusal>
{
    let found = document.items.iter().find(|candidate| return &candidate.id == item);
    let Some(target) = found
    else
    {
        return Err(FinishRefusal::NotHeld {
            refusal: ClaimRefusal::NoSuchItem { item: item.clone() },
        });
    };

    let Some(predicate) = &target.verification
    else
    {
        return Err(FinishRefusal::NoPredicate {
            item: item.clone(),
            done_when: target.done_when.clone(),
        });
    };

    return Ok(predicate);
}

/// One command that reached a verdict, and the tail of what it said on the way.
pub(super) struct Ran
{
    pub(super) code: i32,
    pub(super) tail: String,
}

/// Runs a command to a verdict.
///
/// Both callers need the same three answers — the launcher would not start it, it ended
/// without a code, it ended with one — and answering them once is what keeps the gate step
/// and the predicate from drifting apart in how they treat a process nobody could ask.
pub(super) fn Ran_To_Completion(
    launcher: &impl ProcessLauncher,
    command: &Command,
    item: &ItemId,
) -> Result<Ran, FinishRefusal>
{
    use super::refusal::OUTPUT_TAIL_LIMIT;

    let output = launcher
        .Run(command)
        .map_err(|cause| FinishRefusal::CouldNotRun {
            item: item.clone(),
            cause,
        })?;
    let tail = Tail_Of(&format!("{}{}", output.stdout, output.stderr), OUTPUT_TAIL_LIMIT);

    let ExitOutcome::Exited { code } = output.outcome
    else
    {
        return Err(FinishRefusal::NoVerdict {
            item: item.clone(),
            outcome: output.outcome,
        });
    };

    return Ok(Ran { code, tail });
}

/// A command as this module builds them: what to run, where, and how long to wait.
pub(super) fn Commanded(argv: Vec<String>, runner: Runner<'_>) -> Command
{
    let mut command = Command::New(argv, runner.timeout);
    command.working_directory = runner.working_directory.map(std::path::Path::to_path_buf);

    return command;
}
