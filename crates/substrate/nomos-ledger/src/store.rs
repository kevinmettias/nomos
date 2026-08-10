//! The durable ledger: a JSON file, a lock beside it, and the rules it must satisfy.

use crate::exclusion::{
    Check_Lease, ClaimRefusal, ExclusionLedger, Refusal_From, ReleaseOutcome, Reservation,
};
use crate::item::{Claim, ItemId, ItemState, LedgerItem};
use crate::territory::{Normalize_Path, Territory};
use nomos_model::Intersection;
use nomos_platform::{Clock, CrossProcessLock, FileSystem, StaleTakeover, Timestamp};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
/// `3` since `OD-LEDGER-019` added [`crate::LedgerItem::declined`]; `2` was
/// `OD-LEDGER-012`'s [`crate::LedgerItem::displaced`]. The bump is not discretionary:
/// `Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version` counts the keys on a
/// serialized item, so a field arriving without this number moving is a refusal that
/// misstates why.
pub const SCHEMA_VERSION: u32 = 3;

/// Why a ledger operation could not be carried out.
#[derive(Debug)]
pub enum LedgerError
{
    /// The ledger file could not be read or written.
    Unreadable
    {
        /// What went wrong.
        cause: String,
    },
    /// The ledger file exists and is not valid.
    Malformed
    {
        /// What is wrong with it.
        cause: String,
    },
    /// The lock could not be taken.
    Locked
    {
        /// What went wrong.
        cause: String,
    },
    /// The ledger's own invariants are violated.
    Invalid
    {
        /// Every violation found, not just the first.
        violations: Vec<String>,
    },
    /// The ledger holds something this build cannot account for.
    ///
    /// Distinct from [`LedgerError::Malformed`] because the two remedies are opposites: a
    /// malformed ledger is repaired, and this one is left alone while the *reader* is
    /// rebuilt. Reporting the second as the first sends an operator to edit a file that is
    /// correct, which is the "two causes wearing one name" shape `OD-LEDGER-009` names,
    /// with the causes swapped.
    Unrecognized
    {
        /// What this build understands.
        understood: u32,
        /// What the file says it is.
        found: u32,
        /// What could not be accounted for, verbatim from the parser.
        cause: String,
    },
}

impl core::fmt::Display for LedgerError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unreadable { cause } => write!(formatter, "ledger could not be read: {cause}"),
            Self::Malformed { cause } => write!(formatter, "ledger is malformed: {cause}"),
            Self::Locked { cause } => write!(formatter, "ledger is locked: {cause}"),
            Self::Invalid { violations } => write!(
                formatter,
                "ledger is invalid:\n  {}",
                violations.join("\n  ")
            ),
            Self::Unrecognized {
                understood,
                found,
                cause,
            } => write!(
                formatter,
                "this build understands ledger schema {understood} and the file is schema \
                 {found}: {cause}. Writing it back would drop what could not be read, so \
                 nothing was written. Rebuild (`cargo build -p nomos-cli`) and retry"
            ),
        };
    }
}

impl std::error::Error for LedgerError {}

/// Why an item could not be put on the board.
///
/// Its own vocabulary and not a borrowed [`ClaimRefusal`] arm. Adding an item is not
/// claiming one — it takes no territory, judges no lease and consults no other holder — so
/// every arm of a claim's refusal would be a sentence about the wrong question. That is the
/// mis-subject `OD-LEDGER-014` measured, and the cost of a second small enum is smaller than
/// the cost of one arm meaning two things.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddRefusal
{
    /// The identifier is already on the board.
    ///
    /// An answer and not a store failure: the caller chose an identifier somebody else has
    /// already used, and the remedy is to choose another. Decided **inside** the lock, so two
    /// sessions adding one identifier at once cannot both be told it was free.
    AlreadyPresent
    {
        /// The identifier that is taken.
        item: ItemId,
    },
    /// The territory reserves a record identifier this repository has already published.
    ///
    /// Distinct from [`AddRefusal::RecordReserved`] because the remedies are different and an
    /// author told the wrong one does the wrong thing. A published record is spent forever —
    /// the identifier is allocated and the file exists — so the only fix is choosing another
    /// number. A reserved one belongs to an item that may yet be retired.
    ///
    /// Names the file rather than only the identifier, because an author who reads
    /// "`OD-LEDGER-025` is taken" has to go and find out by what, and the thing that answers
    /// that is a filename.
    ///
    /// A published identifier excludes nobody, which is why nothing caught this before:
    /// `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` reddens on two *open* items
    /// sharing an identifier, and one open item on a spent one is invisible to it.
    RecordPublished
    {
        /// The identifier, in the folded form both spellings reach.
        identifier: String,
        /// The record file that already carries it, as the caller found it.
        file: String,
    },
    /// The territory reserves a record identifier another open item already reserves.
    ///
    /// Decided inside the lock, and that is the whole of why it is here rather than in the
    /// command layer. Two sessions each taking the next free number read a board without the
    /// other's item and are both told it is free — which is exactly how `P11-DISPATCH-SPLIT`
    /// and this item's own first reissue both took `OD-LEDGER-022`, one add landing between
    /// the other's board read and its own.
    ///
    /// Only open items reserve. A `Done` or `Declined` item's territory is history, and
    /// refusing against it would make every closed item a permanent claim on its number.
    RecordReserved
    {
        /// The identifier, in the folded form both spellings reach.
        identifier: String,
        /// The open item that already reserves it.
        item: ItemId,
    },
    /// The item would leave the board violating its own invariants.
    ///
    /// The commonest of these is an item that reserves nothing, which `AGENTS.md` states as a
    /// rule of the board: it would exclude nobody while looking like work.
    ///
    /// Distinct from [`AddRefusal::LedgerUnusable`] because the remedies are opposite and the
    /// exit codes differ. This one is the caller's own item to correct and the board is fine;
    /// that one means nobody can use the board until somebody looks at it. Collapsing them is
    /// how "your territory is empty" comes to read as "stop and fetch a person".
    WouldBeInvalid
    {
        /// Every violation the document would carry, not just the first.
        violations: Vec<String>,
    },
    /// The ledger itself could not be read or written.
    ///
    /// Its own arm for the reason [`ClaimRefusal::LedgerUnusable`] is: a caller told only
    /// "that identifier is taken" while the file is in fact unparseable goes and renames its
    /// item, and the rename does not help. `OD-LEDGER-009`.
    LedgerUnusable
    {
        /// What the store said, verbatim.
        cause: String,
    },
}

