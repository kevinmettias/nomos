//! What became of one node of a definition run.

use crate::ProducedState;

/// What became of one node of a definition run.
///
/// Four answers rather than a `ran: bool`, because "it dispatched", "a prior result stood
/// in for its dispatch" and "a branch decided it should not run at all" are three
/// different things that a boolean reports as two, and the one it loses is the one a
/// reader most needs: whether the effect actually happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeDisposition
{
    /// The node's body dispatched, and this is what it published.
    Dispatched
    {
        /// The state this node published for later conditions to read.
        produced: ProducedState,
    },
    /// The node's declared `nomos_contracts::Cacheability` permitted substituting a prior
    /// result, an earlier node's dispatch matched, and this node's body never dispatched.
    ///
    /// Kept apart from [`Self::Dispatched`] rather than folded into it, because the whole
    /// content of a cache hit is that the effect did *not* happen a second time, and a
    /// report that said only "it produced Clean" would have hidden exactly that.
    Served
    {
        /// The earlier node whose result stood in for this node's dispatch.
        from: String,
        /// The state this node published, which is the state the served node published.
        produced: ProducedState,
    },
    /// A branch chose an arm this node does not belong to, so it never ran.
    Skipped
    {
        /// The branch node that decided this node would not run.
        by: String,
    },
    /// A node that dispatches nothing -- a branch or a join -- settled and published.
    Settled
    {
        /// The state this node published for later conditions to read.
        produced: ProducedState,
    },
}
