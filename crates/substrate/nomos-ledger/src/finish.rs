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

#[path = "finish/finish_refusal.rs"]
mod refusal;
mod running;
mod gate_step;
#[cfg(test)]
mod tests;

pub use refusal::FinishRefusal;
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
    Clock, Command, FilesystemLock, ExitOutcome, FileSystem, ProgramLauncher, Timestamp,
};
use std::path::{Path, PathBuf};

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
pub fn Finish_Item<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProgramLauncher,
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
fn Loaded_Document<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
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
fn Verify_Predicate<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProgramLauncher,
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
/// Reads `HEAD` for the tree under `working_directory` (or `.` when the predicate runs
/// where the caller already is, matching [`Workflow_Path`]'s own fallback) through the
/// ledger's own [`FileLedger::Read_File`], and resolves the ref it names. `None` on any
/// failure along the way -- see [`VerificationRecord::revision`] for why that is not
/// distinguished further.
///
/// Three shapes of ordinary checkout reach this, not one, and joining `.git/HEAD` plus one
/// loose ref resolves only the simplest of them. A linked worktree's `.git` is a *file*
/// holding a `gitdir:` pointer; a worktree checked out on a branch keeps its own `HEAD`
/// while that branch lives in the common directory `commondir` names; and `git pack-refs`
/// moves a branch out of `refs/` and into `packed-refs` altogether. None of the three is
/// exotic -- the first is what this repository's own procedure prescribes for finishing
/// against a contended tree, and the third is how an ordinary clone arrives -- so each
/// returning `None` was one defect rather than three edge cases.
///
/// It stays a read rather than a subprocess, as `OD-LEDGER-027` chose, because the
/// ledger's own file port expresses all three shapes without one.
fn Current_Revision<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    working_directory: Option<&Path>,
) -> Option<String>
{
    let tree = working_directory.unwrap_or_else(|| return Path::new("."));
    let git = Git_Directories(ledger, tree);
    let head = ledger.Read_File(&git.own.join("HEAD")).ok()?;
    let head = head.trim();

    if let Some(name) = head.strip_prefix("ref: ")
    {
        return Resolved_Ref(ledger, &git, name.trim());
    }

    return Non_Empty(head);
}

/// Where a tree's git metadata lives: the directory belonging to this checkout, and the
/// one shared with every other checkout of the same repository.
///
/// The two are the same directory for an ordinary clone. They differ for a linked
/// worktree, which keeps its own `HEAD` and index while the branches that `HEAD` may name
/// stay in the common directory, so a resolution that knows only one of them can read a
/// worktree's `HEAD` and still fail to resolve what it points at.
struct GitDirectories
{
    own: PathBuf,
    common: PathBuf,
}

/// The two directories [`Current_Revision`] resolves `HEAD` and its ref against.
fn Git_Directories<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    tree: &Path,
) -> GitDirectories
{
    let own = Linked_Git_Directory(ledger, tree).unwrap_or_else(|| return tree.join(".git"));
    let common = Common_Git_Directory(ledger, &own);

    return GitDirectories { own, common };
}

/// The directory a linked worktree's `.git` file points at.
///
/// `None` when `.git` is an ordinary directory, which is exactly the case where reading it
/// as a file fails -- so the two shapes are told apart by whether the read succeeds and
/// names a `gitdir:`, rather than by asking the port a question it does not answer.
fn Linked_Git_Directory<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    tree: &Path,
) -> Option<PathBuf>
{
    let pointer = ledger.Read_File(&tree.join(".git")).ok()?;
    let target = pointer.trim().strip_prefix("gitdir:")?.trim();

    return Some(tree.join(target));
}

/// The directory shared with the repository's other checkouts: what `commondir` names for
/// a linked worktree, and the checkout's own directory when there is no such file.
fn Common_Git_Directory<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    own: &Path,
) -> PathBuf
{
    return ledger.Read_File(&own.join("commondir")).ok().map_or_else(
        || return own.to_owned(),
        |pointer| return own.join(pointer.trim()),
    );
}

