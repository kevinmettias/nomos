//! What a `work` verb produced.

use nomos_ledger::{
    AddRefusal, ClaimRefusal, FinishRefusal, LedgerDocument, LedgerError, Reservation,
    VerificationRecord,
};

use crate::board_view::{BoardView, ShowView};

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
