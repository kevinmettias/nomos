//! The bodies of the three verbs that change the board, and of the check over the whole of it.
//!
//! Each of these is the body of a method on [`FileLedger`] and not the method. The method
//! keeps its own documentation and its signature in `store.rs`, because that is where a reader
//! of the crate's surface looks and where `tests/contract`'s snapshot reads it from: that
//! reader resolves `pub use store::FileLedger` against one module, so a `pub fn` written on
//! the type anywhere else is public and unrecorded.

use std::time::Duration;

use nomos_platform::{Clock, CrossProcessLock, FileSystem, Timestamp};

use crate::AddRefusal;
use crate::ClaimRefusal;
use crate::DeclineReason;
use crate::Holder;
use crate::LedgerDocument;
use crate::LedgerItem;
use crate::ItemId;
use crate::LedgerError;
use crate::Reservation;

use super::file::Decide_Under_Lock;
use super::refusal::{Decline_Refusal, Enlargement, Takeover_Refusal, Widen_Refusal};
use super::reservation::{RecordDeclaration, Refuse_A_Spent_Record};
use super::FileLedger;

/// The body of [`FileLedger::Validate_Current`], which keeps the documentation and the signature.
pub(super) fn Validate_Current<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
) -> Result<(), LedgerError>
{
    use super::validation::Validate_Document;

    let violations = Validate_Document(&ledger.Load()?, ledger.clock.Now());

    return if violations.is_empty()
    {
        Ok(())
    }
    else
    {
        Err(LedgerError::Invalid { violations })
    };
}

/// The body of [`FileLedger::Add`], which keeps the documentation and the signature.
pub(super) fn Add_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &LedgerItem,
    holder: &str,
    declared: &RecordDeclaration,
) -> Result<(), AddRefusal>
{
    return Decide_Under_Lock(ledger, holder, |document, _now| {
        if document
            .items
            .iter()
            .any(|existing| existing.id == item.id)
        {
            return Err(AddRefusal::AlreadyPresent {
                item: item.id.clone(),
            });
        }

        Refuse_A_Spent_Record(item, document, declared)?;

        document.items.push(item.clone());

        return Ok(());
    });
}

/// The body of [`FileLedger::Widen`], which keeps the documentation and the signature.
///
/// Read, refuse, enlarge, record and write are one mutation inside one acquisition of the
/// cross-process lock, which is what [`Decide_Under_Lock`] is. Calling the same exclusion a
/// claim calls is necessary and is not the property: two widenings running at once can each
/// observe a board on which their own added paths are free, and jointly produce the overlap
/// that check exists to prevent. Sharing the function does nothing about that; deciding and
/// writing without releasing the lock between them does. `OD-LEDGER-015` is what the three
/// verbs before this one cost by deciding outside it.
///
/// Returns the paths actually added, so a caller can report what happened rather than echoing
/// what was asked for. They differ whenever a path was already reserved.
pub(super) fn Widen_Territory<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    holder: Holder<'_>,
    adding: &[String],
) -> Result<Vec<String>, ClaimRefusal>
{
    let requested = Enlargement { item, holder: holder.As_Text(), adding };

    return Decide_Under_Lock(ledger, holder.As_Text(), |document, now| {
        if let Some(refusal) = Widen_Refusal(document, &requested, now)
        {
            return Err(refusal);
        }

        let mut added = Vec::new();
        for candidate in &mut document.items
        {
            if &candidate.id == item
            {
                // `LedgerItem::Widen` and not two statements here, for the reason
                // `Decline` is one call: a call site that extended the territory itself
                // would be free to extend it and not record what it extended it by, and
                // the record is the half `OD-LEDGER-039` exists to keep.
                added = candidate.Widen(adding, holder, now);
            }
        }

        return Ok(added);
    });
}

/// The body of [`FileLedger::Decline`], which keeps the documentation and the signature.
pub(super) fn Decline_Item<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    holder: Holder<'_>,
    reason: DeclineReason<'_>,
) -> Result<(), ClaimRefusal>
{
    return Decide_Under_Lock(ledger, holder.As_Text(), |document, now| {
        if let Some(refusal) = Decline_Refusal(document, item, now)
        {
            return Err(refusal);
        }

        for candidate in &mut document.items
        {
            if &candidate.id == item
            {
                // `LedgerItem::Decline` and not two statements here, for the reason
                // `Try_Replace_Lapsed_Claim` is one call: a call site that wrote the state
                // itself would be free to write it and not the declination, and the
                // declination is the half this verb was added to keep.
                candidate.Decline(reason, holder, now);
            }
        }

        return Ok(());
    });
}

/// The body of [`FileLedger::Take_Over`], which keeps the documentation and the signature.
pub(super) fn Take_Over<Files: FileSystem, TimeSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    holder: &str,
    lease: Duration,
) -> Result<Reservation, ClaimRefusal>
{
    use crate::exclusion::Check_Lease;

    Check_Lease(lease)?;

    let request = TakeoverRequest { item, holder, lease };

    return Decide_Under_Lock(ledger, holder, |document, now| {
        return Take_Over_Locked(document, &request, now);
    });
}

/// What [`Take_Over`] asks for, grouped so the function that acts on it under the lock
/// stays under this crate's own parameter-count ceiling.
struct TakeoverRequest<'a>
{
    item: &'a ItemId,
    holder: &'a str,
    lease: Duration,
}

