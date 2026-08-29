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
use super::refusal::{Decline_Refusal, Takeover_Refusal};
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
