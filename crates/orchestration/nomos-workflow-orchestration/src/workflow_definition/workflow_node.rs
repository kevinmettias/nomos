//! One node of a workflow definition: a step that dispatches, a branch that chooses, or a
//! join that waits.

use super::BranchArm;
use crate::WorkflowStepPlan;

/// One node of a [`super::WorkflowDefinition`].
///
/// A node's own `name` is the value it publishes. There is no separate `produces` list,
/// because a step produces exactly one thing -- what its dispatch reported -- and a second
/// name for it would be a second authority for the same value. So "a condition over what
/// earlier steps produced" is spelled as a condition over an earlier node's name, and
/// `requires` is the dependence a step declares on what an earlier node produced.
///
/// Three variants and not four: there is no "end" node, because a definition is an ordered
/// list and the end of the list is the end of the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkflowNode
{
    /// A node that dispatches, through exactly the [`WorkflowStepPlan`] the sequential
    /// [`crate::Run`] already takes.
    ///
    /// The same pair, not a second declaration shape: a definition adds topology around a
    /// step and changes nothing about what a step *is*, so a plan assembled for `Run` is
    /// carried into a definition unchanged.
    Step
    {
        /// What later nodes name this node by, and the name of the value it publishes.
        name: String,
        /// The earlier nodes whose published values this step depends on.
        ///
        /// Empty means this step depends on nothing, which is what makes it eligible to
        /// share a dispatch group with the other nodes that depend on nothing.
        requires: Vec<String>,
        /// The declaration and body this node dispatches.
        step: WorkflowStepPlan,
    },
    /// A node that dispatches nothing and chooses which of its arms runs, by the state an
    /// earlier node published.
    Branch
    {
        /// What later nodes name this node by, and the name of the value it publishes --
        /// which is the state it observed, so a branch can itself be branched on.
        name: String,
        /// The earlier node whose published state chooses the arm.
        on: String,
        /// The arms, in the order they are considered. The first whose `when` matches the
        /// observed state is chosen and every other arm's nodes are skipped.
        arms: Vec<BranchArm>,
    },
    /// A node that dispatches nothing and settles once the nodes it names have each either
    /// run or been skipped.
    ///
    /// The wait is structural rather than temporal: the nodes a join names are all in
    /// strictly earlier dependency waves, so by the time the join is visited every one of
    /// them has settled. What the join adds is a single name the nodes after it can depend
    /// on, which is how two arms of a branch reconverge.
    Join
    {
        /// What later nodes name this node by, and the name of the value it publishes.
        name: String,
        /// The earlier nodes this join waits for, by name.
        arms: Vec<String>,
    },
}

impl WorkflowNode
{
    /// What this node is named, and so what it publishes under.
    #[must_use]
    pub fn Name(&self) -> &str
    {
        return match self
        {
            Self::Step { name, .. } | Self::Branch { name, .. } | Self::Join { name, .. } => name,
        };
    }
}