impl AddRefusal
{
    /// A one-line explanation a person or an agent can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::AlreadyPresent { item } => format!("{item} is already on the ledger"),
            // Each names what to do next, because the two are told apart by the remedy and
            // an author who read only "taken" would pick the wrong one half the time.
            Self::RecordPublished { identifier, file } => format!(
                "{identifier} is already published as {file}. A record identifier is \
                 allocated once; choose the next free one"
            ),
            Self::RecordReserved { identifier, item } => format!(
                "{identifier} is already reserved by {item}, which is open. Choose another \
                 identifier, or retire that item if it is not work"
            ),
            // The wording [`LedgerError::Invalid`] would have produced, because this arm
            // exists to carry that refusal out through a different channel and not to
            // rephrase it. An operator who has seen one of these should recognise the other.
            Self::WouldBeInvalid { violations } =>
            {
                format!("ledger is invalid:\n  {}", violations.join("\n  "))
            }
            Self::LedgerUnusable { cause } => cause.clone(),
        };
    }
}

/// Which of a store failure's two meanings this is, for a caller adding an item.
///
/// [`LedgerError::Invalid`] arrives here by a different route from the rest. The others are
/// the store failing at its job; that one is [`FileLedger::Save`] doing its job, refusing a
/// document before it reaches the disk because the item just handed to it is not one the
/// board can hold. Reporting the second as the first is the conflation `OD-LEDGER-009`
/// records, and here it would cost the exit code an agent branches on.
impl From<&LedgerError> for AddRefusal
{
    fn from(error: &LedgerError) -> Self
    {
        return match error
        {
            LedgerError::Invalid { violations } => Self::WouldBeInvalid {
                violations: violations.clone(),
            },
            other => Self::LedgerUnusable {
                cause: other.to_string(),
            },
        };
    }
}

/// The on-disk form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
// The outermost of the strict containers. Reasoned once in `item.rs`'s module documentation and
// decided in `OD-LEDGER-008`: a build that cannot account for every key in the ledger does not
// get to write the ledger back.
#[serde(deny_unknown_fields)]
pub struct LedgerDocument
{
    /// Schema version, so a future reader can tell what it is looking at.
    pub schema_version: u32,
    /// The items, in a stable order.
    pub items: Vec<LedgerItem>,
}

/// The schema version alone, for explaining a strict parse that already failed.
///
/// A separate type, and deliberately *not* `deny_unknown_fields`: its whole job is to read one
/// field out of a document [`LedgerDocument`] has refused, which every key it does not declare
/// is the reason for.
#[derive(Deserialize)]
struct VersionProbe
{
    schema_version: u32,
}

