//! The bodies of the three verbs [`ExclusionLedger`] declares, and of nothing else.
//!
//! Each is the body of a method on [`FileLedger`] and not the method. The method keeps its
//! signature, and the trait keeps its documentation, in `store.rs`; the reason is the one
//! `verbs.rs` gives -- the surface snapshot resolves `pub use store::FileLedger` against one
//! module, so a `pub fn` written on the type anywhere else is public and unrecorded.
//!
//! They are gathered here rather than beside the file's other bodies because they answer one
//! question between them: whether a holder may have an item, and for how long. A reader asking
//! that question should not have to read the parsing, the rendering and the schema to find the
//! three functions that answer it.

use std::time::Duration;

use nomos_platform::{Clock, CrossProcessLock, FileSystem, Timestamp};

use crate::ClaimRefusal;
use crate::ItemId;
use crate::LedgerDocument;
use crate::ReleaseOutcome;
use crate::Reservation;
use crate::exclusion::Check_Lease;

use super::claiming::{Install_Claim, With_Own_Claim};
use super::file::Decide_Under_Lock;
use super::FileLedger;

/// The body of [`ExclusionLedger::Claim`], which keeps the documentation and the signature.
pub(super) fn Claim_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    holder: &str,
    lease: Duration,
) -> Result<Reservation, ClaimRefusal>
{
    Check_Lease(lease)?;

    let request = ClaimRequest { item, holder, lease };

    // The refusal is decided and the grant is written under one acquisition. Deciding
    // outside it is not a narrower window, it is the same defect: what the check reads
    // and what the write is based on are the same snapshot, and another session can
    // replace the file between them.
    return Decide_Under_Lock(ledger, holder, |document, now| {
        return Grant_Claim(document, &request, now);
    });
}

/// What [`Claim_Item`] asks for, grouped so the function that acts on it under the lock stays
/// under this crate's own parameter-count ceiling -- the shape `TakeoverRequest` in `verbs.rs`
/// already has, for the same reason.
struct ClaimRequest<'a>
{
    item: &'a ItemId,
    holder: &'a str,
    lease: Duration,
}

/// [`Claim_Item`]'s body once the lock is held and the instant it was taken at is known: refuses
/// the claim if an exclusion still stands, and otherwise writes the grant and answers with it.
fn Grant_Claim(
    document: &mut LedgerDocument,
    request: &ClaimRequest<'_>,
    now: Timestamp,
) -> Result<Reservation, ClaimRefusal>
{
    use crate::Claim;
    use super::refusal::Claim_Refusal;

    let expires_at = now.Plus(request.lease);
    if let Some(refusal) = Claim_Refusal(document, request.item, now)
    {
        return Err(refusal);
    }

    let granted = Claim {
        holder: request.holder.to_owned(),
        acquired_at: now,
        lease_expires_at: expires_at,
    };
    Install_Claim(document, request.item, &granted);

    return Ok(Reservation {
        item: request.item.clone(),
        holder: request.holder.to_owned(),
        expires_at,
    });
}