/// The commit a named ref resolves to, preferring a loose file to a packed entry.
///
/// That order is not a tie-break chosen for symmetry. `packed-refs` is a snapshot from
/// whenever it was last written, so a loose file beside it is the newer answer -- this
/// repository's own `packed-refs` names `refs/heads/dev` a month behind the loose file
/// next to it. Reading the packed entry first would stamp a revision that is *wrong*
/// rather than one that is absent, which is the worse of the two failures.
fn Resolved_Ref<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    git: &GitDirectories,
    name: &str,
) -> Option<String>
{
    return Loose_Ref(ledger, &git.own, name)
        .or_else(|| return Loose_Ref(ledger, &git.common, name))
        .or_else(|| return Packed_Ref(ledger, &git.common, name));
}

/// The commit a ref's own file holds, when the ref has one.
fn Loose_Ref<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    directory: &Path,
    name: &str,
) -> Option<String>
{
    return Non_Empty(ledger.Read_File(&directory.join(name)).ok()?.trim());
}

/// The commit `packed-refs` records for a ref, when the ref has been packed away.
fn Packed_Ref<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    common: &Path,
    name: &str,
) -> Option<String>
{
    let packed = ledger.Read_File(&common.join("packed-refs")).ok()?;

    return packed.lines().find_map(|line| return Packed_Entry(line, name));
}

/// One `packed-refs` line, read as an entry for `name`.
///
/// A leading `#` is the file's own header and a leading `^` is the commit an annotated tag
/// peels to; neither is a ref line, and the second would otherwise answer for whichever
/// ref happened to precede it.
fn Packed_Entry(line: &str, name: &str) -> Option<String>
{
    let line = line.trim();
    if line.starts_with('#') || line.starts_with('^')
    {
        return None;
    }

    let (object, packed) = line.split_once(' ')?;
    if packed.trim() != name
    {
        return None;
    }

    return Non_Empty(object);
}

