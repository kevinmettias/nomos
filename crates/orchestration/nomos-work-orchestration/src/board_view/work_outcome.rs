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
    /// pass, and the board as it stands afterwards.
    ///
    /// `board` is what lets a caller say which items this ending just made reachable or
    /// unreachable — `OD-LEDGER-038` decision 1. `None` when nothing ended (the refusal
    /// arm, where there is no fanout to report) or when re-reading the board failed, which
    /// is not allowed to fail a transition that already succeeded.
    Finish
    {
        finished: Result<VerificationRecord, FinishRefusal>,
        board: Option<LedgerDocument>,
    },
    /// `claim`: the reservation, or what refused it.
    Claim(Result<Reservation, ClaimRefusal>),
    /// `renew`: the extended reservation, or what refused it.
    Renew(Result<Reservation, ClaimRefusal>),
    /// `takeover`: the reservation it displaced the old one for, or what refused it.
    TakeOver(Result<Reservation, ClaimRefusal>),
    /// `abandon`: whether the claim was given up.
    Abandon(Result<(), ClaimRefusal>),
    /// `widen`: the paths actually added to the territory, or what refused the widening.
    ///
    /// What was added rather than what was asked for. A path the territory already reserved
    /// contributes nothing and is not recorded, so the two differ whenever a holder names one
    /// twice or names one they already had, and a caller echoing the request would report a
    /// widening that did not happen.
    Widen(Result<Vec<String>, ClaimRefusal>),
    /// `decline`: whether the item was ended, and the board as it stands afterwards.
    ///
    /// `board` carries the same thing, for the same reason, as [`WorkOutcome::Finish`]'s.
    /// A decline is the ending that strands dependents rather than freeing them, which is
    /// the case `OD-LEDGER-038` measured: seven items in one write, with nothing said.
    Decline
    {
        declined: Result<(), ClaimRefusal>,
        board: Option<LedgerDocument>,
    },
    /// `validate`: the board, once it is known to satisfy its own invariants, or why it does
    /// not.
    Validate(Result<LedgerDocument, LedgerError>),
    /// `audit`: every item, or the board could not be read.
    Audit(Result<BoardView, LedgerError>),
}
