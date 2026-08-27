//! [`BlockedItem`], carried only by [`super::audit::WorkAuditResponse::Audited`].

use nomos_ledger::LedgerItem;
use serde::Serialize;

/// One `Ready` item [`super::audit::Handle_Work_Audit`] found blocked, paired with why.
#[derive(Debug, Serialize)]
pub struct BlockedItem
{
    /// The blocked item, including what has happened to it.
    pub item: LedgerItem,
    /// What [`nomos_ledger::ClaimRefusal::Describe`] says stands between it and a claimant.
    pub cause: String,
}
