//! One arm of a branch: the state that chooses it, and the nodes that belong to it.

use super::ProducedState;

/// One arm of a [`super::WorkflowNode::Branch`]: the observed state that chooses it, and
/// the nodes that run only if it is chosen.
///
/// The arm names its nodes rather than containing them, which keeps a definition one flat
/// ordered list rather than a tree. A flat list is what makes the dependency waves
/// computable in one forward pass and what makes a node's position in the outcome's
/// ordering a single number -- `index` is already how [`crate::WorkflowOutcome`] names a
/// step, and a nested definition would have had to invent a second way to say where a
/// step was.
///
/// Every name here must be a node declared *after* the branch, which
/// [`super::WorkflowDefinition::Publish`] refuses otherwise: an arm reaching backward would
/// name a node that had already run before the branch could decide whether it should.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchArm
{
    /// The state the branched-on node must have published for this arm to be chosen.
    pub when: ProducedState,
    /// The nodes that run only when this arm is chosen, by name.
    pub nodes: Vec<String>,
}
