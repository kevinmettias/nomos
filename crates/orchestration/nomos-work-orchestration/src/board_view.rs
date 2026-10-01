//! What a `work` verb produced.
//!
//! Every variant carries a value [`nomos_ledger`] already hands out, or a small bundle of
//! the board and the moment it was read. Nothing here is a rendered word or an exit code:
//! [`crate::WorkOutcome`] is the seam's whole point, and a caller that matched a string out
//! of it would have reintroduced the coupling this crate exists to remove.

mod show_view;
mod work_outcome;

pub use show_view::ShowView;
pub use work_outcome::WorkOutcome;

use nomos_ledger::LedgerDocument;
use nomos_platform::Timestamp;

/// The board and the moment it was read.
///
/// Shared by `list` and `audit`: both need every item, and the labels a caller computes
/// over them — `Claim_Refusal`, `Listing_Label` and their kin in `nomos_ledger` — already
/// take exactly this pair.
pub struct BoardView
{
    /// Every item, as the ledger holds them.
    pub document: LedgerDocument,
    /// The moment the board was read, for lease and dependency questions asked against it.
    pub now: Timestamp,
}
