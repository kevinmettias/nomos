//! How much of the board a listing draws its rows from.

/// How much of the board a listing draws its rows from.
///
/// `OD-LEDGER-041` measured what the unbounded answer costs. `nomos work list` with no
/// filter printed one row per item -- 1,486 lines, of which 1,464 were items in a terminal
/// state that nobody can act on -- and `AGENTS.md` step 2 sends every session to exactly that
/// command before it does anything else. The same record refused moving those items to an
/// archive file, because the claim check reads a dependency's terminal state to answer its
/// dependent, and named this instead: the board keeps every item and the *view* is what gets
/// bounded.
///
/// A scope rather than another `--state` value, and the distinction is the reason the two do
/// not collapse into one flag. `--state` picks one bucket out of the board; this says how much
/// of the board the unfiltered listing draws from. Spelling the whole board as a state would
/// make `--state all` a sibling of `--state done`, which is a word about nothing an item can
/// be.
///
/// Nothing in [`crate::Run`] reads it, exactly as nothing there reads
/// [`crate::WorkCommand::List`]'s `state`. Both are carried in the request because the request
/// is where a caller says what it wants, and both are read by whichever composition root
/// renders the answer -- `OD-HOST-001`'s division, where this crate hands back a value and
/// never a line of output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListingScope
{
    /// Only the items that have not ended.
    ///
    /// The live board: every item `nomos_ledger::ItemState::Is_Finished` says is not over,
    /// however each one is labelled once `Claim_Refusal` has been asked about it. The default,
    /// because it is the answer to the question the operating contract's step 2 asks.
    Live,
    /// Every item the board holds, the finished and the declined included.
    Whole,
}
