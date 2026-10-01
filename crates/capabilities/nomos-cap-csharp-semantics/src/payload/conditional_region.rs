//! [`ConditionalRegion`], one branch of one conditional chain.

use super::branch::Branch;
use super::branch_state::BranchState;

/// One branch of a conditional chain: where it is, what it tests, and whether the build compiles
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalRegion
{
    /// The directive that opens the branch.
    pub branch: Branch,
    /// The one-based line of that directive.
    pub line: usize,
    /// The one-based line of the directive that closes the branch -- the next `#elif`, `#else`
    /// or `#endif` of the same chain. The branch's own lines are the ones strictly between.
    pub end_line: usize,
    /// The condition as written, with each run of whitespace collapsed to one space; empty for
    /// `#else`.
    pub condition: String,
    /// Whether the build compiles the branch.
    pub state: BranchState,
}
