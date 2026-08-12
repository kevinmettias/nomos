//! The ledger file as a document, and the probe that reads its schema first.

use serde::Deserialize;
use serde::Serialize;
use crate::LedgerItem;
/// The on-disk form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
// The outermost of the strict containers. Reasoned once in `item.rs`'s module documentation and
// decided in `OD-LEDGER-008`: a build that cannot account for every key in the ledger does not
// get to write the ledger back.
#[serde(deny_unknown_fields)]
pub struct LedgerDocument
{
    /// Schema version, so a future reader can tell what it is looking at.
    pub schema_version: u32,
    /// The items, in a stable order.
    pub items: Vec<LedgerItem>,
}

/// The schema version alone, for explaining a strict parse that already failed.
///
/// A separate type, and deliberately *not* `deny_unknown_fields`: its whole job is to read one
/// field out of a document [`LedgerDocument`] has refused, which every key it does not declare
/// is the reason for.
#[derive(Deserialize)]
pub(crate) struct VersionProbe
{
    pub(crate) schema_version: u32,
}