/// Text that names something, or `None` for text that does not.
///
/// A ref file caught half-written reads as empty, and an empty revision would be a stamp
/// claiming a tree that no commit id names -- which is the one outcome worse than the
/// honest absence [`VerificationRecord::revision`] already accounts for.
fn Non_Empty(text: &str) -> Option<String>
{
    if text.is_empty()
    {
        return None;
    }

    return Some(text.to_owned());
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
    use nomos_platform::ProgramOutput;
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

    /// How long the fixture claim's lease runs, measured from [`NOW`]. Long enough that the
    /// item is still held at every instant the cases here ask about.
    const CLAIM_LEASE_SECONDS: i64 = 3_600;
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

    impl ProgramLauncher for &AlwaysZero
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput {
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
                lease_expires_at: Timestamp::From_Unix_Seconds(NOW + CLAIM_LEASE_SECONDS),
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

        let record = Finished_Record(&mut ledger, &directory, &item_id);

        Assert_Verification_Recorded(&record);
        Assert_Claim_Closed(&ledger);
    }

    fn Temporary_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-finish-item-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
        }
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

    /// Runs `item`'s predicate through [`Finish_Item`] against a launcher that always exits
    /// zero, and answers with the record a green gate left behind. Panics with the refusal
    /// when the finish is refused, which is the failure the caller is testing for.
    fn Finished_Record(
        ledger: &mut FileLedger<StdFileSystem, &FixedClock, FileLock>,
        directory: &Path,
        item: &ItemId,
    ) -> VerificationRecord
    {
        return Finish_Item(
            ledger,
            &&AlwaysZero,
            &Finishing { item, holder: HOLDER },
            Some(directory),
        )
        .expect("a zero-exit predicate behind a green gate must finish");
    }

    /// The record a passing predicate left behind, checked for the predicate's own verdict
    /// and the gate's together.
    fn Assert_Verification_Recorded(record: &VerificationRecord)
    {
        assert_eq!(record.exit_code, 0);
        assert!(
            record.gate.is_some(),
            "the gate's own outcome must ride along with the predicate's"
        );
    }

    /// The item the finish was applied to, checked as closed: `Done`, and no longer held.
    fn Assert_Claim_Closed(ledger: &FileLedger<StdFileSystem, &FixedClock, FileLock>)
    {
        let reloaded = ledger.Load().expect("the release must have been written");
        let closed = reloaded.items.first().expect("the item survives finishing");
        assert_eq!(closed.state, ItemState::Done);
        assert_eq!(closed.claim, None, "a finished item is no longer held");
    }

    /// The commit the fixture checkouts below resolve to. Nothing here parses it; a shape
    /// a reader recognises as a commit id simply keeps the fixtures legible.
    const RESOLVED_COMMIT: &str = "0c0b699bd4f02bad3e83592ee8a12d4c128e5c53";

    /// A second commit id, for the one case where two sources name the same ref and the
    /// test has to say which of them was believed.
    const SUPERSEDED_COMMIT: &str = "333d619b1005de338b19717cd56ab33315767a0e";

    /// The branch the fixture checkouts sit on.
    const FIXTURE_BRANCH: &str = "refs/heads/dev";

    /// A `packed-refs` file of the shape `git pack-refs` writes: a leading comment, the
    /// branch this fixture asks about, an unrelated branch, and a peeled-tag line. The
    /// last three exist so that the entry actually has to be *found* rather than simply
    /// being the only thing in the file.
    fn Packed_Refs(dev: &str) -> String
    {
        return format!(
            "# pack-refs with: peeled fully-peeled sorted \n\
             {dev} {FIXTURE_BRANCH}\n\
             914a091c85eca8922c1964d94cfbaf3acae5f99a refs/heads/main\n\
             ^{RESOLVED_COMMIT}\n"
        );
    }

    /// Writes a fixture file, creating the directories above it.
    fn Write_Under(path: &Path, contents: &str)
    {
        let parent = path.parent().expect("a fixture path has a directory above it");
        std::fs::create_dir_all(parent).expect("test needs the directory above its fixture");
        std::fs::write(path, contents).expect("test needs to write its fixture");
    }

    /// [`Current_Revision`] as a finish calls it, against a fixture checkout.
    fn Revision_At(directory: &Path) -> Option<String>
    {
        let clock = FixedClock(NOW);
        let ledger = Ledger_At(directory, &clock);

        return Current_Revision(&ledger, Some(directory));
    }

    /// The defect this item was raised for. A linked worktree's `.git` is a *file* holding
    /// a `gitdir:` pointer rather than a directory, so a resolution that only ever joined
    /// `.git/HEAD` found nothing and left the record with no revision at all -- and
    /// finishing from a worktree is not exotic, it is what this repository's own procedure
    /// prescribes whenever a peer's in-flight work reddens the shared tree. The stamp was
    /// therefore absent exactly when the tree was most contended.
    #[test]
    fn Test_A_Detached_Worktree_Should_Resolve_Through_Its_Gitdir_Pointer()
    {
        let directory = Temporary_Directory("gitdir-pointer");
        let git = directory.join("metadata").join("worktrees").join("w");
        Write_Under(&git.join("HEAD"), RESOLVED_COMMIT);
        Write_Under(&directory.join(".git"), &format!("gitdir: {}\n", git.display()));

        assert_eq!(
            Revision_At(&directory).as_deref(),
            Some(RESOLVED_COMMIT),
            "a worktree's `.git` names where its metadata lives, and a finish taken there \
             must still record the tree the predicate ran against"
        );
    }

    /// The second half of the worktree shape. A worktree checked out on a branch keeps its
    /// own `HEAD`, but the branch that `HEAD` names lives in the *common* directory, which
    /// `commondir` points at. Resolving the ref beside the worktree's own `HEAD` finds
    /// nothing, so covering only the detached case would leave this one absent.
    #[test]
    fn Test_A_Worktree_On_A_Branch_Should_Resolve_Its_Ref_From_The_Common_Directory()
    {
        let directory = Temporary_Directory("worktree-branch");
        let common = directory.join("metadata");
        let git = common.join("worktrees").join("w");
        Write_Under(&git.join("HEAD"), &format!("ref: {FIXTURE_BRANCH}\n"));
        Write_Under(&git.join("commondir"), "../..\n");
        Write_Under(&common.join(FIXTURE_BRANCH), RESOLVED_COMMIT);
        Write_Under(&directory.join(".git"), &format!("gitdir: {}\n", git.display()));

        assert_eq!(
            Revision_At(&directory).as_deref(),
            Some(RESOLVED_COMMIT),
            "a worktree's branch ref lives in the common directory `commondir` names, not \
             beside the worktree's own `HEAD`"
        );
    }

    /// The shape that needs no worktree at all: `git pack-refs` moves a branch out of
    /// `refs/heads/` and into one `packed-refs` file, after which reading the loose path
    /// finds nothing. An ordinary clone arrives packed, so a guard covering only the
    /// worktree shape would still lose the stamp here.
    #[test]
    fn Test_A_Ref_That_Is_Packed_Rather_Than_Loose_Should_Still_Resolve()
    {
        let directory = Temporary_Directory("packed-ref");
        let git = directory.join(".git");
        Write_Under(&git.join("HEAD"), &format!("ref: {FIXTURE_BRANCH}\n"));
        Write_Under(&git.join("packed-refs"), &Packed_Refs(RESOLVED_COMMIT));

        assert_eq!(
            Revision_At(&directory).as_deref(),
            Some(RESOLVED_COMMIT),
            "a packed branch is an ordinary state of an ordinary checkout, not a corrupt one"
        );
    }

    /// Which source wins when both carry the ref, which is not a tie-break invented for
    /// symmetry: `packed-refs` is a snapshot from whenever it was last written, and this
    /// repository's own copy names `refs/heads/dev` at a commit a month behind the loose
    /// file beside it. Reading the packed entry first would stamp a revision that is wrong
    /// rather than absent, which is the worse of the two failures.
    #[test]
    fn Test_A_Loose_Ref_Should_Be_Preferred_To_A_Superseded_Packed_Entry()
    {
        let directory = Temporary_Directory("loose-over-packed");
        let git = directory.join(".git");
        Write_Under(&git.join("HEAD"), &format!("ref: {FIXTURE_BRANCH}\n"));
        Write_Under(&git.join("packed-refs"), &Packed_Refs(SUPERSEDED_COMMIT));
        Write_Under(&git.join(FIXTURE_BRANCH), RESOLVED_COMMIT);

        assert_eq!(
            Revision_At(&directory).as_deref(),
            Some(RESOLVED_COMMIT),
            "a loose ref is the current one and the packed entry may be stale, so a stamp \
             must never be taken from the packed file while a loose file exists"
        );
    }

    /// The shape that already worked, kept so that repairing the other three cannot
    /// quietly cost the ordinary checkout its stamp.
    #[test]
    fn Test_An_Ordinary_Checkout_Should_Still_Resolve_Its_Loose_Ref()
    {
        let directory = Temporary_Directory("loose-ref");
        let git = directory.join(".git");
        Write_Under(&git.join("HEAD"), &format!("ref: {FIXTURE_BRANCH}\n"));
        Write_Under(&git.join(FIXTURE_BRANCH), RESOLVED_COMMIT);

        assert_eq!(Revision_At(&directory).as_deref(), Some(RESOLVED_COMMIT));
    }

    /// Where no revision can be resolved at all the absence stays undistinguished, which
    /// is [`VerificationRecord::revision`]'s own reasoning and is not reopened here.
    #[test]
    fn Test_A_Tree_With_No_Git_Metadata_Should_Resolve_To_No_Revision()
    {
        let directory = Temporary_Directory("no-metadata");

        assert_eq!(Revision_At(&directory), None);
    }
}
