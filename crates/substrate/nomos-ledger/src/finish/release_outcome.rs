//! What became of a claim that was let go.

use nomos_platform::Timestamp;
use crate::LedgerItem;
use crate::VerificationRecord;
/// How a release ended.
///
/// The finished arm carries the [`VerificationRecord`], which is what makes a finished
/// item structurally impossible without one. An earlier shape had a bare `Finished`
/// variant, and it did not work: the release set the item to `Done`, the item had no
/// recorded verification, and the ledger's own validation then refused the write — so
/// the only way to finish anything was a path that could never succeed, reported as
/// "no such item". Carrying the record moves the requirement from a rule that rejects
/// the write to a signature that cannot express it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseOutcome
{
    /// The work was finished, and this is the evidence.
    Finished(VerificationRecord),
    /// The work was abandoned; the item returns to being claimable.
    Abandoned
    {
        /// Why.
        reason: String,
    },
}

impl ReleaseOutcome
{
    /// Writes this outcome onto the item it happened to, and ends the claim.
    ///
    /// The single place a release's evidence is persisted, for the same reason
    /// [`Refusal_From`] is the single place independence is judged: an implementation
    /// that spells the rule out for itself is free to spell one arm of it and not the
    /// other. That is not hypothetical — it is how this function came to exist. The
    /// `Finished` arm carried its [`VerificationRecord`] into storage and the `Abandoned`
    /// arm matched `{ .. }` and dropped the reason, adjacent arms of one match, one
    /// keeping its evidence and one discarding it. Both arms are now written once, next
    /// to the type that carries the evidence, so a second [`ExclusionLedger`] cannot
    /// persist half of it.
    ///
    /// Ending the claim is part of this rather than left to the caller. An abandonment is
    /// a record of something that stopped, and a record that went on excluding people
    /// would be a worse defect than the one this fixes.
    pub fn Record_On(&self, item: &mut LedgerItem, holder: &str, at: Timestamp)
    {
        use crate::Abandonment;
        use crate::ItemState;

        item.claim = None;

        match self
        {
            Self::Finished(record) =>
            {
                item.state = ItemState::Done;
                item.verified = Some(record.clone());
            }
            Self::Abandoned { reason } =>
            {
                item.state = ItemState::Ready;
                item.abandoned.push(Abandonment {
                    holder: holder.to_owned(),
                    reason: reason.clone(),
                    abandoned_at: at,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Claim, ItemId, ItemKind, ItemOrigin, ItemState, Territory};

    #[test]
    fn Test_Record_On_Should_Clear_The_Claim_And_Mark_A_Finished_Item_Done()
    {
        let mut item = Item("T-1");
        let record = VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: String::new(),
            verified_at: Timestamp::From_Unix_Seconds(1_500),
            gate: None,
            revision: None,
        };

        ReleaseOutcome::Finished(record.clone()).Record_On(&mut item, "agent-a", Timestamp::From_Unix_Seconds(2_500));

        assert_eq!(item.state, ItemState::Done);
        assert_eq!(item.claim, None, "a finished item is no longer held");
        assert_eq!(item.verified, Some(record), "the evidence must be kept, not just the verdict");
    }

    #[test]
    fn Test_Record_On_Should_Reopen_An_Abandoned_Item_Keeping_Its_Reason()
    {
        let mut item = Item("T-2");

        ReleaseOutcome::Abandoned { reason: "wrong approach".to_owned() }
            .Record_On(&mut item, "agent-b", Timestamp::From_Unix_Seconds(2_500));

        assert_eq!(item.state, ItemState::Ready);
        assert_eq!(item.claim, None, "an abandoned item is no longer held either");
        assert_eq!(item.abandoned.len(), 1, "the reason must survive, not just the state change");
        let abandonment = item.abandoned.first().expect("the assertion above found exactly one abandonment");
        assert_eq!(abandonment.holder, "agent-b");
        assert_eq!(abandonment.reason, "wrong approach");
    }

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
            state: ItemState::Claimed,
            depends_on: Vec::new(),
            blocked: None,
            claim: Some(Claim {
                holder: "agent-a".to_owned(),
                acquired_at: Timestamp::From_Unix_Seconds(1_000),
                lease_expires_at: Timestamp::From_Unix_Seconds(2_000),
            }),
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            widened: Vec::new(),
            declined: None,
        };
    }
}
