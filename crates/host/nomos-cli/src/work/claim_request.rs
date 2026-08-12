//! Who is holding what, and for how long.

use std::time::Duration;

use nomos_ledger::ItemId;

/// Who is holding what, and for how long.
///
/// One type for `claim`, `renew` and `takeover` because they take exactly the same three
/// arguments and default the lease the same way. Three identical types would be three
/// chances for them to drift apart in what they accept while being documented as identical.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClaimRequest
{
    /// Which item.
    pub item: ItemId,
    /// Who is taking or holding it.
    pub holder: String,
    /// How long to hold it.
    pub lease: Duration,
}
