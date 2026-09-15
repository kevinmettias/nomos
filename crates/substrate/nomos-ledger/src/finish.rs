//! Finishing an item, which means running something rather than saying something.
//!
//! `done_when` is prose. A person reads it, agrees with it, and moves on. That is how
//! the prototype's ledger accumulated items marked complete whose work had not been
//! done: nothing stood between the claim of completion and the record of it.
//!
//! [`Finish_Item`] is what stands there. It runs the item's [`VerificationPredicate`] and
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
pub(super) mod release_outcome;

pub use abandonment::Abandonment;
pub use declination::Declination;

#[path = "finish/refusal.rs"]
mod refusal;
mod running;
mod gate_step;
#[cfg(test)]
mod tests;

pub use refusal::Refusal as FinishRefusal;
use refusal::Tail_Of;
use running::{Command_From_Argv, Ran, Ran_To_Completion, Refuse_Nonzero, Runnable_Predicate, Runner};

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
pub fn Finish_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProcessLauncher,
    finishing: &Finishing<'_>,
    working_directory: Option<&Path>,
) -> Result<VerificationRecord, FinishRefusal>
{
    use release_outcome::ReleaseOutcome;

    let item = finishing.item;
    let document = Loaded_Document(ledger)?;
    let predicate = Runnable_Predicate(&document, item)?;
    let runner = Runner {
        working_directory,
        timeout: std::time::Duration::from_secs(predicate.timeout_seconds),
    };

    let (gate, ran) = Verify_Predicate(ledger, launcher, item, PredicateRun { predicate, runner })?;

    let revision = Current_Revision(ledger, working_directory);
    let record = Record_From_Argv(&predicate.argv, &ran, gate, RecordContext { at: ledger.Now(), revision });
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

/// The ledger as it stands, or the reason it could not be read.
///
/// A ledger that will not load is `NotRecorded` rather than `NotHeld`: nothing was found
/// out about the claim, and reporting it as unheld would send the author to re-claim an
/// item they may well still hold.
fn Loaded_Document<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
) -> Result<LedgerDocument, FinishRefusal>
{
    return ledger.Load().map_err(|error| {
        return FinishRefusal::NotRecorded {
            cause: error.to_string(),
        };
    });
}

/// Runs the gate's step, then the item's own predicate, refusing on either's failure.
///
/// The gate's own step runs first and short-circuits: an author told "your tests passed"
/// and "you cannot land" in one breath reads only the first sentence.
fn Verify_Predicate<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProcessLauncher,
    item: &ItemId,
    run: PredicateRun<'_>,
) -> Result<(GateOutcome, Ran), FinishRefusal>
{
    let gate = gate_step::Run_Gate_Step(ledger, launcher, item, run.runner)?;

    let command = Command_From_Argv(run.predicate.argv.clone(), run.runner);
    let ran = Ran_To_Completion(launcher, &command, item)?;
    Refuse_Nonzero(item, ran.code, &ran.tail)?;

    return Ok((gate, ran));
}

/// When, and against which tree revision, a predicate was verified — grouped so
/// [`Record_From_Argv`] stays under this crate's own parameter-count ceiling.
struct RecordContext
{
    at: Timestamp,
    revision: Option<String>,
}

/// The tree's current revision, read directly rather than shelled out to `git`.
///
/// Reads `.git/HEAD` under `working_directory` (or `.` when the predicate runs where the
/// caller already is, matching [`Workflow_Path`]'s own fallback) through the ledger's own
/// [`FileLedger::Read_File`], and follows one loose ref if `HEAD` names one rather than
/// naming a commit directly. `None` on any failure along the way -- see
/// [`VerificationRecord::revision`] for why that is not distinguished further.
fn Current_Revision<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
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

/// The record a passing predicate leaves behind.
///
/// It carries the gate's outcome as well as its own, because "this item was verified" is
/// only true of a tree the gate also accepted.
fn Record_From_Argv(argv: &[String], ran: &Ran, gate: GateOutcome, context: RecordContext) -> VerificationRecord
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

