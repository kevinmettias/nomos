//! The dependency waves a definition's nodes fall into, and the groups a bound cuts a
//! wave into.

use std::num::NonZeroUsize;

use crate::WorkflowNode;

/// The dependency wave each node falls into, by declared index.
///
/// Wave zero is every node that reads nothing and belongs to no branch arm; a node's wave
/// is one past the highest wave it depends on. One forward pass suffices, and it suffices
/// because of what `WorkflowDefinition::Publish` already refused: a step's `requires` and
/// a branch's `on` resolve backward, a join's arms resolve backward, and the one forward
/// reference -- a branch arm naming the nodes that belong to it -- is a dependence in the
/// other direction, the arm's nodes on the branch. So every edge points from a lower index
/// to a higher one, and the waves of everything a node depends on are known by the time
/// that node is reached.
pub(super) fn Waves(nodes: &[WorkflowNode]) -> Vec<usize>
{
    let owners = Arm_Owners(nodes);
    let mut waves = vec![0_usize; nodes.len()];

    for index in 0..nodes.len()
    {
        let owner = owners.get(index).copied().flatten();
        let after_branch = owner.map_or(0, |owner| return After(&waves, owner));
        let after_reads = Reads(nodes, index).into_iter().map(|read| return After(&waves, read)).max().unwrap_or(0);

        let Some(wave) = waves.get_mut(index)
        else
        {
            continue;
        };

        *wave = after_branch.max(after_reads);
    }

    return waves;
}

/// One past the wave `index` fell into, or wave zero when `index` names no node.
///
/// Wave zero is the answer to an absence for the same reason it is the answer to a node
/// that depends on nothing: there is no wave this node must come after.
fn After(waves: &[usize], index: usize) -> usize
{
    return waves.get(index).map_or(0, |wave| return wave.saturating_add(1));
}

/// The groups a run visits, in order: the wave each was cut from and the declared indices
/// of its members.
///
/// A wave wider than `bound` becomes several groups rather than one oversized one, which
/// is the whole of how the bound is honored: every group a run reports holds at most
/// `bound` nodes, and a caller can check that against the number it stated.
pub(super) fn Cut(waves: &[usize], bound: NonZeroUsize) -> Vec<(usize, Vec<usize>)>
{
    let mut groups = Vec::new();
    let highest = waves.iter().copied().max().unwrap_or(0);

    for wave in 0..=highest
    {
        let members: Vec<usize> = waves.iter().enumerate().filter(|(_, at)| return **at == wave).map(|(index, _)| return index).collect();

        groups.extend(members.chunks(bound.get()).map(|chunk| return (wave, chunk.to_vec())));
    }

    return groups;
}

/// The declared indices, all lower than `index`, of the nodes the node at `index` reads.
fn Reads(nodes: &[WorkflowNode], index: usize) -> Vec<usize>
{
    let Some(node) = nodes.get(index)
    else
    {
        return Vec::new();
    };
    let earlier = nodes.get(..index).unwrap_or_default();
    let named: Vec<&String> = match node
    {
        WorkflowNode::Step { requires, .. } => requires.iter().collect(),
        WorkflowNode::Branch { on, .. } => vec![on],
        WorkflowNode::Join { arms, .. } => arms.iter().collect(),
    };

    return named.into_iter().filter_map(|name| return Position(earlier, name)).collect();
}

/// For each node, the branch whose arm names it, if any -- the one dependence that is not
/// spelled by the node itself.
fn Arm_Owners(nodes: &[WorkflowNode]) -> Vec<Option<usize>>
{
    let mut owners = vec![None; nodes.len()];

    for branch in 0..nodes.len()
    {
        Owned_By(nodes, branch, &mut owners);
    }

    return owners;
}

/// Records `branch` as the owner of every node its arms name.
fn Owned_By(nodes: &[WorkflowNode], branch: usize, owners: &mut [Option<usize>])
{
    let Some(WorkflowNode::Branch { arms, .. }) = nodes.get(branch)
    else
    {
        return;
    };

    for named in arms.iter().flat_map(|arm| return arm.nodes.iter())
    {
        let Some(owner) = Position(nodes, named).and_then(|owned| return owners.get_mut(owned))
        else
        {
            continue;
        };

        *owner = Some(branch);
    }
}

/// Where `name` is declared among `nodes`.
fn Position(nodes: &[WorkflowNode], name: &str) -> Option<usize>
{
    return nodes.iter().position(|node| return node.Name() == name);
}
