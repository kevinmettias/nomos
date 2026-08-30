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
    Holder_Entitled(candidate, holder)?;
    act(candidate);

    return Ok(());
}

/// Whether `holder` may change this item's claim.
fn Holder_Entitled(candidate: &LedgerItem, holder: &str) -> Result<(), ClaimRefusal>
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
/// `Try_Replace_Lapsed_Claim` and not two statements at the call site: the move of the old claim
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
            taken = candidate.Try_Replace_Lapsed_Claim(replacement.clone(), now);
        }
    }

    if !taken
    {
        return Err(ClaimRefusal::NoSuchItem { item: item.clone() });
    }

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ItemKind, ItemOrigin, ItemState, Territory};

    fn Item(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files(["src/a.rs"]),
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

    fn Claimed_Item(id: &str, holder: &str) -> LedgerItem
    {
        let mut item = Item(id);
        item.state = ItemState::Claimed;
        item.claim = Some(Claim {
            holder: holder.to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(1_000),
            lease_expires_at: Timestamp::From_Unix_Seconds(2_000),
        });
        return item;
    }

    fn Document_Of(items: Vec<LedgerItem>) -> LedgerDocument
    {
        return LedgerDocument { schema_version: crate::SCHEMA_VERSION, items };
    }

    #[test]
    fn Test_Install_Claim_Should_Mark_The_Item_Claimed_And_Record_The_Grant()
    {
        let mut document = Document_Of(vec![Item("T-1")]);
        let granted = Claim {
            holder: "agent-a".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(1_000),
            lease_expires_at: Timestamp::From_Unix_Seconds(2_000),
        };

        Install_Claim(&mut document, &ItemId::New("T-1"), &granted);

        let item = document.items.first().expect("the fixture has one item");
        assert_eq!(item.state, ItemState::Claimed);
        assert_eq!(item.claim, Some(granted));
    }

    #[test]
    fn Test_With_Own_Claim_Should_Refuse_A_Holder_That_Does_Not_Match()
    {
        let mut document = Document_Of(vec![Claimed_Item("T-2", "agent-a")]);

        let refusal = With_Own_Claim(&mut document, &ItemId::New("T-2"), "agent-b", |_| {})
            .expect_err("a different holder must be refused");
        assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }), "got {refusal:?}");

        With_Own_Claim(&mut document, &ItemId::New("T-2"), "agent-a", |candidate| {
            candidate.claim = None;
        })
        .expect("the true holder may act on its own claim");
        assert!(
            document.items.first().expect("Document_Of built one item").claim.is_none(),
            "the entitled holder's action must have run"
        );
    }

    #[test]
    fn Test_Replace_Lapsed_Should_Move_The_Old_Claim_Aside_And_Install_The_New_One()
    {
        let mut document = Document_Of(vec![Claimed_Item("T-3", "dead-agent")]);
        let replacement = Claim {
            holder: "agent-c".to_owned(),
            acquired_at: Timestamp::From_Unix_Seconds(5_000),
            lease_expires_at: Timestamp::From_Unix_Seconds(9_000),
        };

        Replace_Lapsed(&mut document, &ItemId::New("T-3"), &replacement, Timestamp::From_Unix_Seconds(5_000))
            .expect("a lapsed claim must be replaceable");

        let item = document.items.first().expect("Document_Of built one item");
        assert_eq!(item.claim.as_ref().map(|claim| return claim.holder.as_str()), Some("agent-c"));
        assert_eq!(item.displaced.len(), 1, "the claim the takeover replaced must be kept");
    }
}
