//! The durable ledger: a JSON file, a lock beside it, and the rules it must satisfy.

use crate::exclusion::{
    Check_Lease, ClaimRefusal, ExclusionLedger, Refusal_From, ReleaseOutcome, Reservation,
};
use crate::item::{Claim, ItemId, ItemState, LedgerItem};
use crate::territory::Territory;
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

/// The schema version written into every ledger file.
const SCHEMA_VERSION: u32 = 1;

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
        };
    }
}

impl std::error::Error for LedgerError {}

/// The on-disk form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerDocument
{
    /// Schema version, so a future reader can tell what it is looking at.
    pub schema_version: u32,
    /// The items, in a stable order.
    pub items: Vec<LedgerItem>,
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
    /// # Errors
    ///
    /// Returns [`LedgerError::Malformed`] when the file exists and cannot be parsed,
    /// and [`LedgerError::Unreadable`] when it cannot be read at all.
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

        return serde_json::from_str(&text).map_err(|error| LedgerError::Malformed {
            cause: format!("{}: {error}", self.path.display()),
        });
    }

    /// Writes the ledger, refusing to persist one that violates its own invariants.
    ///
    /// Validation happens *before* the write, not after. A ledger that has already been
    /// written and then found invalid is a ledger somebody has to repair by hand, and
    /// in the meantime every agent reading it is reading something the system itself
    /// says is wrong.
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

        let mut rendered =
            serde_json::to_string_pretty(document).map_err(|error| LedgerError::Unreadable {
                cause: error.to_string(),
            })?;
        rendered.push('\n');

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

        let mut document = self.Load()?;
        let outcome = modify(&mut document)?;
        self.Save(&document)?;

        // The takeover travels out with the result rather than being logged here. A
        // caller that surfaces it can tell the user their predecessor abandoned an
        // update; a caller that drops it has made a choice, and this signature is what
        // makes that choice visible in review.
        return Ok((outcome, acquisition.broke_stale));
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

