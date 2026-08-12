//! What one held item takes out of circulation.

use nomos_platform::Timestamp;
use crate::item::ItemId;
/// A granted claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation
{
    /// The item claimed.
    pub item: ItemId,
    /// Who holds it.
    pub holder: String,
    /// When the lease lapses.
    pub expires_at: Timestamp,
}