/// Which of the two parse failures this is.
///
/// A file newer than this build and a file that is simply broken both fail
/// `serde_json::from_str`, and the operator's next action is opposite in the two cases: rebuild
/// the reader, or repair the file. Telling them apart is the whole of what [`SCHEMA_VERSION`]
/// does — it is consulted here, after the refusal, and never to decide whether to refuse.
///
/// A forgotten bump therefore degrades this to [`LedgerError::Malformed`] and costs a sentence.
/// It cannot cost a field.
fn Explain(path: &Path, text: &str, error: &serde_json::Error) -> LedgerError
{
    if let Ok(probe) = serde_json::from_str::<VersionProbe>(text)
        && probe.schema_version > SCHEMA_VERSION
    {
        return LedgerError::Unrecognized {
            understood: SCHEMA_VERSION,
            found: probe.schema_version,
            cause: error.to_string(),
        };
    }

    return LedgerError::Malformed {
        cause: format!("{}: {error}", path.display()),
    };
}

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
        if !self.filesystem.Exists(&self.path)
        {
            return Ok(LedgerDocument {
                schema_version: SCHEMA_VERSION,
                items: Vec::new(),
            });
        }

        let text = self
            .filesystem
            .Read_To_String(&self.path)
            .map_err(|error| LedgerError::Unreadable {
                cause: error.to_string(),
            })?;

        return serde_json::from_str::<LedgerDocument>(&text)
            .map_err(|error| return Explain(&self.path, &text, &error));
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
        let violations = Validate(document, self.clock.Now());
        if !violations.is_empty()
        {
            return Err(LedgerError::Invalid { violations });
        }

        let rendered = Rendered(document)?;

        return self
            .filesystem
            .Replace_Atomically(&self.path, &rendered)
            .map_err(|error| LedgerError::Unreadable {
                cause: error.to_string(),
            });
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

    /// Runs a decision that may refuse, over the document, inside one lock acquisition.
    ///
    /// The [`ExclusionLedger`] verbs share a shape that [`Self::With_Lock`] cannot express on
    /// its own: they answer with a refusal rather than a [`LedgerError`], and a refusal is an
    /// *answer*, not a failure of the store. Carrying it out through the error channel would
    /// put "agent-b holds this" and "the file will not parse" in one type, which is the
    /// conflation `OD-LEDGER-009` already had to undo once. So the refusal rides out as the
    /// modification's value, and only genuine store failures use the error.
    ///
    /// Written once and called from every verb rather than spelled out in each. A copy of
    /// "take the lock, read the clock, decide, write" per verb is a chance per verb for one of
    /// them to stop taking the lock — which is the defect this exists to have fixed, and which
    /// `add` then went on to demonstrate anyway by never being routed through here at all.
    /// `OD-LEDGER-021`.
    ///
    /// # Why the refusal type is generic
    ///
    /// It was [`ClaimRefusal`] concretely while the only callers were the three claim verbs.
    /// `add` refuses for a reason that is not about claiming — the identifier is already on
    /// the board — and giving it a [`ClaimRefusal`] arm to borrow would have been the
    /// mis-subject `OD-LEDGER-014` measured. The alternative, letting `add` reach for
    /// [`Self::With_Lock`] directly, is the copy this function exists to prevent. So the door
    /// stays single and each verb brings its own vocabulary, bound only by being able to say
    /// "the store itself failed".
    ///
    /// `now` is read **inside** the acquisition and handed to the decision, so the clock a
    /// claim is judged against and the clock its lease is measured from are one reading
    /// taken after the wait for the lock. Read before, a claim that waited on a contended
    /// lock would be granted a lease shortened by however long it waited, and would judge
    /// other holders' leases against a time that had already passed.
    fn Decide_Under_Lock<T, E>(
        &self,
        holder: &str,
        decide: impl FnOnce(&mut LedgerDocument, Timestamp) -> Result<T, E>,
    ) -> Result<T, E>
    where
        for<'error> E: From<&'error LedgerError>,
    {
        // The stale takeover is dropped here, deliberately and visibly. None of the three
        // verbs' return types can carry one — `Reservation` and `ClaimRefusal` are public
        // and adding a field or a variant to either is a change to the crate's surface,
        // which this item did not have. What is lost is a diagnostic and not consistency:
        // a broken lock is only ever broken after `LOCK_STALE_AFTER`, and `Save` replaces
        // the file atomically, so the document a takeover finds is whole either way.
        let (outcome, _takeover) = self
            .With_Lock(holder, |document| {
                let now = self.clock.Now();

                return Ok(decide(document, now));
            })
            .map_err(|error| return E::from(&error))?;

        return outcome;
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
        Check_Lease(lease)?;

        return self.Decide_Under_Lock(holder, |document, now| {
            let expires_at = now.Plus(lease);

            if let Some(refusal) = Takeover_Refusal(document, item, now)
            {
                return Err(refusal);
            }

            let replacement = Claim {
                holder: holder.to_owned(),
                acquired_at: now,
                lease_expires_at: expires_at,
            };
            Replace_Lapsed(document, item, replacement, now)?;

            return Ok(Reservation {
                item: item.clone(),
                holder: holder.to_owned(),
                expires_at,
            });
        });
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
        return self.Decide_Under_Lock(holder, |document, now| {
            if let Some(refusal) = Decline_Refusal(document, item, now)
            {
                return Err(refusal);
            }

            for candidate in &mut document.items
            {
                if &candidate.id == item
                {
                    // `LedgerItem::Decline` and not two statements here, for the reason
                    // `Replace_Lapsed_Claim` is one call: a call site that wrote the state
                    // itself would be free to write it and not the declination, and the
                    // declination is the half this verb was added to keep.
                    candidate.Decline(reason, holder, now);
                }
            }

            return Ok(());
        });
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
    /// # Errors
    ///
    /// [`AddRefusal::AlreadyPresent`] for a duplicate identifier,
    /// [`AddRefusal::RecordPublished`] or [`AddRefusal::RecordReserved`] for a record
    /// identifier that is already spent or already spoken for,
    /// [`AddRefusal::WouldBeInvalid`] for an item that would break the board's invariants,
    /// and [`AddRefusal::LedgerUnusable`] when the file itself cannot be used.
    pub fn Add(
        &mut self,
        item: &LedgerItem,
        holder: &str,
        published: &Territory,
    ) -> Result<(), AddRefusal>
    {
        return self.Decide_Under_Lock(holder, |document, _now| {
            if document
                .items
                .iter()
                .any(|existing| existing.id == item.id)
            {
                return Err(AddRefusal::AlreadyPresent {
                    item: item.id.clone(),
                });
            }

            Refuse_A_Spent_Record(item, document, published)?;

            document.items.push(item.clone());

            return Ok(());
        });
    }

    /// Whether the ledger currently satisfies its invariants.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerError::Invalid`] listing every violation.
    pub fn Validate_Current(&self) -> Result<(), LedgerError>
    {
        let violations = Validate(&self.Load()?, self.clock.Now());

        return if violations.is_empty()
        {
            Ok(())
        }
        else
        {
            Err(LedgerError::Invalid { violations })
        };
    }
}

