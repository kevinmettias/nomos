//! The durable ledger: a JSON file, a lock beside it, and the rules it must satisfy.

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

// The two files a board is kept in, beside the type that reads one of them. Not in
// `constants.rs`: that file is this ledger's tunable numbers, and these are a published
// format rather than a number anyone may turn.
#[path = "store/board_files.rs"]
mod board_files;

// The bodies of the three verbs `ExclusionLedger` declares, beside the file's other
// bodies. They answer one question between them -- whether a holder may have an item,
// and for how long -- which is why they are together and not scattered.
#[path = "store/exclusion_verbs.rs"]
mod exclusion_verbs;

use exclusion_verbs::{Claim_Item, Release_Item, Renew_Item};
use file::{Load_Document, Save_Document, With_Lock_Run};
use verbs::{Add_Item, Decline_Item, Take_Over, Validate_Current, Widen_Territory};

pub use refusal::{Claim_Refusal, Eligible_Items};
pub use validation::Validate_Document;
pub use constants::{LOCK_STALE_AFTER, LOCK_WAIT_LIMIT, SCHEMA_VERSION};
pub use board_files::{BoardFiles, Board_In, DOCUMENT_FILENAME, LOCK_FILENAME};

use std::path::{Path, PathBuf};
use std::time::Duration;

use nomos_platform::{Clock, CrossProcessLock, FileSystem, StaleTakeover, Timestamp};

use crate::ClaimRefusal;
use crate::DeclineReason;
use crate::Holder;
use crate::exclusion::ExclusionLedger;
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
        return With_Lock_Run(self, holder, modify);
    }

    /// Enlarges a held item's territory, keeping what the enlargement added.
    ///
    /// A reservation is authored before the change it reserves has been attempted, so it is a
    /// prediction, and a prediction is sometimes wrong by one file. Without this the only
    /// repair is to abandon, decline and re-author -- which ends the item, strands every
    /// dependent, and destroys the evidence that the prediction was short at the moment it is
    /// produced. `OD-LEDGER-039`.
    ///
    /// # Why this is not on [`ExclusionLedger`]
    ///
    /// For the reason [`Self::Take_Over`] is not: the run-scoped reservations a correction
    /// scheduler holds and the session-scoped leases delegated agents hold both die with the
    /// process that made them, so neither has an author who authored a territory and a later
    /// execution to disagree with it. Putting this on the shared trait would oblige two
    /// instances to implement a repair for a mistake they cannot make.
    ///
    /// # What it will not do
    ///
    /// It only adds. There is no argument here that could express a replacement territory:
    /// dropping a path drops the `done_when` clause that path carried, and a verb taking a
    /// whole new territory would put narrowing one typo away from a holder who meant to add
    /// one file. A path already reserved contributes nothing and is not recorded as added.
    ///
    /// It is not a licence to reserve loosely and discover territory as you go. The
    /// measurement is the reason the enlargement is kept, and an escape rate says nothing
    /// unless the reservations it is measured against were genuine attempts to get the
    /// territory right.
    ///
    /// # Errors
    ///
    /// [`ClaimRefusal::Lapsed`] if the lease has run out -- a lapsed claim stops excluding, so
    /// enlarging one reaches ground a peer may legitimately hold since, and the remedy named is
    /// `takeover`. [`ClaimRefusal::StillHeld`] if somebody else holds it,
    /// [`ClaimRefusal::NotClaimable`] if nobody does or the item has ended, and whatever a
    /// claim over the enlarged territory would be refused with otherwise.
    pub fn Widen(
        &mut self,
        item: &ItemId,
        holder: Holder<'_>,
        adding: &[String],
    ) -> Result<Vec<String>, ClaimRefusal>
    {
        return Widen_Territory(self, item, holder, adding);
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
        return Claim_Item(self, item, holder, lease);
    }

    fn Renew(
        &mut self,
        item: &ItemId,
        holder: &str,
        lease: Duration,
    ) -> Result<Reservation, ClaimRefusal>
    {
        return Renew_Item(self, item, holder, lease);
    }

    fn Release(
        &mut self,
        item: &ItemId,
        holder: &str,
        outcome: ReleaseOutcome,
    ) -> Result<(), ClaimRefusal>
    {
        return Release_Item(self, item, holder, outcome);
    }

}

#[cfg(test)]
#[path = "store/tests/file_ledger.rs"]
mod tests;