/// The body of [`ExclusionLedger::Renew`], which keeps the documentation and the signature.
pub(super) fn Renew_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    holder: &str,
    lease: Duration,
) -> Result<Reservation, ClaimRefusal>
{
    Check_Lease(lease)?;

    // A lost renewal does not look like a lost write. It looks like a lease that ran
    // out early, which reads as an agent that died — so this verb being outside the
    // lock sent whoever noticed to investigate the wrong thing.
    return Decide_Under_Lock(ledger, holder, |document, now| {
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

/// The body of [`ExclusionLedger::Release`], which keeps the documentation and the signature.
pub(super) fn Release_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
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
    return Decide_Under_Lock(ledger, holder, |document, now| {
        // Both arms, written once, in `ReleaseOutcome::Record_On`. Spelling them out at
        // the call site is what let this store keep the finished arm's evidence and drop
        // the abandoned arm's.
        return With_Own_Claim(document, item, holder, |candidate| {
            outcome.Record_On(candidate, holder, now);
        });
    });
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence, Timestamp};
    use nomos_platform_std::{FileLock, StdFileSystem};
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::{ItemKind, ItemOrigin, ItemState, LedgerDocument, LedgerItem, Territory};

    /// The instant every fixture here is read at.
    const NOW_SECONDS: i64 = 1_000;

    /// The lease a claim asks for, and the longer one a renewal asks for.
    const ONE_HOUR: Duration = Duration::from_secs(3_600);
    const TWO_HOURS: Duration = Duration::from_secs(7_200);

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

    #[test]
    fn Test_Claim_Item_Should_Grant_The_Claim_To_Whom_It_Was_Asked_For()
    {
        let directory = Temporary_Directory("claim-item");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Unclaimed_Board(&directory, &clock, "C-1");

        let reservation = Claim_Item(&mut ledger, &ItemId::New("C-1"), "agent-a", ONE_HOUR)
            .expect("an unclaimed item over free ground must be claimable");

        assert_eq!(reservation.holder, "agent-a");
        assert_eq!(reservation.item, ItemId::New("C-1"));
        assert_eq!(
            reservation.expires_at,
            Timestamp::From_Unix_Seconds(NOW_SECONDS).Plus(ONE_HOUR),
            "the lease must be measured from the instant the lock was taken"
        );
        let held = Reloaded_Item(&ledger, "C-1");
        assert_eq!(held.state, ItemState::Claimed);
        let claim = held.claim.expect("a granted claim must have been written, not only returned");
        assert_eq!(claim.holder, "agent-a");
        assert_eq!(claim.lease_expires_at, reservation.expires_at);
    }

    #[test]
    fn Test_Renew_Item_Should_Extend_The_Holders_Own_Lease_And_Not_Restart_It()
    {
        let directory = Temporary_Directory("renew-item");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Unclaimed_Board(&directory, &clock, "R-1");
        Claim_Item(&mut ledger, &ItemId::New("R-1"), "agent-a", ONE_HOUR).expect("the fixture claim must land");

        let renewed = Renew_Item(&mut ledger, &ItemId::New("R-1"), "agent-a", TWO_HOURS)
            .expect("a holder may renew its own live claim");

        assert_eq!(renewed.holder, "agent-a");
        assert_eq!(renewed.expires_at, Timestamp::From_Unix_Seconds(NOW_SECONDS).Plus(TWO_HOURS));
        let claim = Reloaded_Item(&ledger, "R-1").claim.expect("a renewed claim is still recorded");
        assert_eq!(claim.lease_expires_at, renewed.expires_at, "the renewal must reach the file");
        assert_eq!(
            claim.acquired_at,
            Timestamp::From_Unix_Seconds(NOW_SECONDS),
            "a renewal extends the lease; it does not begin a new claim"
        );
    }

    #[test]
    fn Test_Release_Item_Should_Record_The_Outcome_And_Clear_The_Claim()
    {
        let directory = Temporary_Directory("release-item");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Unclaimed_Board(&directory, &clock, "F-1");
        Claim_Item(&mut ledger, &ItemId::New("F-1"), "agent-a", ONE_HOUR).expect("the fixture claim must land");
        let outcome = ReleaseOutcome::Abandoned { reason: "wrong approach".to_owned() };

        Release_Item(&mut ledger, &ItemId::New("F-1"), "agent-a", outcome)
            .expect("a holder may release its own claim");

        let released = Reloaded_Item(&ledger, "F-1");
        assert_eq!(released.state, ItemState::Ready, "an abandoned item must be claimable again");
        assert_eq!(released.claim, None, "an ended claim must stop excluding");
        let abandonment = released.abandoned.first().expect("the reason must have been written down");
        assert_eq!(abandonment.holder, "agent-a");
        assert_eq!(abandonment.reason, "wrong approach");
    }

    /// A ledger holding one unclaimed, workable item, and a lock file beside it.
    fn Unclaimed_Board<'clock>(
        directory: &Path,
        clock: &'clock FixedClock,
        id: &str,
    ) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
    {
        let ledger = FileLedger::At(
            directory.join("ledger.json"),
            StdFileSystem,
            clock,
            FileLock::At(directory.join("ledger.lock")),
        );
        let item = Workable_Item(id);
        ledger
            .Save(&LedgerDocument { schema_version: crate::SCHEMA_VERSION, items: vec![item] })
            .expect("an unclaimed workable item is a valid document");
        return ledger;
    }

    /// The item `id` as it stands on disk, which is where a verb's effect has to be visible.
    fn Reloaded_Item(
        ledger: &FileLedger<StdFileSystem, &FixedClock, FileLock>,
        id: &str,
    ) -> LedgerItem
    {
        let document = ledger.Load().expect("the ledger this test just wrote must be readable");
        let wanted = ItemId::New(id);
        return document
            .items
            .into_iter()
            .find(|item| return item.id == wanted)
            .expect("the item the fixture put on the board must still be there");
    }

    fn Temporary_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-store-exclusion-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the stale scratch directory must be removable");
        }
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    /// A `Ready` item with a non-empty territory of its own, so it can sit on a board without
    /// itself violating the "reserves nothing" rule.
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
            widened: Vec::new(),
            declined: None,
        };
    }
}
