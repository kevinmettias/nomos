//! The durable ledger: a JSON file, a lock beside it, and the rules it must satisfy.

// The ledger file as a document, beside the reader that parses one.
mod ledger_document;

pub use ledger_document::LedgerDocument;
pub(crate) use ledger_document::VersionProbe;

// Why an add was refused, beside the guard in store.rs that refuses it.
mod add_refusal;

pub use add_refusal::AddRefusal;

mod claiming;
mod document;
mod file;
mod refusal;
mod reservation;
mod rendering;
mod validation;
mod verbs;

use claiming::{Install_Claim, With_Own_Claim};
use file::{Decide_Under_Lock, Load, Save};
use verbs::{Add, Decline, Take_Over, Validate_Current};

pub use refusal::Claim_Refusal;
pub use validation::Validate;

use std::path::{Path, PathBuf};
use std::time::Duration;

use nomos_platform::{Clock, CrossProcessLock, FileSystem, StaleTakeover, Timestamp};

use crate::Claim;
use crate::ClaimRefusal;
use crate::exclusion::{Check_Lease, ExclusionLedger};
use crate::LedgerItem;
use crate::ItemId;
use crate::LedgerError;
use crate::ReleaseOutcome;
use crate::Reservation;
use crate::Territory;

/// How long to wait for the ledger lock before giving up.
pub const LOCK_WAIT_LIMIT: Duration = Duration::from_secs(20);

/// How old a ledger lock must be before it may be broken.
///
/// Generous relative to how long a ledger write takes — writes are milliseconds — so
/// that breaking one is genuinely evidence the holder died rather than evidence the
/// machine was briefly busy.
pub const LOCK_STALE_AFTER: Duration = Duration::from_secs(15 * 60);

/// The highest ledger schema version this build can account for.
///
/// Written into every file this build saves, and compared against a file this build failed to
/// read. It does not *guarantee* anything: the guarantee is `deny_unknown_fields` on every
/// container reachable from [`LedgerDocument`], which is mechanical and cannot be forgotten.
/// This number's only job is to decide which sentence an operator whose parse just failed
/// reads — [`LedgerError::Unrecognized`] rather than [`LedgerError::Malformed`].
///
/// That ordering is deliberate and is `OD-LEDGER-008`'s decision. `e88f92d` added a field to
/// [`crate::LedgerItem`] and raised nothing, so a guard resting on the bump would report clean
/// on the next instance of the defect it was built for. Here a forgotten bump can only degrade
/// a message, and can never cost a field.
///
/// `5` since `OD-LEDGER-024` added [`crate::LedgerItem::kind`] and
/// [`crate::LedgerItem::origin`]; `4` was `OD-LEDGER-027`'s
/// [`crate::VerificationRecord::revision`]; `3` was `OD-LEDGER-019`'s
/// [`crate::LedgerItem::declined`]; `2` was `OD-LEDGER-012`'s [`crate::LedgerItem::displaced`].
/// The bump is not discretionary: `Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version`
/// counts the keys on a serialized item, so a field arriving without this number moving is a
/// refusal that misstates why.
///
/// `kind` and `origin` carry no `#[serde(default)]`, unlike every field before them — every
/// item on the board was migrated to carry both in the same commit that raised this number,
/// so a build older than this one refuses the file outright rather than silently accepting a
/// row missing either. `nomos work validate` prints the same two numbers on request, which is
/// how to tell before that refusal arrives rather than at it.
pub const SCHEMA_VERSION: u32 = 5;

/// A ledger stored as a JSON file, coordinated by a lock beside it.
///
/// # Why a file and not a database
///
/// The ledger is committed, and its readability under `git diff` is load-bearing: it is
/// how a person sees what the agents did to the roadmap. A row in a database has no
/// such review surface. This is a deliberate trade of query power for legibility, and
/// it holds only while the ledger stays small enough to read.
pub struct FileLedger<F, C, L>
{
    path: PathBuf,
    filesystem: F,
    clock: C,
    lock: L,
}

impl<F: FileSystem, C: Clock, L: CrossProcessLock> FileLedger<F, C, L>
{
    /// A ledger at the given path.
    pub fn At(path: impl Into<PathBuf>, filesystem: F, clock: C, lock: L) -> Self
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
        return Load(self);
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
        return Save(self, document);
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
    pub fn With_Lock<T>(
        &self,
        holder: &str,
        modify: impl FnOnce(&mut LedgerDocument) -> Result<T, LedgerError>,
    ) -> Result<(T, Option<StaleTakeover>), LedgerError>
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
    pub fn Decline(
        &mut self,
        item: &ItemId,
        holder: &str,
        reason: &str,
    ) -> Result<(), ClaimRefusal>
    {
        return Decline(self, item, holder, reason);
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
    /// not exist, [`AddRefusal::WouldBeInvalid`] for an item that would break the board's
    /// invariants, and [`AddRefusal::LedgerUnusable`] when the file itself cannot be used.
    pub fn Add(
        &mut self,
        item: &LedgerItem,
        holder: &str,
        published: &Territory,
        amending: &Territory,
    ) -> Result<(), AddRefusal>
    {
        use reservation::RecordDeclaration;

        return Add(
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

impl<F: FileSystem, C: Clock, L: CrossProcessLock> ExclusionLedger for FileLedger<F, C, L>
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
