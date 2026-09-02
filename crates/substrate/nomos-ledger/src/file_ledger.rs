//! The durable ledger: a JSON file, a lock beside it, and the rules it must satisfy.

// file-size: allow: this file is the FileLedger struct, its two impl blocks -- one
// type's own surface, whose doc comments carry the "why" for
// OD-LEDGER-008/009/012/015/019/021 inline -- and that type's own per-method test
// suite. check-test-coverage keys a test's companion unit off the literal file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their one-file address; splitting either impl block would also put one type's
// methods behind two module paths for no cohesion gained.

// The ledger file as a document, beside the reader that parses one.
#[path = "store/ledger_document.rs"]
mod ledger_document;

pub use ledger_document::LedgerDocument;
pub(crate) use ledger_document::VersionProbe;

// Why an add was refused, beside the guard in store.rs that refuses it.
#[path = "store/add_refusal.rs"]
mod add_refusal;

pub use add_refusal::AddRefusal;

#[path = "store/claiming.rs"]
mod claiming;
#[path = "store/document.rs"]
mod document;
#[path = "store/file.rs"]
mod file;
#[path = "store/refusal.rs"]
mod refusal;
#[path = "store/reservation.rs"]
mod reservation;
#[path = "store/rendering.rs"]
mod rendering;
#[path = "store/validation.rs"]
mod validation;
#[path = "store/verbs.rs"]
mod verbs;

// The lock timing and schema version numbers are their own responsibility, tunable
// independent of everything else this file does.
#[path = "store/constants.rs"]
mod constants;

use claiming::{Install_Claim, With_Own_Claim};
use file::{Decide_Under_Lock, Load_Document, Save_Document};
use verbs::{Add_Item, Decline_Item, Take_Over, Validate_Current};

pub use refusal::{Claim_Refusal, Eligible_Items};
pub use validation::Validate_Document;
pub use constants::{LOCK_STALE_AFTER, LOCK_WAIT_LIMIT, SCHEMA_VERSION};

use std::path::{Path, PathBuf};
use std::time::Duration;

use nomos_platform::{Clock, CrossProcessLock, FileSystem, StaleTakeover, Timestamp};

use crate::Claim;
use crate::ClaimRefusal;
use crate::DeclineReason;
use crate::Holder;
use crate::exclusion::{Check_Lease, ExclusionLedger};
use crate::LedgerItem;
use crate::ItemId;
use crate::LedgerError;
use crate::ReleaseOutcome;
use crate::Reservation;
use crate::Territory;

/// A ledger stored as a JSON file, coordinated by a lock beside it.
///
/// # Why a file and not a database
///
/// The ledger is committed, and its readability under `git diff` is load-bearing: it is
/// how a person sees what the agents did to the roadmap. A row in a database has no
/// such review surface. This is a deliberate trade of query power for legibility, and
/// it holds only while the ledger stays small enough to read.
pub struct FileLedger<Files, TimeSource, Lock>
{
    path: PathBuf,
    filesystem: Files,
    clock: TimeSource,
    lock: Lock,
}