/// Refuses an item whose territory reserves a record identifier that is already spent.
///
/// Two comparisons, in the order an author can act on. A published identifier is spent
/// forever and the remedy is unconditional; a reserved one may be released, so being told
/// about it second is being told about the one that might still resolve itself.
///
/// Both are decided by [`Territory::Intersect`], one authored path at a time so the refusal
/// can name which. That is not a second containment rule beside the first: `Intersect` folds
/// a record filename onto the identifier it carries, which is what makes
/// `docs/records/OD-LEDGER-025` and `docs/records/OD-LEDGER-025-a-slug.md` one subject
/// without anything here knowing the grammar. `OD-LEDGER-016` is that decision and this is
/// the second caller to rely on it.
fn Refuse_A_Spent_Record(
    item: &LedgerItem,
    document: &LedgerDocument,
    published: &Territory,
) -> Result<(), AddRefusal>
{
    for reserved in Record_Reservations(&item.territory)
    {
        let mine = Territory::Of_Files([reserved.clone()]);

        Refuse_If_Published(&mine, &reserved, published)?;
        Refuse_If_Reserved(&mine, &reserved, document)?;
    }

    return Ok(());
}

/// Refuses an identifier some committed record already carries.
fn Refuse_If_Published(
    mine: &Territory,
    reserved: &str,
    published: &Territory,
) -> Result<(), AddRefusal>
{
    for file in &published.paths
    {
        let theirs = Territory::Of_Files([file.clone()]);

        if matches!(mine.Intersect(&theirs), Intersection::Overlaps(_))
        {
            return Err(AddRefusal::RecordPublished {
                identifier: Normalize_Path(reserved),
                file: file.clone(),
            });
        }
    }

    return Ok(());
}

/// Refuses an identifier another open item is already holding.
///
/// Only open items reserve. A closed item's territory is history, and refusing against it
/// would make every finished item a permanent claim on its number — which would refuse the
/// whole board, since almost every item ever written reserved a record.
fn Refuse_If_Reserved(
    mine: &Territory,
    reserved: &str,
    document: &LedgerDocument,
) -> Result<(), AddRefusal>
{
    for other in &document.items
    {
        if !matches!(other.state, ItemState::Ready | ItemState::Claimed)
        {
            continue;
        }

        if matches!(mine.Intersect(&other.territory), Intersection::Overlaps(_))
        {
            return Err(AddRefusal::RecordReserved {
                identifier: Normalize_Path(reserved),
                item: other.id.clone(),
            });
        }
    }

    return Ok(());
}

/// The authored paths in a territory that name a record identifier rather than a file.
///
/// Scoped deliberately. `add` does not refuse overlapping territory in general and must not
/// start: items overlap constantly and claims are what serialize them. What is being guarded
/// is the one reservation an author cannot recover from mid-claim, because there is no
/// `work edit` to move a record identifier once somebody else has published it.
///
/// Recognised through [`Normalize_Path`] rather than by re-reading the grammar here. A path
/// names an identifier when folding lands it directly inside the record directory: both
/// `docs/records/OD-LEDGER-025` and `docs/records/OD-LEDGER-025-a-slug.md` fold to
/// `docs/records/od-ledger-025`, and anything nested deeper is some other thing that happens
/// to live there. The directory itself is not an identifier — reserving all of it is a
/// different problem, and `P10-RECORD-LOCK` is where it was answered.
fn Record_Reservations(territory: &Territory) -> Vec<String>
{
    return territory
        .paths
        .iter()
        .filter(|path| {
            let folded = Normalize_Path(path);
            let Some(within) = folded.strip_prefix(RECORD_DIRECTORY_PREFIX)
            else
            {
                return false;
            };

            return !within.is_empty() && !within.contains('/');
        })
        .cloned()
        .collect();
}

