//! Who is ending what, and why.

use nomos_ledger::ItemId;

/// Who is ending what, and why.
///
/// Shared by `abandon` and `decline` for the shape of the argument list only. What they end
/// is different, which is why they stay two verbs and two ledger calls — `OD-LEDGER-019`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndingRequest
{
    /// Which item.
    pub item: ItemId,
    /// Who is ending it.
    pub holder: String,
    /// Why.
    pub reason: String,
}
