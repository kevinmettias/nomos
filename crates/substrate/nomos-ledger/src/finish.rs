//! Finishing an item, which means running something rather than saying something.
//!
//! `done_when` is prose. A person reads it, agrees with it, and moves on. That is how
//! the prototype's ledger accumulated items marked complete whose work had not been
//! done: nothing stood between the claim of completion and the record of it.
//!
//! [`Finish`] is what stands there. It runs the item's [`VerificationPredicate`] and
//! writes the result into the item, so `Done` is a state the ledger arrives at by
//! observation.

// A finish in progress, beside the verb that runs it.
mod finishing;

pub use finishing::Finishing;

// The three ways an item stops being held without being finished. They sit beside the
// finish they are the alternatives to, rather than in a `finishing` of their own that a
// reader would have had to tell apart from this one by opening both.
mod abandonment;
mod declination;
mod release_outcome;

pub use abandonment::Abandonment;
pub use declination::Declination;
pub use release_outcome::ReleaseOutcome;

mod refusal;
mod running;
mod gate_step;
#[cfg(test)]
mod tests;

pub use refusal::FinishRefusal;
use refusal::Tail_Of;
use running::{Commanded, Ran, Ran_To_Completion, Refuse_Nonzero, Runnable_Predicate, Runner};

use crate::ClaimRefusal;
use crate::ExclusionLedger;
use crate::Derive_Step;
use crate::GateUnknown;
use crate::LINT_STEP;
use crate::Workflow_Path;
use crate::GateOutcome;
use crate::ItemId;
use crate::VerificationPredicate;
use crate::VerificationRecord;
use crate::FileLedger;
use crate::LedgerDocument;
use nomos_platform::{
    Clock, Command, CrossProcessLock, ExitOutcome, FileSystem, ProcessLauncher, Timestamp,
};
use std::path::Path;

/// Runs an item's verification predicate and, if it passes, records the item as done.
///
/// `working_directory` of `None` runs the predicate wherever the caller already is,
/// which for a repository tool invoked from a repository is the right answer. A test, or
/// a caller that is not where it wants the predicate to run, names the directory.
///
/// # Errors
///
/// Returns a [`FinishRefusal`] naming what stopped it, and in particular distinguishing
/// a failing predicate from one that could not be asked.
pub fn Finish<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    launcher: &impl ProcessLauncher,
    finishing: &Finishing<'_>,
    working_directory: Option<&Path>,
) -> Result<VerificationRecord, FinishRefusal>
{
    let item = finishing.item;
    let document = Loaded(ledger)?;
    let predicate = Runnable_Predicate(&document, item)?;
    let runner = Runner {
        working_directory,
        timeout: std::time::Duration::from_secs(predicate.timeout_seconds),
    };

    let (gate, ran) = Verify_Predicate(ledger, launcher, item, PredicateRun { predicate, runner })?;

    let revision = Current_Revision(ledger, working_directory);
    let record = Verified(&predicate.argv, &ran, gate, RecordContext { at: ledger.Now(), revision });
    ledger
        .Release(item, finishing.holder, ReleaseOutcome::Finished(record.clone()))
        .map_err(|refusal| FinishRefusal::NotHeld { refusal })?;

    return Ok(record);
}

/// A predicate paired with how it is to be run — the two [`Verify_Predicate`] needs
/// together for both the gate's step and the predicate's own, grouped so the function that
/// takes them stays under this crate's own parameter-count ceiling.
struct PredicateRun<'a>
{
    predicate: &'a VerificationPredicate,
    runner: Runner<'a>,
}

/// Runs the gate's step, then the item's own predicate, refusing on either's failure.
///
/// The gate's own step runs first and short-circuits: an author told "your tests passed"
/// and "you cannot land" in one breath reads only the first sentence.
fn Verify_Predicate<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    launcher: &impl ProcessLauncher,
    item: &ItemId,
    run: PredicateRun<'_>,
) -> Result<(GateOutcome, Ran), FinishRefusal>
{
    let gate = gate_step::Run_Gate_Step(ledger, launcher, item, run.runner)?;

    let command = Commanded(run.predicate.argv.clone(), run.runner);
    let ran = Ran_To_Completion(launcher, &command, item)?;
    Refuse_Nonzero(item, ran.code, &ran.tail)?;

    return Ok((gate, ran));
}

/// The ledger as it stands, or the reason it could not be read.
///
/// A ledger that will not load is `NotRecorded` rather than `NotHeld`: nothing was found
/// out about the claim, and reporting it as unheld would send the author to re-claim an
/// item they may well still hold.
fn Loaded<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
) -> Result<LedgerDocument, FinishRefusal>
{
    return ledger.Load().map_err(|error| {
        return FinishRefusal::NotRecorded {
            cause: error.to_string(),
        };
    });
}

/// When, and against which tree revision, a predicate was verified — grouped so
/// [`Verified`] stays under this crate's own parameter-count ceiling.
struct RecordContext
{
    at: Timestamp,
    revision: Option<String>,
}

/// The record a passing predicate leaves behind.
///
/// It carries the gate's outcome as well as its own, because "this item was verified" is
/// only true of a tree the gate also accepted.
fn Verified(argv: &[String], ran: &Ran, gate: GateOutcome, context: RecordContext) -> VerificationRecord
{
    return VerificationRecord {
        argv: argv.to_vec(),
        exit_code: ran.code,
        output_tail: ran.tail.clone(),
        verified_at: context.at,
        gate: Some(gate),
        revision: context.revision,
    };
}

/// The tree's current revision, read directly rather than shelled out to `git`.
///
/// Reads `.git/HEAD` under `working_directory` (or `.` when the predicate runs where the
/// caller already is, matching [`Workflow_Path`]'s own fallback) through the ledger's own
/// [`FileLedger::Read_File`], and follows one loose ref if `HEAD` names one rather than
/// naming a commit directly. `None` on any failure along the way -- see
/// [`VerificationRecord::revision`] for why that is not distinguished further.
fn Current_Revision<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
    working_directory: Option<&Path>,
) -> Option<String>
{
    let tree = working_directory.unwrap_or_else(|| return Path::new("."));
    let head = ledger.Read_File(&tree.join(".git").join("HEAD")).ok()?;
    let head = head.trim();

    if let Some(ref_path) = head.strip_prefix("ref: ")
    {
        return ledger
            .Read_File(&tree.join(".git").join(ref_path))
            .ok()
            .map(|contents| return contents.trim().to_owned());
    }

    return Some(head.to_owned());
}