#[cfg(test)]
mod local_tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    // A SEPARATE, literal `#[cfg(test)] mod tests` (`finish/tests.rs`) already exercises this
    // module's exported behaviour in depth. It cannot address `Finish_Item` itself:
    // `check-test-coverage` keys a test's companion unit off the file it is textually written
    // in, and `finish/tests.rs` is a different file with its own unit. This second, literal
    // inline module gives `Finish_Item` the one-file address the check reads, without
    // disturbing that broader suite.
    use super::*;
    use crate::{Claim, ItemKind, ItemOrigin, ItemState, Territory};
    use nomos_platform::ProcessOutput;
    use nomos_platform_std::{FileLock, StdFileSystem};
    use std::path::PathBuf;

    struct FixedClock(i64);

    /// Fixed instants, so both the values and their timing reproduce.
    impl Strategy for FixedClock
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl Clock for &FixedClock
    {
        fn Now(&self) -> Timestamp
        {
            return Timestamp::From_Unix_Seconds(self.0);
        }
    }

    const NOW: i64 = 1_000_000;
    const HOLDER: &str = "agent-a";

    const WORKFLOW: &str = "name: gate\n\
                            \n\
                            jobs:\n\
                            \x20 gate:\n\
                            \x20   steps:\n\
                            \x20     - uses: actions/checkout@v4\n\
                            \x20     - name: Lint\n\
                            \x20       run: cargo clippy --workspace --all-targets -- -D warnings\n\
                            \x20     - name: Test\n\
                            \x20       run: cargo test --workspace\n";

    /// A launcher that always exits zero, standing in for a lint step and a predicate that
    /// both pass.
    struct AlwaysZero;

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for AlwaysZero
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProcessLauncher for &AlwaysZero
    {
        fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
        {
            return Ok(ProcessOutput {
                outcome: ExitOutcome::Exited { code: 0 },
                stdout: "all good".to_owned(),
                stderr: String::new(),
            });
        }
    }

    fn Claimed_Item(id: ItemId) -> crate::LedgerItem
    {
        return crate::LedgerItem {
            id,
            title: "an item".to_owned(),
            why: "it needs doing".to_owned(),
            done_when: "the predicate passes".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files(["src/a.rs"]),
            state: ItemState::Claimed,
            depends_on: Vec::new(),
            blocked: None,
            claim: Some(Claim {
                holder: HOLDER.to_owned(),
                acquired_at: Timestamp::From_Unix_Seconds(NOW),
                lease_expires_at: Timestamp::From_Unix_Seconds(NOW + 3_600),
            }),
            verification: Some(VerificationPredicate::From_String_Arguments(vec![
                "a-predicate".to_owned(),
            ])),
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            widened: Vec::new(),
            declined: None,
        };
    }

    /// The property this whole module exists for, stated over the one function that ties its
    /// pieces together: a predicate that passes behind a green gate closes the claim and
    /// leaves a verification record behind, rather than merely saying it did.
    #[test]
    fn Test_Finish_Item_Should_Record_A_Passing_Predicate_And_Close_The_Claim()
    {
        let directory = Temporary_Directory("passing");
        Write_Workflow(&directory);
        let clock = FixedClock(NOW);
        let mut ledger = Ledger_At(&directory, &clock);
        let item_id = ItemId::New("F-1");
        ledger
            .Save(&LedgerDocument {
                schema_version: crate::SCHEMA_VERSION,
                items: vec![Claimed_Item(item_id.clone())],
            })
            .expect("a claimed item is a valid document");
        let launcher = AlwaysZero;

        let record = Finish_Item(
            &mut ledger,
            &&launcher,
            &Finishing { item: &item_id, holder: HOLDER },
            Some(&directory),
        )
        .expect("a zero-exit predicate behind a green gate must finish");

        assert_eq!(record.exit_code, 0);
        assert!(record.gate.is_some(), "the gate's own outcome must ride along with the predicate's");

        let reloaded = ledger.Load().expect("the release must have been written");
        let closed = reloaded.items.first().expect("the item survives finishing");
        assert_eq!(closed.state, ItemState::Done);
        assert_eq!(closed.claim, None, "a finished item is no longer held");
    }

    fn Temporary_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-finish-item-{name}-{}", std::process::id()));
        // error-info: allow this is a best-effort clean slate before creating the directory fresh below
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    fn Write_Workflow(directory: &Path)
    {
        let workflows = directory.join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
        std::fs::write(workflows.join("gate.yml"), WORKFLOW).expect("test needs a workflow");
    }

    fn Ledger_At<'clock>(
        directory: &Path,
        clock: &'clock FixedClock,
    ) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
    {
        return FileLedger::At(
            directory.join("ledger.json"),
            StdFileSystem,
            clock,
            FileLock::At(directory.join("ledger.lock")),
        );
    }
}
