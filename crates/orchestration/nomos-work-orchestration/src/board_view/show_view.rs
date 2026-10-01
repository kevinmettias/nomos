//! What `show` read.

use nomos_ledger::LedgerDocument;
use nomos_platform::Timestamp;

/// What `show` read: the whole board (an item is reported alongside what happened to it,
/// and a caller finds it by id rather than this crate deciding "found" or "not found" on
/// its behalf — the lookup is free once the board is in hand), the moment, and this tree's
/// revision right now.
pub struct ShowView
{
    /// Every item, as the ledger holds them.
    pub document: LedgerDocument,
    /// The moment the board was read.
    pub now: Timestamp,
    /// This tree's revision right now, read the same way [`nomos_ledger::Finish_Item`] reads it
    /// when it stamps a [`nomos_ledger::VerificationRecord`] — `.git/HEAD`, following one
    /// loose ref. `None` on any failure: no `.git` here, a packed ref this build does not
    /// chase, or any other read error.
    pub current_revision: Option<String>,
}
