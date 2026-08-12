//! Finishing an item, which means running something rather than saying something.
//!
//! `done_when` is prose. A person reads it, agrees with it, and moves on. That is how
//! the prototype's ledger accumulated items marked complete whose work had not been
//! done: nothing stood between the claim of completion and the record of it.
//!
//! [`Finish`] is what stands there. It runs the item's [`VerificationPredicate`] and
//! writes the result into the item, so `Done` is a state the ledger arrives at by
//! observation.

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
use gate_step::Run_Gate_Step;

use crate::finishing::Finishing;
use crate::claim::ClaimRefusal;
use crate::exclusion::ExclusionLedger;
use crate::gate::Derive_Step;
use crate::gate::GateUnknown;
use crate::gate::LINT_STEP;
use crate::gate::Workflow_Path;
use crate::gate::GateOutcome;
use crate::item::ItemId;
use crate::verification::VerificationPredicate;
use crate::verification::VerificationRecord;
use crate::store::FileLedger;
use crate::ledger_document::LedgerDocument;
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

    // The gate's own step runs first and short-circuits. An author told "your tests passed"
    // and "you cannot land" in one breath reads only the first sentence.
    let gate = Run_Gate_Step(ledger, launcher, item, runner)?;

    let command = Commanded(predicate.argv.clone(), runner);
    let ran = Ran_To_Completion(launcher, &command, item)?;
    Refuse_Nonzero(item, ran.code, &ran.tail)?;

    let record = Verified(&predicate.argv, &ran, gate, ledger.Now());
    ledger
        .Release(item, finishing.holder, ReleaseOutcome::Finished(record.clone()))
        .map_err(|refusal| FinishRefusal::NotHeld { refusal })?;

    return Ok(record);
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

/// The record a passing predicate leaves behind.
///
/// It carries the gate's outcome as well as its own, because "this item was verified" is
/// only true of a tree the gate also accepted.
fn Verified(argv: &[String], ran: &Ran, gate: GateOutcome, at: Timestamp) -> VerificationRecord
{
    return VerificationRecord {
        argv: argv.to_vec(),
        exit_code: ran.code,
        output_tail: ran.tail.clone(),
        verified_at: at,
        gate: Some(gate),
    };
}
