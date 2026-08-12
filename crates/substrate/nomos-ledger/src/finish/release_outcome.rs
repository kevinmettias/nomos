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