impl<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>
FileLedger<Files, TimeSource, Lock>
{
    /// A ledger at the given path.
    pub fn At(path: impl Into<PathBuf>, filesystem: Files, clock: TimeSource, lock: Lock) -> Self
    {
        return Self {
            path: path.into(),
            filesystem,
            clock,
            lock,
        };
    }

    /// The ledger file's path.
    #[must_use]
    pub fn Path(&self) -> &Path
    {
        return &self.path;
    }

    /// Reads a file from the working tree through the ledger's own filesystem.
    ///
    /// Exposed for one reason: finishing an item has to read what the gate checks, and a
    /// caller that reached for `std::fs` instead would bypass the filesystem this ledger
    /// was constructed with, so a test could no longer control what finishing sees.
    ///
    /// # Errors
    ///
    /// Returns the path and the underlying cause, which the caller reports as an
    /// undetermined gate rather than as failing work.
    pub fn Read_File(&self, path: &Path) -> Result<String, String>
    {
        return self
            .filesystem
            .Read_To_String(path)
            .map_err(|error| format!("{error}"));
    }

    /// The time this ledger judges claims and leases against.
    ///
    /// Exposed so that a record written alongside a ledger operation carries the same
    /// clock the operation was decided by. A caller reading the wall clock separately
    /// would stamp evidence from one time base onto a decision made in another, which
    /// is a defect this workspace has already shipped once, in the file lock.
    #[must_use]
    pub fn Now(&self) -> Timestamp
    {
        return self.clock.Now();
    }

    /// Reads the ledger.
    ///
    /// A missing file is an empty ledger, not an error — that is a repository which has
    /// not started tracking work yet. A file that exists and cannot be parsed *is* an
    /// error, because the alternative is treating somebody's corrupted roadmap as an
    /// empty one and cheerfully letting agents claim everything.
    ///
    /// # Why the parse is strict
    ///
    /// This is the single door. `With_Lock`, `Claim`, `Renew`, `Release`, `Conflicts` and
    /// `Validate_Current` all read through here, so a document that got past this point is
    /// one this build accounts for in full — which is what makes re-serializing it in
    /// [`Self::Save`] lossless without a second, separately-forgettable guard there.
    ///
    /// A key no declared type recognises therefore fails here, before anything is written,
    /// rather than being dropped and written back. `OD-LEDGER-008`.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::Malformed`] when the file exists and cannot be parsed,
    /// [`LedgerError::Unrecognized`] when it cannot be parsed *and* says it is newer than
    /// this build, and [`LedgerError::Unreadable`] when it cannot be read at all.
    pub fn Load(&self) -> Result<LedgerDocument, LedgerError>
    {
        return Load_Document(self);
    }

    /// Writes the ledger, refusing to persist one that violates its own invariants.
    ///
    /// Validation happens *before* the write, not after. A ledger that has already been
    /// written and then found invalid is a ledger somebody has to repair by hand, and
    /// in the meantime every agent reading it is reading something the system itself
    /// says is wrong.
    ///
    /// # Why the version is stamped rather than echoed
    ///
    /// What goes to disk says [`SCHEMA_VERSION`], not whatever the loaded document said. A
    /// build that writes a field it invented while echoing the older number it read produces
    /// the one file the version cannot explain: it carries keys an older build must refuse,
    /// and it tells that build they are the same age, so the refusal comes out as
    /// [`LedgerError::Malformed`] and sends somebody to repair a file that is correct.
    ///
    /// No check on the way out. The guard is [`Self::Load`]'s strict parse, and a document
    /// that got through it is one this build accounts for in full, so writing it back is
    /// lossless whatever number it arrived with. A second guard here would be a second
    /// opinion about the same question, and the two would eventually disagree.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::Invalid`] if the document violates an invariant, and
    /// [`LedgerError::Unreadable`] if it cannot be written.
    pub fn Save(&self, document: &LedgerDocument) -> Result<(), LedgerError>
    {
        return Save_Document(self, document);
    }

    /// Reads, modifies and writes the ledger while holding the lock.
    ///
    /// Every mutation goes through here. Read-modify-write without the lock narrows the
    /// window in which two agents lose each other's update; it does not close it, and
    /// the prototype recorded a near-miss where one claim was almost lost that way.
    ///
    /// That sentence was false for eleven weeks, which is what `OD-LEDGER-015` records.
    /// `Claim`, `Renew` and `Release` — the three verbs that change the board — each did
    /// their own `Load`, decided, and `Save`d with nothing held in between, so two sessions
    /// overlapping lost one of the two writes whole. It is stated here rather than only in
    /// the record because a doc comment that describes an arrangement is the thing that
    /// stops being true when the arrangement is bypassed, and nothing checked it.
    ///
    /// # What this must never be wrapped around
    ///
    /// The span held is a read, a decision and a write, all of them milliseconds. It is
    /// **not** the span in which work is verified: [`crate::Finish`] runs an item's
    /// predicate — a test suite, minutes of it — and only then calls `Release`, so what
    /// takes the lock is the recording of the verdict and never the reaching of it. A
    /// caller that put a subprocess inside `modify` would hold a cross-process lock for as
    /// long as that subprocess ran, and every other session on the machine would sit in
    /// [`LOCK_WAIT_LIMIT`] and then fail.
    ///
    /// # Why the write is conditional
    ///
    /// A modification that leaves the document exactly as it was read writes nothing. The
    /// three verbs above all have refusal paths — the item is held by somebody else, the
    /// item does not exist — that decide against changing anything, and rewriting the file
    /// on those paths would mean a refused claim rewrites the roadmap. It also means a
    /// document that is already invalid on disk refuses reads of itself: `Save` validates,
    /// so an unconditional write would turn "your claim is held by agent-b" into "the
    /// ledger is unusable" for a reason having nothing to do with the caller's question.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::Locked`] if the lock cannot be taken, and whatever the
    /// modification or the write returns otherwise.
    pub fn With_Lock<Outcome>(
        &self,
        holder: &str,
        modify: impl FnOnce(&mut LedgerDocument) -> Result<Outcome, LedgerError>,
    ) -> Result<(Outcome, Option<StaleTakeover>), LedgerError>
    {
        let acquisition = self
            .lock
            .Acquire(holder, LOCK_WAIT_LIMIT, LOCK_STALE_AFTER)
            .map_err(|error| LedgerError::Locked {
                cause: error.to_string(),
            })?;

        let read = self.Load()?;
        let mut document = read.clone();
        let outcome = modify(&mut document)?;

        if document != read
        {
            self.Save(&document)?;
        }

        // The takeover travels out with the result rather than being logged here. A
        // caller that surfaces it can tell the user their predecessor abandoned an
        // update; a caller that drops it has made a choice, and this signature is what
        // makes that choice visible in review.
        return Ok((outcome, acquisition.broke_stale));
    }

    /// Takes a lapsed item over, keeping the claim it displaces.
    ///
    /// # Why this is not on [`ExclusionLedger`]
    ///
    /// A lapse is a property of a ledger that outlives its writers. The run-scoped
    /// reservations a correction scheduler holds and the session-scoped leases delegated
    /// agents hold both die with the process that made them, so there is no case in either
    /// where the holder is gone and the record is not. Putting this on the shared trait would
    /// oblige two instances to implement an operation about a failure they cannot have.
    ///
    /// # Why this is not `Claim`
    ///
    /// `claim` never becomes a takeover. Displacing a dead agent's claim is a decision, and a
    /// `claim` that quietly began making it would reintroduce the loss `OD-LEDGER-009`
    /// guarded against: the record would exist and nothing would make the agent creating one
    /// notice that it had. `OD-LEDGER-012`.
    ///
    /// Read, decide and write happen inside one lock acquisition, through
    /// [`Self::Decide_Under_Lock`], because this is a fourth verb that changes the board and
    /// `OD-LEDGER-015` is the record of what the other three cost by deciding outside it.
    ///
    /// # Errors
    ///
    /// Returns [`ClaimRefusal::HeldBy`] if the lease has not run out, [`ClaimRefusal::
    /// NotClaimable`] if the item is not a lapsed claim at all, and whatever a claim would be
    /// refused with otherwise — an unfinished dependency, or ground somebody has taken since
    /// the lapse.
    pub fn Take_Over(
        &mut self,
        item: &ItemId,
        holder: &str,
        lease: Duration,
    ) -> Result<Reservation, ClaimRefusal>
    {
        return Take_Over(self, item, holder, lease);
    }

    /// Ends an item that turned out not to be work.
    ///
    /// # Why this is not a way of releasing a claim
    ///
    /// [`ExclusionLedger::Release`] answers what happened to a *claim* — finished, or given
    /// up — and both of its outcomes leave an item the board can still act on. This answers
    /// what happened to the *item*, and it takes no claim at all: both of the items
    /// `OD-LEDGER-019` was written for had been released long before anybody established they
    /// were superseded, so a verb reachable only through a live claim could not have reached
    /// either of them. Widening `abandon` instead would put a decision in front of the
    /// eighteen releases in twenty that have never needed one, and would still leave the
    /// remaining two behind a claim on work nobody intends to do.
    ///
    /// # Why it is not on [`ExclusionLedger`]
    ///
    /// The reason [`Self::Take_Over`] is not: the trait is the exclusion question, and this is
    /// not one. A declined item excludes nothing, and the run-scoped and session-scoped
    /// instances have no notion of an item outliving the run that would decline it.
    ///
    /// Read, decide and write happen inside one lock acquisition, through
    /// [`Self::Decide_Under_Lock`], for the reason `OD-LEDGER-015` gives about the three verbs
    /// that did not.
    ///
    /// # Errors
    ///
    /// Returns [`ClaimRefusal::StillHeld`] when somebody is holding it — retryable, and the
    /// sentence names the two commands that resolve it — [`ClaimRefusal::Lapsed`] when the
    /// holder is gone rather than working, [`ClaimRefusal::NotClaimable`] when the item is
    /// already `Done` or already `Declined`, and [`ClaimRefusal::NoSuchItem`] for an
    /// identifier that matches nothing.
    pub fn Decline<'a>(
        &mut self,
        item: &ItemId,
        holder: impl Into<Holder<'a>>,
        reason: impl Into<DeclineReason<'a>>,
    ) -> Result<(), ClaimRefusal>
    {
        return Decline_Item(self, item, holder.into(), reason.into());
    }

    /// Puts a new item on the board.
    ///
    /// # Why this is on the store at all
    ///
    /// It was not, and that is what `OD-LEDGER-021` records. `add` lived in the command layer
    /// as a `Load`, a push and a `Save` with nothing held in between — the same shape
    /// `OD-LEDGER-015` had already found in `Claim`, `Renew` and `Release`, surviving in a
    /// fourth verb because those three were fixed by name rather than the door being made the
    /// only way through. An add that read the board before somebody else's claim wrote its
    /// own snapshot back over that claim, and the loss is symmetric: whichever of the two
    /// writers saves last wins whole, so the same defect appears once as a lost claim and once
    /// as an add that returned exit 0 and was never on the board.
    ///
    /// The duplicate check moves inside the lock with the write it guards. Outside it, two
    /// sessions adding one identifier could both read a board without it and both be told it
    /// was theirs, and the second write would leave a document `Validate` calls invalid — a
    /// board that refuses to load, reached by two callers who were each told they succeeded.
    ///
    /// # Errors
    ///
    /// Returns [`AddRefusal::AlreadyPresent`] if the identifier is taken,
    /// [`AddRefusal::WouldBeInvalid`] if the item would leave the board violating its own
    /// invariants — [`Self::Save`] refuses that before anything reaches the disk — and
    /// [`AddRefusal::LedgerUnusable`] if the ledger could not be read or written at all.
    /// Adds an item to the board.
    ///
    /// `published` is the record files this repository has already published, supplied by the
    /// caller rather than discovered here. The store owns the *decision* — `OD-LEDGER-021`
    /// put `add` behind one lock for that reason — and enumerating a repository is not a
    /// thing a general exclusion ledger should learn to do. A caller with nothing to declare
    /// passes [`Territory::Empty`], which is honest: it is saying it does not know, and the
    /// open-item half still holds.
    ///
    /// `amending` is the subset of that reservation the item declares it is *editing* rather
    /// than allocating. Declared by the author and not inferred: an identifier tells you
    /// nothing about which of the two acts is meant, and a rule that guessed from the spelling
    /// of a path would be a convention this board does not enforce.
    ///
    /// # Errors
    ///
    /// [`AddRefusal::AlreadyPresent`] for a duplicate identifier,
    /// [`AddRefusal::RecordPublished`] or [`AddRefusal::RecordReserved`] for a record
    /// identifier that is already spent or already spoken for,
    /// [`AddRefusal::AmendmentNotPublished`] for a declared amendment of a record that does
    /// not exist, [`AddRefusal::AmendmentMisspelled`] for one of a record that does exist
    /// under a different filename, [`AddRefusal::WouldBeInvalid`] for an item that would break
    /// the board's invariants, and [`AddRefusal::LedgerUnusable`] when the file itself cannot
    /// be used.
    pub fn Add(
        &mut self,
        item: &LedgerItem,
        holder: &str,
        published: &Territory,
        amending: &Territory,
    ) -> Result<(), AddRefusal>
    {
        use reservation::RecordDeclaration;

        return Add_Item(
            self,
            item,
            holder,
            &RecordDeclaration {
                published,
                amending,
            },
        );
    }

    /// Whether the ledger currently satisfies its invariants.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::Invalid`] listing every violation.
    pub fn Validate_Current(&self) -> Result<(), LedgerError>
    {
        return Validate_Current(self);
    }
}