/// The folded record directory, with its separator, as [`Normalize_Path`] leaves it.
const RECORD_DIRECTORY_PREFIX: &str = "docs/records/";

/// The document as it goes to disk, stamped with the schema version this build writes.
///
/// Stamped here rather than taken from the document read in: a file that keeps whatever
/// version it arrived with is a file a build without a field can rewrite while still
/// claiming to speak the newer schema.
fn Rendered(document: &LedgerDocument) -> Result<String, LedgerError>
{
    let stamped = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: document.items.clone(),
    };

    let mut rendered =
        serde_json::to_string_pretty(&stamped).map_err(|error| LedgerError::Unreadable {
            cause: error.to_string(),
        })?;
    rendered.push('\n');

    return Ok(rendered);
}

/// Marks an item claimed and records the grant.
fn Install_Claim(document: &mut LedgerDocument, item: &ItemId, granted: &Claim)
{
    for candidate in &mut document.items
    {
        if &candidate.id == item
        {
            candidate.state = ItemState::Claimed;
            candidate.claim = Some(granted.clone());
        }
    }
}

/// Changes an item's own claim, once the holder has been shown entitled to change it.
///
/// `Renew` and `Release` differ only in what they do to a claim they may act on, so the
/// entitlement question is answered in one place. Written out twice, the two were one edit
/// away from disagreeing about who may act.
fn With_Own_Claim(
    document: &mut LedgerDocument,
    item: &ItemId,
    holder: &str,
    act: impl FnOnce(&mut LedgerItem),
) -> Result<(), ClaimRefusal>
{
    let found = document.items.iter_mut().find(|candidate| return &candidate.id == item);
    let Some(candidate) = found
    else
    {
        return Err(ClaimRefusal::NoSuchItem { item: item.clone() });
    };
    Entitled(candidate, holder)?;
    act(candidate);

    return Ok(());
}

/// Whether `holder` may change this item's claim.
fn Entitled(candidate: &LedgerItem, holder: &str) -> Result<(), ClaimRefusal>
{
    let Some(claim) = &candidate.claim
    else
    {
        return Err(ClaimRefusal::NotClaimable {
            item: candidate.id.clone(),
            state: "unclaimed".to_owned(),
        });
    };

    if claim.holder != holder
    {
        return Err(ClaimRefusal::HeldBy {
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
            item: candidate.id.clone(),
        });
    }

    return Ok(());
}

/// Moves a lapsed claim aside and installs the replacement, as one operation.
///
/// `Replace_Lapsed_Claim` and not two statements at the call site: the move of the old claim
/// and the install of the new one are one operation precisely so that no caller can perform
/// half of it.
fn Replace_Lapsed(
    document: &mut LedgerDocument,
    item: &ItemId,
    replacement: Claim,
    now: Timestamp,
) -> Result<(), ClaimRefusal>
{
    let mut taken = false;
    for candidate in &mut document.items
    {
        if &candidate.id == item
        {
            taken = candidate.Replace_Lapsed_Claim(replacement.clone(), now);
        }
    }

    if !taken
    {
        return Err(ClaimRefusal::NoSuchItem { item: item.clone() });
    }

    return Ok(());
}

/// Every way a ledger can be internally inconsistent.
///
/// Returns all violations rather than the first. An author fixing one at a time and
/// re-running is an author who stops running it.
#[must_use]
pub fn Validate(document: &LedgerDocument, now: Timestamp) -> Vec<String>
{
    let mut violations = Duplicate_Identifiers(document);
    let declared: Vec<&ItemId> = document.items.iter().map(|item| return &item.id).collect();

    for item in &document.items
    {
        Check_Dependencies(item, &declared, &mut violations);
        Check_State(item, &mut violations);
        Check_Predicate(item, &mut violations);
        Check_Territory(item, &mut violations);
    }

    let overlapping = Overlapping_Claims(document, now);
    violations.extend(overlapping);

    return violations;
}

/// Identifiers that appear more than once, which makes every lookup ambiguous.
fn Duplicate_Identifiers(document: &LedgerDocument) -> Vec<String>
{
    let mut violations = Vec::new();
    let mut seen: Vec<&ItemId> = Vec::new();

    for item in &document.items
    {
        if seen.contains(&&item.id)
        {
            violations.push(format!("{} appears more than once", item.id));
        }
        seen.push(&item.id);
    }

    return violations;
}

