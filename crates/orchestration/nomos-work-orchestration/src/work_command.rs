//! One `nomos work` command, independent of how it was spelled.
//!
//! This is the request vocabulary a caller above the platform hands to [`crate::Run`]. An
//! argument parser is one way to produce one; it is not the only way this crate expects one
//! to arrive, which is the point of stating the vocabulary here rather than leaving it a
//! shape only `nomos-cli`'s own parser produces.

use nomos_ledger::{ItemId, LedgerItem, Territory};

use crate::{ClaimRequest, EndingRequest, ListingScope};

/// What to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkCommand
{
    /// Show items, bounded by how much of the board to draw from and optionally filtered
    /// by state.
    List
    {
        /// Only items in this state.
        state: Option<String>,
        /// How much of the board the rows are drawn from when no state was named.
        ///
        /// It governs the unfiltered listing and only that one. A `state` is itself a bound,
        /// and the one the caller actually asked for: `--state done` wants terminal rows, so
        /// a scope applied on top of it would answer a question with none of the rows it
        /// named. `OD-LEDGER-041`.
        scope: ListingScope,
    },
    /// Report one item, including what has happened to it.
    Show
    {
        /// Which item.
        item: ItemId,
    },
    /// Put a new item on the ledger.
    Add
    {
        /// The item to record.
        item: Box<LedgerItem>,
        /// Which of the records it reserves the item edits rather than allocates.
        ///
        /// Beside the item rather than on it. The declaration decides whether the add is
        /// refused and has no reader afterwards, so carrying it on [`LedgerItem`] would put a
        /// field on a document two sessions share — and a build older than a field drops it
        /// silently at exit 0, which is what `OD-LEDGER-008` prices.
        amending: Territory,
    },
    /// Enlarge a held item's territory by paths its holder found it short of.
    ///
    /// The paths are beside the item rather than a request struct of their own, because the
    /// three fields have no reader after the verb refuses or applies -- the same shape, and
    /// the same reason, as [`WorkCommand::Add`]'s `amending`.
    Widen
    {
        /// Which item.
        item: ItemId,
        /// Who is widening it, which must be the live holder.
        holder: String,
        /// The paths to add. Never a replacement territory: dropping a path drops the
        /// `done_when` clause it carried, so there is deliberately no way to spell one here.
        adding: Vec<String>,
    },
    /// Run an item's verification predicate and record it done if it passes.
    Finish
    {
        /// Which item.
        item: ItemId,
        /// Who holds it.
        holder: String,
    },
    /// Take an item.
    Claim(ClaimRequest),
    /// Extend a held claim.
    Renew(ClaimRequest),
    /// Take over an item whose holder's lease ran out, keeping the claim it displaces.
    ///
    /// Separate from [`WorkCommand::Claim`] because taking another agent's abandoned work is
    /// a decision, and a decision belongs in a verb somebody typed — `OD-LEDGER-012`.
    TakeOver(ClaimRequest),
    /// Give up a claim without finishing.
    Abandon(EndingRequest),
    /// End an item that turned out not to be work.
    ///
    /// Separate from [`WorkCommand::Abandon`] because they are about different subjects —
    /// abandoning ends a claim and puts the item back on the board, declining ends the item —
    /// and because the items this exists for are unclaimed, which `abandon` cannot reach.
    /// `OD-LEDGER-019`.
    Decline(EndingRequest),
    /// Check the ledger's invariants.
    Validate,
    /// Show what would block a claim.
    Audit,
}