impl<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock> ExclusionLedger
for FileLedger<Files, TimeSource, Lock>
{
    fn Claim(
        &mut self,
        item: &ItemId,
        holder: &str,
        lease: Duration,
    ) -> Result<Reservation, ClaimRefusal>
    {
        Check_Lease(lease)?;

        // The refusal is decided and the grant is written under one acquisition. Deciding
        // outside it is not a narrower window, it is the same defect: what the check reads
        // and what the write is based on are the same snapshot, and another session can
        // replace the file between them.
        return Decide_Under_Lock(self, holder, |document, now| {
            let expires_at = now.Plus(lease);

            if let Some(refusal) = Claim_Refusal(document, item, now)
            {
                return Err(refusal);
            }

            let granted = Claim {
                holder: holder.to_owned(),
                acquired_at: now,
                lease_expires_at: expires_at,
            };
            Install_Claim(document, item, &granted);

            return Ok(Reservation {
                item: item.clone(),
                holder: holder.to_owned(),
                expires_at,
            });
        });
    }

    fn Renew(
        &mut self,
        item: &ItemId,
        holder: &str,
        lease: Duration,
    ) -> Result<Reservation, ClaimRefusal>
    {
        Check_Lease(lease)?;

        // A lost renewal does not look like a lost write. It looks like a lease that ran
        // out early, which reads as an agent that died — so this verb being outside the
        // lock sent whoever noticed to investigate the wrong thing.
        return Decide_Under_Lock(self, holder, |document, now| {
            let expires_at = now.Plus(lease);

            With_Own_Claim(document, item, holder, |candidate| {
                if let Some(claim) = &mut candidate.claim
                {
                    claim.lease_expires_at = expires_at;
                }
            })?;

            return Ok(Reservation {
                item: item.clone(),
                holder: holder.to_owned(),
                expires_at,
            });
        });
    }

    fn Release(
        &mut self,
        item: &ItemId,
        holder: &str,
        outcome: ReleaseOutcome,
    ) -> Result<(), ClaimRefusal>
    {
        // This is the write [`crate::Finish`] performs once its predicate has passed, and
        // the reason the lock is taken here rather than around finishing: the predicate is
        // minutes of somebody else's test suite and holds nothing, while the recording of
        // its verdict is this, and is milliseconds. A verdict recorded outside the lock is
        // an agent told its work was written down over a board that has since forgotten it.
        return Decide_Under_Lock(self, holder, |document, now| {
            // Both arms, written once, in `ReleaseOutcome::Record_On`. Spelling them out at
            // the call site is what let this store keep the finished arm's evidence and drop
            // the abandoned arm's.
            return With_Own_Claim(document, item, holder, |candidate| {
                outcome.Record_On(candidate, holder, now);
            });
        });
    }

}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ItemKind, ItemOrigin, ItemState};
    use nomos_platform_std::{FileLock, StdFileSystem};

    /// A clock that never moves, so a test's fixture and its assertions read the same
    /// instant the ledger did.
    struct FixedClock(i64);

    impl Clock for &FixedClock
    {
        fn Now(&self) -> Timestamp
        {
            return Timestamp::From_Unix_Seconds(self.0);
        }
    }

    #[test]
    fn Test_At_Should_Remember_The_Ledger_File_Location()
    {
        let directory = Temporary_Directory("at");
        let clock = FixedClock(1_000);
        let target = directory.join("ledger.json");

        let ledger = Ledger_At(&directory, &clock);

        assert_eq!(ledger.Path(), target.as_path());
    }

    #[test]
    fn Test_Path_Should_Return_The_File_This_Ledger_Reads_And_Writes()
    {
        let directory = Temporary_Directory("path");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);

        assert_eq!(ledger.Path(), directory.join("ledger.json").as_path());
    }

    #[test]
    fn Test_Read_File_Should_Surface_The_Underlying_Cause_As_Text()
    {
        let directory = Temporary_Directory("read-file");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);
        let missing = directory.join("missing.txt");

        let error = ledger.Read_File(&missing).expect_err("a missing file cannot be read");

        assert!(error.contains("does not exist"), "the cause must be legible, got: {error}");
    }

    #[test]
    fn Test_Now_Should_Reflect_The_Ledgers_Own_Clock()
    {
        let directory = Temporary_Directory("now");
        let clock = FixedClock(4_242);
        let ledger = Ledger_At(&directory, &clock);

        assert_eq!(ledger.Now(), Timestamp::From_Unix_Seconds(4_242));
    }

    #[test]
    fn Test_Load_Should_Answer_An_Empty_Document_When_Nothing_Was_Written()
    {
        let directory = Temporary_Directory("load-missing");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);

        let document = ledger.Load().expect("a missing ledger is an empty one, not an error");

        assert_eq!(document.schema_version, SCHEMA_VERSION);
        assert!(document.items.is_empty());
    }

    #[test]
    fn Test_Save_Should_Write_A_Document_That_Reads_Back_Unchanged()
    {
        let directory = Temporary_Directory("save");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);
        let document = LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: vec![Workable_Item("SAVE-1")],
        };

        ledger.Save(&document).expect("a valid document must be writable");
        let reloaded = ledger.Load().expect("what was just written must be readable");

        assert_eq!(reloaded, document);
    }

    #[test]
    fn Test_With_Lock_Should_Run_The_Modification_And_Persist_A_Real_Change()
    {
        let directory = Temporary_Directory("with-lock");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);
        let item = Workable_Item("LOCK-1");

        let (outcome, takeover) = ledger
            .With_Lock("agent-a", |document| {
                document.items.push(item.clone());
                return Ok(item.id.clone());
            })
            .expect("a plain modification must succeed");

        assert_eq!(outcome, item.id);
        assert!(takeover.is_none(), "a fresh lock is never a stale takeover");
        let reloaded = ledger.Load().expect("the modification must have been written");
        assert_eq!(reloaded.items.len(), 1);
    }

    #[test]
    fn Test_Take_Over_Should_Replace_A_Lapsed_Claim_With_A_Fresh_One()
    {
        let directory = Temporary_Directory("take-over");
        let clock = FixedClock(10_000);
        let mut ledger = Ledger_At(&directory, &clock);
        let mut item = Workable_Item("TAKE-1");
        item.state = ItemState::Claimed;
        item.claim = Some(Claim {
            holder: "dead-agent".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(1_000),
            lease_expires_at: Timestamp::From_Unix_Seconds(2_000),
        });
        ledger
            .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![item] })
            .expect("a claimed item is still a valid document");

        let reservation = ledger
            .Take_Over(&ItemId::New("TAKE-1"), "agent-b", Duration::from_secs(3_600))
            .expect("a lapsed claim must be takeable");

        assert_eq!(reservation.holder, "agent-b");
        let reloaded = ledger.Load().expect("the takeover must have been written");
        let taken = reloaded.items.first().expect("the item survives its takeover");
        assert_eq!(
            taken.claim.as_ref().map(|claim| return claim.holder.as_str()),
            Some("agent-b")
        );
        assert_eq!(taken.displaced.len(), 1, "the displaced claim must be kept, not dropped");
    }

    #[test]
    fn Test_Decline_Should_End_A_Ready_Item_With_No_Claim_At_All()
    {
        let directory = Temporary_Directory("decline");
        let clock = FixedClock(1_000);
        let mut ledger = Ledger_At(&directory, &clock);
        ledger
            .Save(&LedgerDocument {
                schema_version: SCHEMA_VERSION,
                items: vec![Workable_Item("DECLINE-1")],
            })
            .expect("a ready item is a valid document");

        ledger
            .Decline(&ItemId::New("DECLINE-1"), "agent-a", "superseded")
            .expect("a ready, unclaimed item may be declined");

        let reloaded = ledger.Load().expect("the decline must have been written");
        let declined = reloaded.items.first().expect("the item survives its decline");
        assert_eq!(
            declined.state,
            ItemState::Declined { reason: "superseded".to_owned() }
        );
    }

    #[test]
    fn Test_Add_Should_Put_A_New_Item_On_The_Board()
    {
        let directory = Temporary_Directory("add");
        let clock = FixedClock(1_000);
        let mut ledger = Ledger_At(&directory, &clock);
        let item = Workable_Item("ADD-1");

        ledger
            .Add(&item, "agent-a", &Territory::Empty(), &Territory::Empty())
            .expect("a fresh identifier over an empty board must be accepted");

        let reloaded = ledger.Load().expect("the add must have been written");
        assert_eq!(reloaded.items.len(), 1);
        assert_eq!(reloaded.items.first().expect("the assertion above found exactly one item").id, item.id);
    }

    #[test]
    fn Test_Validate_Current_Should_Report_A_Duplicate_Identifier_Written_To_Disk()
    {
        let directory = Temporary_Directory("validate-current");
        let clock = FixedClock(1_000);
        let ledger = Ledger_At(&directory, &clock);
        // Written directly rather than through `Save`, which would itself refuse this
        // document -- this test is about `Validate_Current` surfacing what is already on
        // disk, not about `Save`'s own guard.
        let duplicated = LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: vec![Workable_Item("DUP-1"), Workable_Item("DUP-1")],
        };
        let raw = serde_json::to_string(&duplicated).expect("the fixture document serializes");
        std::fs::write(ledger.Path(), raw).expect("test can write the raw fixture directly");

        let error = ledger
            .Validate_Current()
            .expect_err("a repeated identifier violates an invariant");

        assert!(
            matches!(
                &error,
                LedgerError::Invalid { violations } if violations.iter().any(|line| line.contains("more than once"))
            ),
            "expected a duplicate-identifier violation, got {error:?}"
        );
    }

    /// A scratch directory this test owns outright, named for the test so two tests
    /// running at once never share one file.
    fn Temporary_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-file-ledger-{name}-{}", std::process::id()));
        // error-info: allow this is a best-effort clean slate before creating the directory fresh below
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    /// A real ledger over a real file, exactly as every caller of this type gets one.
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

    /// A `Ready` item with a non-empty territory of its own, so it can sit on a board
    /// without itself violating [`Validate_Document`]'s "reserves nothing" rule.
    fn Workable_Item(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([format!("src/{id}.rs")]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            declined: None,
        };
    }
}