/// Dependencies naming an item the ledger does not hold.
fn Check_Dependencies(item: &LedgerItem, declared: &[&ItemId], violations: &mut Vec<String>)
{
    for dependency in &item.depends_on
    {
        if !declared.contains(&dependency)
        {
            violations.push(format!(
                "{} depends on {dependency}, which is not in the ledger",
                item.id
            ));
        }
    }
}

/// What a state must be able to say about itself.
///
/// The claimed case is deliberately `claim.is_none()` and not `!Has_Active_Claim(now)`.
///
/// A lease expiring is `Claimed` with an inactive claim, and it is the normal end of an
/// agent that died rather than a corruption. Validating against the clock made a document's
/// validity a function of when it was read: one written valid stopped being valid on its
/// own, `Load` refuses an invalid document, and every operation loads first — so one lapsed
/// lease refused every claim on the board, including items sharing no territory with it.
/// `MAXIMUM_LEASE` exists to stop a crashed agent holding territory until somebody edits the
/// file, and the lease expiring was causing exactly what the lease exists to prevent.
///
/// What remains is the invariant that does not move: an item claimed by nobody records who
/// claimed it. That is a real corruption — nothing can say whose work was abandoned — and it
/// cannot arrive by the passage of time.
fn Check_State(item: &LedgerItem, violations: &mut Vec<String>)
{
    if item.state == ItemState::Blocked && item.blocked.is_none()
    {
        violations.push(format!("{} is blocked without saying why", item.id));
    }
    if item.state == ItemState::Claimed && item.claim.is_none()
    {
        violations.push(format!(
            "{} is marked claimed and records no claim, so nothing can say who holds it or \
             held it",
            item.id
        ));
    }
    if item.state == ItemState::Done && item.verified.is_none()
    {
        violations.push(format!(
            "{} is done with no recorded verification; done_when is prose, and prose is not \
             a predicate",
            item.id
        ));
    }

}

/// A predicate nothing could run, which makes `done_when` unenforceable.
fn Check_Predicate(item: &LedgerItem, violations: &mut Vec<String>)
{
    if let Some(predicate) = &item.verification
        && !predicate.Is_Runnable()
    {
        violations.push(format!(
            "{} carries a verification predicate that cannot be run",
            item.id
        ));
    }
}

/// What a territory must be able to exclude.
///
/// An item somebody can pick up must say what it touches. An empty territory is disjoint
/// from every other territory, so two agents working an unstated item are told they may both
/// proceed — the ledger answers the exclusion question confidently and wrongly. Silence
/// about territory is not a claim of touching nothing.
fn Check_Territory(item: &LedgerItem, violations: &mut Vec<String>)
{
    for (first, second) in item.territory.Ambiguous_Paths()
    {
        violations.push(format!(
            "{}'s territory lists `{first}` and `{second}`, which name the same subject; \
             whoever wrote it probably believed they were reserving two things",
            item.id
        ));
    }

    if matches!(item.state, ItemState::Ready | ItemState::Claimed) && item.territory.Is_Empty()
    {
        violations.push(format!(
            "{} is workable but reserves nothing, so it excludes nobody",
            item.id
        ));
    }
}

/// Every pair of concurrently-claimed items whose territories are not provably disjoint.
///
/// This is the invariant the whole ledger exists to hold. Two active claims on
/// overlapping territory means two agents editing the same files, and the first one to
/// write wins silently.
fn Overlapping_Claims(document: &LedgerDocument, now: Timestamp) -> Vec<String>
{
    let active: Vec<&LedgerItem> = document
        .items
        .iter()
        .filter(|item| item.Has_Active_Claim(now))
        .collect();

    let mut violations = Vec::new();

    for (index, item) in active.iter().enumerate()
    {
        for other in active.iter().skip(index.saturating_add(1))
        {
            let contested = Not_Provably_Disjoint(item, other);
            violations.extend(contested);
        }
    }

    return violations;
}

/// What one pair of active claims has to answer for, if anything.
///
/// `Unknown` is reported alongside `Overlaps` rather than passed over. The ledger's promise
/// is that two holders were shown independent, and a comparison that could not decide has
/// not shown it.
fn Not_Provably_Disjoint(item: &LedgerItem, other: &LedgerItem) -> Option<String>
{
    let holder = Holder_Of(item);
    let other_holder = Holder_Of(other);

    return match item.territory.Intersect(&other.territory)
    {
        Intersection::Disjoint => None,
        Intersection::Overlaps(shared) => Some(format!(
            "{} (held by {holder}) and {} (held by {other_holder}) both claim {} overlapping \
             subject(s)",
            item.id,
            other.id,
            shared.len()
        )),
        Intersection::Unknown(reason) => Some(format!(
            "{} (held by {holder}) and {} (held by {other_holder}) cannot be shown \
             independent: {}",
            item.id,
            other.id,
            reason.Describe()
        )),
    };
}

