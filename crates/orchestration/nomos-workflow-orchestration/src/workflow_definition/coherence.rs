//! What a definition is checked for before it can be published.
//!
//! Every check here reads the declaration alone. Nothing dispatches, nothing reads the
//! filesystem, and nothing needs a platform -- which is the whole point of doing it at
//! publish time rather than at run time.

use super::{BranchArm, DefinitionRefusal, WorkflowNode};
use crate::WorkflowStepPlan;

/// Why `nodes` cannot be published, or `None` when nothing refuses them.
///
/// Names are checked for uniqueness first, because every other check resolves a name and a
/// duplicate makes each of those resolutions meaningless rather than merely wrong.
pub(super) fn Refusal(nodes: &[WorkflowNode]) -> Option<DefinitionRefusal>
{
    if let Some(refusal) = Duplicate_Name(nodes)
    {
        return Some(refusal);
    }

    return nodes.iter().enumerate().find_map(|(index, node)| return Node_Refusal(nodes, index, node));
}

/// The first spelling two nodes share, if any.
fn Duplicate_Name(nodes: &[WorkflowNode]) -> Option<DefinitionRefusal>
{
    let mut seen: Vec<&str> = Vec::new();

    for node in nodes
    {
        if seen.contains(&node.Name())
        {
            return Some(DefinitionRefusal::DuplicateNodeName { name: node.Name().to_owned() });
        }
        seen.push(node.Name());
    }

    return None;
}

/// Why the node at `index` cannot be published.
///
/// A step and a branch read backward, so each is checked against the nodes declared before
/// it; a branch's arms reach forward, so they are checked against the nodes declared after
/// it. Slicing rather than searching the whole list is what makes the direction a fact of
/// the check rather than one more comparison inside it.
///
/// An `index` naming no node leaves both sides empty, so every name it could resolve
/// resolves to nothing and the node is refused rather than published unchecked.
fn Node_Refusal(nodes: &[WorkflowNode], index: usize, node: &WorkflowNode) -> Option<DefinitionRefusal>
{
    let earlier = nodes.get(..index).unwrap_or_default();
    let later = nodes.get(index.saturating_add(1)..).unwrap_or_default();

    return match node
    {
        WorkflowNode::Step { name, requires, step } => Step_Refusal(earlier, name, requires, step),
        WorkflowNode::Branch { name, on, arms } => Condition_Refusal(earlier, name, on).or_else(|| return Arms_Refusal(later, name, arms)),
        WorkflowNode::Join { name, arms } => Join_Refusal(earlier, name, arms),
    };
}

/// Why a step node cannot be published: an incoherent declaration, or a dependence on a
/// value nothing before it publishes.
fn Step_Refusal(earlier: &[WorkflowNode], name: &str, requires: &[String], step: &WorkflowStepPlan) -> Option<DefinitionRefusal>
{
    if !step.declaration.Is_Coherent()
    {
        return Some(DefinitionRefusal::IncoherentStep { node: name.to_owned() });
    }

    let missing = requires.iter().find(|value| return !Publishes(earlier, value))?;

    return Some(DefinitionRefusal::RequiresUnproducedValue { node: name.to_owned(), value: missing.clone() });
}

/// Why a branch cannot be published: it reads a value nothing before it publishes.
fn Condition_Refusal(earlier: &[WorkflowNode], name: &str, on: &str) -> Option<DefinitionRefusal>
{
    if Publishes(earlier, on)
    {
        return None;
    }

    return Some(DefinitionRefusal::BranchesOnUnproducedValue { branch: name.to_owned(), value: on.to_owned() });
}

/// Why a branch's arms cannot be published: an arm names a node not declared after it.
fn Arms_Refusal(later: &[WorkflowNode], name: &str, arms: &[BranchArm]) -> Option<DefinitionRefusal>
{
    let missing = arms.iter().flat_map(|arm| return arm.nodes.iter()).find(|node| return !Publishes(later, node))?;

    return Some(DefinitionRefusal::NamesUnknownNode { node: name.to_owned(), named: missing.clone() });
}

/// Why a join cannot be published: it names a node not declared before it.
fn Join_Refusal(earlier: &[WorkflowNode], name: &str, arms: &[String]) -> Option<DefinitionRefusal>
{
    let missing = arms.iter().find(|node| return !Publishes(earlier, node))?;

    return Some(DefinitionRefusal::NamesUnknownNode { node: name.to_owned(), named: missing.clone() });
}

/// Whether any of `nodes` publishes `value` -- which, because a node's own name is the
/// value it publishes, is whether any of them is named that.
fn Publishes(nodes: &[WorkflowNode], value: &str) -> bool
{
    return nodes.iter().any(|node| return node.Name() == value);
}