/// Every way a ledger can be internally inconsistent.
///
/// Returns all violations rather than the first. An author fixing one at a time and
/// re-running is an author who stops running it.
#[must_use]
pub fn Validate(document: &LedgerDocument, now: Timestamp) -> Vec<String>
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

    for item in &document.items
    {
        for dependency in &item.depends_on
        {
            if !seen.contains(&dependency)
            {
                violations.push(format!(
                    "{} depends on {dependency}, which is not in the ledger",
                    item.id
                ));
            }
        }

        if item.state == ItemState::Blocked && item.blocked.is_none()
        {
            violations.push(format!("{} is blocked without saying why", item.id));
        }

        if item.state == ItemState::Claimed && !item.Has_Active_Claim(now)
        {
            violations.push(format!(
                "{} is marked claimed but has no active claim",
                item.id
            ));
        }

        if item.state == ItemState::Done && item.verified.is_none()
        {
            violations.push(format!(
                "{} is done with no recorded verification; done_when is prose, and prose \
                 is not a predicate",
                item.id
            ));
        }

        if let Some(predicate) = &item.verification
            && !predicate.Is_Runnable()
        {
            violations.push(format!(
                "{} carries a verification predicate that cannot be run",
                item.id
            ));
        }

        for (first, second) in item.territory.Ambiguous_Paths()
        {
            violations.push(format!(
                "{}'s territory lists `{first}` and `{second}`, which name the same \
                 subject; whoever wrote it probably believed they were reserving two things",
                item.id
            ));
        }

        // An item somebody can pick up must say what it touches. An empty territory is
        // disjoint from every other territory, so two agents working an unstated item
        // are told they may both proceed — the ledger answers the exclusion question
        // confidently and wrongly. Silence about territory is not a claim of touching
        // nothing.
        if matches!(item.state, ItemState::Ready | ItemState::Claimed)
            && item.territory.Is_Empty()
        {
            violations.push(format!(
                "{} is workable but reserves nothing, so it excludes nobody",
                item.id
            ));
        }
    }

    violations.extend(Overlapping_Claims(document, now));

    return violations;
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
            let holder = item
                .claim
                .as_ref()
                .map_or("someone", |claim| claim.holder.as_str());
            let other_holder = other
                .claim
                .as_ref()
                .map_or("someone", |claim| claim.holder.as_str());

            match item.territory.Intersect(&other.territory)
            {
                nomos_model::Intersection::Disjoint =>
                {}
                nomos_model::Intersection::Overlaps(shared) => violations.push(format!(
                    "{} (held by {holder}) and {} (held by {other_holder}) both claim {} \
                     overlapping subject(s)",
                    item.id,
                    other.id,
                    shared.len()
                )),
                nomos_model::Intersection::Unknown(reason) => violations.push(format!(
                    "{} (held by {holder}) and {} (held by {other_holder}) cannot be shown \
                     independent: {}",
                    item.id,
                    other.id,
                    reason.Describe()
                )),
            }
        }
    }

    return violations;
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
    let Some(target) = document
        .items
        .iter()
        .find(|candidate| &candidate.id == item)
    else
    {
        return Some(ClaimRefusal::NoSuchItem { item: item.clone() });
    };

    if !target.state.Is_Claimable()
    {
        return Some(ClaimRefusal::NotClaimable {
            item: item.clone(),
            state: format!("{:?}", target.state),
        });
    }

    for dependency in &target.depends_on
    {
        let state = document
            .items
            .iter()
            .find(|candidate| &candidate.id == dependency)
            .map_or_else(|| "not in the ledger".to_owned(), |found| {
                format!("{:?}", found.state)
            });

        if state != format!("{:?}", ItemState::Done)
        {
            return Some(ClaimRefusal::DependencyUnmet {
                item: item.clone(),
                dependency: dependency.clone(),
                state,
            });
        }
    }

    for other in &document.items
    {
        if &other.id == item || !other.Has_Active_Claim(now)
        {
            continue;
        }

        let Some(claim) = &other.claim
        else
        {
            continue;
        };

        if let Some(refusal) = Refusal_From(
            &target.territory.Intersect(&other.territory),
            &other.id,
            &claim.holder,
            claim.lease_expires_at,
        )
        {
            return Some(refusal);
        }
    }

    return None;
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

        let now = self.clock.Now();
        let expires_at = now.Plus(lease);
        let mut document = self.Load().map_err(|_| ClaimRefusal::NoSuchItem {
            item: item.clone(),
        })?;

        if let Some(refusal) = Claim_Refusal(&document, item, now)
        {
            return Err(refusal);
        }

        for candidate in &mut document.items
        {
            if &candidate.id == item
            {
                candidate.state = ItemState::Claimed;
                candidate.claim = Some(Claim {
                    holder: holder.to_owned(),
                    acquired_at: now,
                    lease_expires_at: expires_at,
                });
            }
        }

        self.Save(&document)
            .map_err(|_| ClaimRefusal::NoSuchItem { item: item.clone() })?;

        return Ok(Reservation {
            item: item.clone(),
            holder: holder.to_owned(),
            expires_at,
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

        let now = self.clock.Now();
        let expires_at = now.Plus(lease);
        let mut document = self.Load().map_err(|_| ClaimRefusal::NoSuchItem {
            item: item.clone(),
        })?;

        let mut renewed = false;
        for candidate in &mut document.items
        {
            if &candidate.id != item
            {
                continue;
            }
            match &mut candidate.claim
            {
                Some(claim) if claim.holder == holder =>
                {
                    claim.lease_expires_at = expires_at;
                    renewed = true;
                }
                Some(claim) =>
                {
                    return Err(ClaimRefusal::HeldBy {
                        holder: claim.holder.clone(),
                        until: claim.lease_expires_at,
                        item: item.clone(),
                    });
                }
                None =>
                {
                    return Err(ClaimRefusal::NotClaimable {
                        item: item.clone(),
                        state: "unclaimed".to_owned(),
                    });
                }
            }
        }

        if !renewed
        {
            return Err(ClaimRefusal::NoSuchItem { item: item.clone() });
        }

        self.Save(&document)
            .map_err(|_| ClaimRefusal::NoSuchItem { item: item.clone() })?;

        return Ok(Reservation {
            item: item.clone(),
            holder: holder.to_owned(),
            expires_at,
        });
    }

    fn Release(
        &mut self,
        item: &ItemId,
        holder: &str,
        outcome: ReleaseOutcome,
    ) -> Result<(), ClaimRefusal>
    {
        let mut document = self.Load().map_err(|_| ClaimRefusal::NoSuchItem {
            item: item.clone(),
        })?;

        let mut released = false;
        for candidate in &mut document.items
        {
            if &candidate.id != item
            {
                continue;
            }
            match &candidate.claim
            {
                Some(claim) if claim.holder == holder =>
                {
                    candidate.claim = None;
                    match &outcome
                    {
                        ReleaseOutcome::Finished(record) =>
                        {
                            candidate.state = ItemState::Done;
                            candidate.verified = Some(record.clone());
                        }
                        ReleaseOutcome::Abandoned { .. } => candidate.state = ItemState::Ready,
                    }
                    released = true;
                }
                Some(claim) =>
                {
                    return Err(ClaimRefusal::HeldBy {
                        holder: claim.holder.clone(),
                        until: claim.lease_expires_at,
                        item: item.clone(),
                    });
                }
                None =>
                {
                    return Err(ClaimRefusal::NotClaimable {
                        item: item.clone(),
                        state: "unclaimed".to_owned(),
                    });
                }
            }
        }

        if !released
        {
            return Err(ClaimRefusal::NoSuchItem { item: item.clone() });
        }

        return self
            .Save(&document)
            .map_err(|_| ClaimRefusal::NoSuchItem { item: item.clone() });
    }

    fn Conflicts(&self, territory: &Territory) -> Vec<ClaimRefusal>
    {
        let now = self.clock.Now();
        let Ok(document) = self.Load()
        else
        {
            return Vec::new();
        };

        return document
            .items
            .iter()
            .filter(|item| item.Has_Active_Claim(now))
            .filter_map(|item| {
                let claim = item.claim.as_ref()?;
                Refusal_From(
                    &territory.Intersect(&item.territory),
                    &item.id,
                    &claim.holder,
                    claim.lease_expires_at,
                )
            })
            .collect();
    }
}