/// Who holds a claim, for a message that has to name somebody either way.
fn Holder_Of(item: &LedgerItem) -> &str
{
    return item
        .claim
        .as_ref()
        .map_or("someone", |claim| return claim.holder.as_str());
}

/// What would refuse a claim on `item` as of `now`, if anything.
///
/// # Why this is a function rather than a check inside `Claim`
///
/// Two callers need this answer and they must not compute it twice. [`ExclusionLedger::Claim`]
/// asks it to decide whether to grant; a listing asks it to decide what to *call* an item.
/// When those were separate, `work list` read the `state` field alone and printed `ready`
/// for items nothing could take — on 2026-08-09 it said `ready` for eight items while a
/// single held claim refused all eight. An agent picking work off that column burns a round
/// trip per item and learns to distrust the column.
///
/// The fix is not a second guard. Two implementations of one rule is how they come to
/// disagree, and a listing that disagreed with claiming would be worse than one that says
/// too little. So this is the only implementation, and `Claim` is one of its callers.
///
/// The order of the checks is the order a claim refuses in, so the reason reported is the
/// first reason a claimant would actually hit.
#[must_use]
pub fn Claim_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };
    // The lapse check comes before the state check, because `Claimed` is the state a lapsed
    // item is in and reporting it as merely "not claimable" is what left the operator with
    // no next step: true of a `Done` item as well, and the two have opposite remedies.
    // `OD-LEDGER-012`.
    if let Some(refusal) = Lapse_Refusal(target, now)
    {
        return Some(refusal);
    }
    if !target.state.Is_Claimable()
    {
        return Some(ClaimRefusal::NotClaimable {
            item: item.clone(),
            state: target.state.Describe(),
        });
    }

    return Contested_By(document, target, now);
}

/// The refusal a lapsed item earns, if it is one.
///
/// One implementation, two callers, for the reason [`Claim_Refusal`] itself is a function:
/// this predicate now decides both what `claim` refuses with *and* whether
/// [`FileLedger::Take_Over`] will succeed, and a second copy of it would eventually let a
/// listing say `lapsed` about an item the takeover then declined.
///
/// `now` comes from the caller for the same reason every other judgment here takes it: a
/// second clock read would decide half of one answer against a different instant.
fn Lapse_Refusal(target: &LedgerItem, now: Timestamp) -> Option<ClaimRefusal>
{
    if target.state != ItemState::Claimed
    {
        return None;
    }

    let claim = target.claim.as_ref()?;
    if !claim.Has_Lapsed(now)
    {
        return None;
    }

    return Some(ClaimRefusal::Lapsed {
        item: target.id.clone(),
        holder: claim.holder.clone(),
        since: claim.lease_expires_at,
    });
}

/// Why an item may not be declined, if it may not.
///
/// Only a `Ready` item can be ended, and the three refusals below are the three ways of not
/// being one. `OD-LEDGER-019` decision 4 states each; what matters here is the *order*, which
/// is the order a caller hits them so that the reason reported is the first one that actually
/// applies — the same discipline [`Claim_Refusal`] follows, and the lapse check is first for
/// the same reason it is first there.
///
/// [`Contested_By`] is deliberately not consulted. Territory contention decides who may *work*
/// an item; it has nothing to say about whether the item is work at all, and a decline refused
/// because some unrelated peer holds an overlapping file would be a refusal nobody could act
/// on.
fn Decline_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };
    // The lapse check is first, because `Claimed` is the state a lapsed item is in and the
    // remedies differ: a live holder is asked to release, and a dead one is taken over.
    // `OD-LEDGER-012`.
    if let Some(refusal) = Lapse_Refusal(target, now)
    {
        return Some(refusal);
    }

    return Held_Or_Unready(target);
}

/// What a live claim or a state other than `Ready` does to a decline.
///
/// `Describe` rather than a bare state word, so a `Declined` item's refusal carries the
/// reason it already holds. Told only "it is declined", a caller cannot tell a duplicate
/// from a disagreement, and both need a person.
fn Held_Or_Unready(target: &LedgerItem) -> Option<ClaimRefusal>
{
    if let Some(claim) = target.claim.as_ref()
    {
        return Some(ClaimRefusal::StillHeld {
            item: target.id.clone(),
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
        });
    }
    if target.state != ItemState::Ready
    {
        return Some(ClaimRefusal::NotClaimable {
            item: target.id.clone(),
            state: target.state.Describe(),
        });
    }

    return None;
}

/// Everything that refuses an item for a reason outside the item's own state: an unfinished
/// dependency, or territory somebody else is actively holding.
///
/// Lifted out of [`Claim_Refusal`] unchanged so that [`FileLedger::Take_Over`] re-establishes
/// independence by the same code rather than by a second copy of it. A takeover that skipped
/// this would grant overlapping ground, and the case is not hypothetical: a lapsed claim stops
/// excluding, so another item may since have been claimed over exactly the files this one
/// reserves.
///
/// Private. Both entry points are public and the rule is not a third one.
fn Contested_By(
    document: &LedgerDocument,
    target: &LedgerItem,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    if let Some(refusal) = Unmet_Dependency(document, target)
    {
        return Some(refusal);
    }

    return Held_Ground(document, target, now);
}

