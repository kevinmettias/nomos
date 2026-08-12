//! Granting a claim on a document, and replacing the one a lapse left behind.

use nomos_platform::Timestamp;

use crate::Claim;
use crate::ClaimRefusal;
use crate::LedgerItem;
use crate::ItemId;
use crate::LedgerDocument;

/// Marks an item claimed and records the grant.
pub(super) fn Install_Claim(document: &mut LedgerDocument, item: &ItemId, granted: &Claim)
{
    use crate::ItemState;

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
pub(super) fn With_Own_Claim(
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
pub(super) fn Replace_Lapsed(
    document: &mut LedgerDocument,
    item: &ItemId,
    replacement: &Claim,
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
