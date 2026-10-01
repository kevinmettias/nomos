//! Which arm a branch chose, and why.

use crate::ProducedState;

/// Which arm one branch chose, and why it chose it.
///
/// The *why* is structured rather than rendered. A prose reason would have been one more
/// thing to keep true and nothing could have checked it; these three fields are the whole
/// derivation -- the value read, the state it was observed in, and the arm that state
/// selected -- so a caller can re-derive the choice rather than read an account of it.
///
/// Both `Option`s carry the same case, and it is a real one: the node named by
/// [`Self::on`] may itself have been skipped by an earlier branch, so there is no state to
/// read. That is not a failure and it is not a default arm. Nothing is chosen, and every
/// node in every arm is skipped, because a branch that could not decide must not be read
/// as having decided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchChoice
{
    /// The branch node this choice belongs to.
    pub branch: String,
    /// The node whose published state the branch read.
    pub on: String,
    /// The state that node published, or `None` when it published nothing because it was
    /// itself skipped.
    pub observed: Option<ProducedState>,
    /// The position in the branch's own arm list of the arm that was chosen, or `None`
    /// when no arm matched the observed state or there was no state to match.
    pub arm: Option<usize>,
}
