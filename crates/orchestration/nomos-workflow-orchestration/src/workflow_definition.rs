//! A workflow definition, published as an immutable artifact carrying its own identity and
//! version.

mod branch_arm;
mod coherence;
mod definition_refusal;
mod produced_state;
mod workflow_definition_id;
mod workflow_node;

pub use branch_arm::BranchArm;
pub use definition_refusal::DefinitionRefusal;
pub use produced_state::ProducedState;
pub use workflow_definition_id::WorkflowDefinitionId;
pub use workflow_node::WorkflowNode;

/// An immutable workflow definition, carrying its own identity and version.
///
/// `OD-ROADMAP-006` decision 2 supersedes `OD-WORKFLOW-005`'s `WF-009` clause, under which
/// this workspace published no immutable artifacts, and its `WF-011` clause, under which
/// there was no independently versioned definition to replay against. This is both: a
/// value whose fields are private and whose only constructor is [`Self::Publish`], so a
/// definition that exists is one that has been checked and cannot afterward be edited into
/// one that has not.
///
/// Immutable in the sense the replay guarantee needs, which is worth stating exactly
/// because the word is usually used for less. Nothing can reach inside a published
/// definition and change a node: there is no accessor handing out a mutable reference and
/// no public field. What a caller *can* do is publish a different definition under the
/// same identity, which is not a mutation of this value but a second artifact -- and
/// telling those two apart is exactly what [`crate::Replay`] does, by comparing the
/// definition a run recorded against the one a caller offers now.
///
/// Identity and version are both stated by the publisher rather than derived from the
/// content. A content digest would collapse the two into one number and lose the
/// distinction the replay refusal is built on: a definition republished at the same
/// version with different content is a mistake worth refusing loudly, and under a
/// content-derived identity it would instead be a different definition nobody had asked
/// about. [`WorkflowDefinitionId`]'s own doc carries the rest of that argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowDefinition
{
    id: WorkflowDefinitionId,
    version: u32,
    nodes: Vec<WorkflowNode>,
}

impl WorkflowDefinition
{
    /// Publishes `nodes` as the definition `id` at `version`, or refuses.
    ///
    /// Every refusal is taken here, before a definition value exists at all and therefore
    /// before any body could dispatch -- including each step's own
    /// `WorkflowStep::Is_Coherent`, which the sequential [`crate::Run`] checks one step at
    /// a time as it reaches them. That is the stricter promise a definition can make and a
    /// bare plan cannot: it holds the whole topology before anything runs.
    /// [`DefinitionRefusal`] names each case.
    ///
    /// # Errors
    ///
    /// [`DefinitionRefusal`] when the nodes do not form a coherent definition.
    pub fn Publish(id: WorkflowDefinitionId, version: u32, nodes: Vec<WorkflowNode>) -> Result<Self, DefinitionRefusal>
    {
        if let Some(refusal) = coherence::Refusal(&nodes)
        {
            return Err(refusal);
        }

        return Ok(Self { id, version, nodes });
    }

    /// The identity this definition was published under.
    #[must_use]
    pub const fn Id(&self) -> &WorkflowDefinitionId
    {
        return &self.id;
    }

    /// The version this definition was published at.
    #[must_use]
    pub const fn Version(&self) -> u32
    {
        return self.version;
    }

    /// The nodes, in the order they were declared.
    #[must_use]
    pub fn Nodes(&self) -> &[WorkflowNode]
    {
        return &self.nodes;
    }
}