/// [`Take_Over`]'s body once the lock is held and `now` is known: replaces a lapsed claim
/// with a fresh one, or refuses.
fn Take_Over_Locked(
    document: &mut LedgerDocument,
    request: &TakeoverRequest<'_>,
    now: Timestamp,
) -> Result<Reservation, ClaimRefusal>
{
    use crate::Claim;
    use super::claiming::Replace_Lapsed;

    let expires_at = now.Plus(request.lease);

    if let Some(refusal) = Takeover_Refusal(document, request.item, now)
    {
        return Err(refusal);
    }

    let replacement = Claim {
        holder: request.holder.to_owned(),
        acquired_at: now,
        lease_expires_at: expires_at,
    };
    Replace_Lapsed(document, request.item, &replacement, now)?;

    return Ok(Reservation {
        item: request.item.clone(),
        holder: request.holder.to_owned(),
        expires_at,
    });
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use crate::{Claim, ItemKind, ItemOrigin, ItemState, Territory};
    use nomos_platform_std::{FileLock, StdFileSystem};

    /// The instant the tests here act at, named so that a fixture change is one edit.
    const NOW_SECONDS: i64 = 1_000;

    /// When the live claim's lease runs out. Far past [`NOW_SECONDS`], so the holder the decline
    /// case refuses against is still holding the item at the instant that case asks.
    const LEASE_ENDS_AT_SECONDS: i64 = 9_000;

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
    fn Test_Validate_Current_Should_Load_And_Report_Todays_Violations()
    {
        let directory = Temporary_Directory("validate-current");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let mut reserves_nothing = Workable_Item("BAD-1");
        reserves_nothing.territory = Territory::Empty();
        let raw = serde_json::to_string(&LedgerDocument {
            schema_version: crate::SCHEMA_VERSION,
            items: vec![reserves_nothing],
        })
        .expect("the fixture document serializes");
        std::fs::write(ledger.Path(), raw).expect("test can write the raw fixture directly");

        let error = Validate_Current(&ledger).expect_err("an item reserving nothing violates an invariant");

        assert!(matches!(error, LedgerError::Invalid { .. }), "got {error:?}");
    }

    #[test]
    fn Test_Add_Item_Should_Refuse_A_Duplicate_Identifier()
    {
        let directory = Temporary_Directory("add-item");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Ledger_At(&directory, &clock);
        let item = Workable_Item("A-1");
        ledger
            .Save(&LedgerDocument { schema_version: crate::SCHEMA_VERSION, items: vec![item.clone()] })
            .expect("a fresh item is a valid document");
        let declared = RecordDeclaration { published: &Territory::Empty(), amending: &Territory::Empty() };

        let refusal = Add_Item(&mut ledger, &item, "agent-a", &declared)
            .expect_err("the identifier is already on the board");

        assert!(matches!(refusal, AddRefusal::AlreadyPresent { .. }), "got {refusal:?}");
    }

    #[test]
    fn Test_Decline_Item_Should_Refuse_An_Item_Someone_Else_Is_Holding()
    {
        let directory = Temporary_Directory("decline-item");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Ledger_Holding_A_Live_Claim(&directory, &clock);

        let refusal = Decline_Item(
            &mut ledger,
            &ItemId::New("D-1"),
            Holder::from("agent-b"),
            DeclineReason::from("not needed"),
        )
        .expect_err("a live claim held by somebody else must refuse the decline");

        assert!(matches!(refusal, ClaimRefusal::StillHeld { .. }), "got {refusal:?}");
    }

    /// A ledger holding one item claimed by `agent-a` and still live at the clock's instant.
    ///
    /// The claim expires well past that instant, so whoever asks for the item is asking while
    /// somebody else is holding it — which is the state every refusal of a live holder turns on.
    fn Ledger_Holding_A_Live_Claim<'clock>(
        directory: &std::path::Path,
        clock: &'clock FixedClock,
    ) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
    {
        let ledger = Ledger_At(directory, clock);
        let mut claimed = Workable_Item("D-1");
        claimed.state = ItemState::Claimed;
        claimed.claim = Some(Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(NOW_SECONDS),
            lease_expires_at: Timestamp::From_Unix_Seconds(LEASE_ENDS_AT_SECONDS),
        });
        ledger
            .Save(&LedgerDocument { schema_version: crate::SCHEMA_VERSION, items: vec![claimed] })
            .expect("a claimed item is a valid document");
        return ledger;
    }

    #[test]
    fn Test_Take_Over_Should_Refuse_A_Lease_Request_Beyond_The_Ceiling()
    {
        let directory = Temporary_Directory("take-over-verb");
        let clock = FixedClock(NOW_SECONDS);
        let mut ledger = Ledger_At(&directory, &clock);

        let refusal = Take_Over(
            &mut ledger,
            &ItemId::New("T-1"),
            "agent-a",
            crate::MAXIMUM_LEASE + Duration::from_secs(1),
        )
        .expect_err("a lease beyond the ceiling must be refused before anything is read");

        assert!(matches!(refusal, ClaimRefusal::LeaseTooLong { .. }), "got {refusal:?}");
    }

    fn Temporary_Directory(name: &str) -> std::path::PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-store-verbs-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
        }
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    fn Ledger_At<'clock>(
        directory: &std::path::Path,
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