/// The first dependency of `target` that is not done, as a refusal.
fn Unmet_Dependency(document: &LedgerDocument, target: &LedgerItem) -> Option<ClaimRefusal>
{
    let done = format!("{:?}", ItemState::Done);

    for dependency in &target.depends_on
    {
        let found = document.items.iter().find(|candidate| return &candidate.id == dependency);
        let state = found.map_or_else(
            || return "not in the ledger".to_owned(),
            |found| return format!("{:?}", found.state),
        );

        if state != done
        {
            return Some(ClaimRefusal::DependencyUnmet {
                item: target.id.clone(),
                dependency: dependency.clone(),
                state,
            });
        }
    }

    return None;
}

/// The first active claim whose territory `target` cannot be shown independent of.
fn Held_Ground(
    document: &LedgerDocument,
    target: &LedgerItem,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    for other in &document.items
    {
        if other.id == target.id || !other.Has_Active_Claim(now)
        {
            continue;
        }

        let refusal = Refused_By(target, other);
        if refusal.is_some()
        {
            return refusal;
        }
    }

    return None;
}

/// What `other`'s live claim does to `target`, if anything.
fn Refused_By(target: &LedgerItem, other: &LedgerItem) -> Option<ClaimRefusal>
{
    let claim = other.claim.as_ref()?;
    let overlap = target.territory.Intersect(&other.territory);

    return Refusal_From(&overlap, &other.id, &claim.holder, claim.lease_expires_at);
}

/// What refuses a takeover of `item` as of `now`, if anything.
///
/// The mirror of [`Claim_Refusal`], differing in exactly one clause: a claim needs the item to
/// be free and this needs it to be lapsed. Everything after that question is the same code,
/// which is the point — [`Contested_By`] is called and not copied, so a takeover cannot come to
/// disagree with a claim about whether two territories are independent.
fn Takeover_Refusal(
    document: &LedgerDocument,
    item: &ItemId,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    let Some(target) = document
        .items
        .iter()
        .find(|candidate| &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };

    // A takeover answers a lapse and nothing else. An item with a live claim is a queue, and
    // everything else is the caller reaching for the wrong verb.
    if Lapse_Refusal(target, now).is_none()
    {
        return Some(Wrong_Verb(target, now));
    }

    return Contested_By(document, target, now);
}

/// What to say to a caller that used `takeover` on an item that has not lapsed.
///
/// A live claim is [`ClaimRefusal::HeldBy`] — retryable, because the lease running out is what
/// resolves it, and telling an agent to wait is the honest answer when somebody is working.
/// Everything else is the state word, so that `takeover` against a `Ready` or `Done` item reads
/// as the wrong verb rather than as a queue that will never clear.
fn Wrong_Verb(target: &LedgerItem, now: Timestamp) -> ClaimRefusal
{
    if let Some(claim) = &target.claim
        && !claim.Has_Lapsed(now)
    {
        return ClaimRefusal::HeldBy {
            holder: claim.holder.clone(),
            until: claim.lease_expires_at,
            item: target.id.clone(),
        };
    }

    return ClaimRefusal::NotClaimable {
        item: target.id.clone(),
        state: target.state.Describe(),
    };
}

/// A store failure, reported as itself rather than as a missing item.
///
/// Every load and save in the three operations below used to discard its error and return
/// [`ClaimRefusal::NoSuchItem`], so an unreadable file, a parse error and an invalid
/// document all told the operator that their identifier matched nothing. That is a reason
/// destroyed by the failure it explains, and it is why `P10-LAPSE-BRICKS` needed an
/// experiment to diagnose rather than a glance: the surface was reporting a spelling
/// mistake while the ledger was refusing to load.
///
/// A conversion rather than a free function since [`FileLedger::Decide_Under_Lock`] became
/// generic over what its caller refuses with: this is the one thing every such vocabulary
/// has to be able to say, so it is the bound rather than a step each verb performs.
impl From<&LedgerError> for ClaimRefusal
{
    fn from(error: &LedgerError) -> Self
    {
        return Self::LedgerUnusable {
            cause: error.to_string(),
        };
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
        return self.Decide_Under_Lock(holder, |document, now| {
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
        return self.Decide_Under_Lock(holder, |document, now| {
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
        return self.Decide_Under_Lock(holder, |document, now| {
            // Both arms, written once, in `ReleaseOutcome::Record_On`. Spelling them out at
            // the call site is what let this store keep the finished arm's evidence and drop
            // the abandoned arm's.
            return With_Own_Claim(document, item, holder, |candidate| {
                outcome.Record_On(candidate, holder, now);
            });
        });
    }

}
