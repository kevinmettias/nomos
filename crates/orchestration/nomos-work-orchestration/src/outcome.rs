//! What a `work` verb produced.
//!
//! Every variant carries a value [`nomos_ledger`] already hands out, or a small bundle of
//! the board and the moment it was read. Nothing here is a rendered word or an exit code:
//! [`crate::WorkOutcome`] is the seam's whole point, and a caller that matched a string out
//! of it would have reintroduced the coupling this crate exists to remove.

use nomos_ledger::{
    AddRefusal, ClaimRefusal, FinishRefusal, LedgerDocument, LedgerError, Reservation,
    VerificationRecord,
};
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
    /// This tree's revision right now, read the same way [`nomos_ledger::Finish`] reads it
    /// when it stamps a [`VerificationRecord`] — `.git/HEAD`, following one loose ref.
    /// `None` on any failure: no `.git` here, a packed ref this build does not chase, or any
    /// other read error.
    pub current_revision: Option<String>,
}

/// What a `work` verb produced.
pub enum WorkOutcome
{
    /// `list`: every item, or the board could not be read.
    List(Result<BoardView, LedgerError>),
    /// `show`: the board and this tree's revision, or the board could not be read.
    Show(Result<ShowView, LedgerError>),
    /// `add`: whether the item was recorded.
    Add(Result<(), AddRefusal>),
    /// `finish`: the verification record the predicate earned, or why it did not run to a
    /// pass.
    Finish(Result<VerificationRecord, FinishRefusal>),
    /// `claim`: the reservation, or what refused it.
    Claim(Result<Reservation, ClaimRefusal>),
    /// `renew`: the extended reservation, or what refused it.
    Renew(Result<Reservation, ClaimRefusal>),
    /// `takeover`: the reservation it displaced the old one for, or what refused it.
    TakeOver(Result<Reservation, ClaimRefusal>),
    /// `abandon`: whether the claim was given up.
    Abandon(Result<(), ClaimRefusal>),
    /// `decline`: whether the item was ended.
    Decline(Result<(), ClaimRefusal>),
    /// `validate`: the board, once it is known to satisfy its own invariants, or why it does
    /// not.
    Validate(Result<LedgerDocument, LedgerError>),
    /// `audit`: every item, or the board could not be read.
    Audit(Result<BoardView, LedgerError>),
}
