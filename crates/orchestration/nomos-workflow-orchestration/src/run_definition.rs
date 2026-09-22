//! Running a published `WorkflowDefinition`, and replaying one against the definition it
//! ran under.

mod group_visit_order;
mod parallelism;
mod produced;
mod replay_refusal;
mod traversal;
mod waves;
mod workflow_execution;
mod workflow_run_record;

pub use group_visit_order::GroupVisitOrder;
pub use parallelism::Parallelism;
pub use replay_refusal::ReplayRefusal;
pub use workflow_execution::WorkflowExecution;
pub use workflow_run_record::WorkflowRunRecord;

use produced::State_Of;
use traversal::Traversal;

use nomos_platform::{Environment, FileSystem, ProgramLauncher};

use crate::run::Dispatching;
use crate::{
    BranchArm, BranchChoice, DefinitionRun, DispatchError, DispatchGroup, NodeDisposition, ProducedState, StepCompensation, WorkflowDefinition,
    WorkflowNode, WorkflowOutcome, WorkflowStepPlan,
};

/// Runs `definition` and records the run beside the definition it ran under.
///
/// The topology the sequential [`crate::Run`] cannot express: a branch chooses an arm by
/// the state an earlier node published, a join reconverges the arms it names, and nodes
/// that declare no dependence on one another share a dispatch group bounded by
/// `execution.parallelism`. `OD-ROADMAP-006` decision 2 supersedes the `OD-WORKFLOW-005`
/// clauses that excluded each of those, by name and by version.
///
/// Nothing here refuses at run time. Every refusal a definition can earn was taken at
/// [`WorkflowDefinition::Publish`], which is why a definition that exists can always be
/// run and why [`WorkflowOutcome::Refused`] never appears in what this reports.
///
/// The sequential path is untouched. [`crate::Run`], [`crate::Run_Unclocked`] and
/// [`crate::Run_With_Clock`] keep their signatures and their behaviour, and this shares
/// their per-step machinery rather than reimplementing it: a node's retry budget, its
/// declared timeout and its compensation are honored by exactly the code that already
/// honored them for a plan.
#[must_use]
pub fn Run_Definition<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    definition: &WorkflowDefinition, execution: &WorkflowExecution<'_, Launcher, Fs, Env>,
) -> WorkflowRunRecord
{
    let produced = Ran_Definition(definition, execution);

    return WorkflowRunRecord::Of(definition.clone(), produced);
}

/// Replays `record` against the definition it ran under, refusing when `against` has moved.
///
/// The replay runs `record`'s own definition, never `against`. That is the whole of
/// `OD-ROADMAP-006` decision 2's `WF-011` clause: a run is replayed against what it ran
/// under rather than against whatever a caller assembles now, and `against` is there to be
/// *compared*, not to be run. When the two agree the distinction is invisible, which is
/// exactly why the comparison has to be the thing that decides rather than a convention
/// nobody could check.
///
/// # Errors
///
/// [`ReplayRefusal`] when the offered definition's identity, version or content differs
/// from the one the record ran under.
pub fn Replay<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    record: &WorkflowRunRecord, against: &WorkflowDefinition, execution: &WorkflowExecution<'_, Launcher, Fs, Env>,
) -> Result<DefinitionRun, ReplayRefusal>
{
    if let Some(refusal) = Moved(record.Definition(), against)
    {
        return Err(refusal);
    }

    return Ok(Ran_Definition(record.Definition(), execution));
}

/// How `offered` differs from the definition a run recorded, or `None` when it does not.
///
/// Identity, then version, then content, because that is the order a reader needs them
/// in: a different workflow entirely, the same workflow at a different revision, and the
/// same revision saying something different are three answers of decreasing obviousness,
/// and the last is the one no caller would otherwise have noticed.
fn Moved(recorded: &WorkflowDefinition, offered: &WorkflowDefinition) -> Option<ReplayRefusal>
{
    if recorded.Id() != offered.Id()
    {
        return Some(ReplayRefusal::Identity { recorded: recorded.Id().clone(), offered: offered.Id().clone() });
    }

    if recorded.Version() != offered.Version()
    {
        return Some(ReplayRefusal::Version { recorded: recorded.Version(), offered: offered.Version() });
    }

    if recorded == offered
    {
        return None;
    }

    return Some(ReplayRefusal::Content { id: recorded.Id().clone(), version: recorded.Version() });
}

/// The one runner [`Run_Definition`] and [`Replay`] share.
fn Ran_Definition<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    definition: &WorkflowDefinition, execution: &WorkflowExecution<'_, Launcher, Fs, Env>,
) -> DefinitionRun
{
    let nodes = definition.Nodes();
    let fallen = waves::Waves(nodes);
    let cut = waves::Cut(&fallen, execution.parallelism.bound);
    let context = Dispatching { platform: execution.platform, variant: execution.variant, run: execution.run, read_clock: None };
    let mut state = Traversal::New(nodes, fallen);

    for (_, members) in &cut
    {
        let Some(failure) = Visited_Group(&Visiting(members, execution.parallelism.visit), &context, &mut state)
        else
        {
            continue;
        };

        return Failed_Run(failure, &state, Grouped_Report(nodes, &cut), execution.platform.filesystem);
    }

    return Completed_Run(&state, Grouped_Report(nodes, &cut));
}

/// The declared indices of `members`, in the order this run visits them.
fn Visiting(members: &[usize], visit: GroupVisitOrder) -> Vec<usize>
{
    return match visit
    {
        GroupVisitOrder::Declared => members.to_vec(),
        GroupVisitOrder::Reversed => members.iter().rev().copied().collect(),
    };
}

/// Visits one group, reporting the first node whose dispatch failed.
fn Visited_Group<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    visiting: &[usize], context: &Dispatching<'_, Launcher, Fs, Env>, state: &mut Traversal<'_>,
) -> Option<(usize, DispatchError)>
{
    for index in visiting
    {
        if let Some(error) = Visited_Node(*index, context, state)
        {
            return Some((*index, error));
        }
    }

    return None;
}

/// Visits one node: a step dispatches or is served, a branch decides, a join settles.
fn Visited_Node<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    index: usize, context: &Dispatching<'_, Launcher, Fs, Env>, state: &mut Traversal<'_>,
) -> Option<DispatchError>
{
    if state.Is_Settled(index)
    {
        return None;
    }

    let nodes = state.nodes;
    let node = nodes.get(index)?;

    return match node
    {
        WorkflowNode::Step { step, .. } => Visited_Step(index, step, context, state),
        WorkflowNode::Branch { .. } => Decided_Branch(index, state),
        WorkflowNode::Join { .. } => Settled_Join(index, state),
    };
}

/// Dispatches the step at `index`, unless its own declared `Cacheability` lets an earlier
/// node's result stand in for the dispatch.
fn Visited_Step<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    index: usize, step: &WorkflowStepPlan, context: &Dispatching<'_, Launcher, Fs, Env>, state: &mut Traversal<'_>,
) -> Option<DispatchError>
{
    if let Some(from) = Servable(index, step, state)
    {
        state.Serve(index, from);

        return None;
    }

    let attempted = crate::run::Attempted_Step(step, context, index, &mut state.attempts);

    return match attempted
    {
        Ok(outcome) =>
        {
            state.Dispatched(index, outcome);
            None
        }
        Err(error) => Some(error),
    };
}

/// The earlier node whose result may stand in for this step's dispatch, if any.
///
/// `OD-ROADMAP-006` decision 2 supersedes the **cache** half of `OD-WORKFLOW-005`'s
/// `WF-012` clause, so a runtime may substitute a prior result where `Cacheability`
/// permits it. What it permits is stated as `Cacheable::key_inputs`, a subset of field
/// names within the step's own `input_schema` -- a shape this crate does not resolve and
/// has no resolver for. So the key used here is strictly stronger than the declared one:
/// the whole body and the whole input schema must be equal, which implies every named key
/// input is equal whatever those names turn out to resolve to. A substitution is therefore
/// never made where the declaration would have forbidden one, and a step declaring
/// `NotCacheable` is never substituted at all. What is given up is hits the declaration
/// would have allowed, which is the right direction to be wrong in for a cache.
///
/// Only nodes in a strictly earlier wave are eligible, and that bound is not an
/// optimisation. Two nodes in one group have no order between them, so a substitution
/// inside a group would depend on which of the two this run happened to visit first --
/// which is exactly the leak the determinism guarantee promises does not happen.
fn Servable(index: usize, step: &WorkflowStepPlan, state: &Traversal<'_>) -> Option<usize>
{
    if !step.declaration.cacheability.Is_Cacheable()
    {
        return None;
    }

    let wave = state.Wave(index)?;
    let strictly_earlier = |earlier: &usize| return state.Wave(*earlier).is_some_and(|seen| return seen < wave);

    return (0..index).filter(strictly_earlier).find(|earlier| return Repeats(*earlier, step, state));
}

/// Whether the node at `earlier` dispatched exactly the input `step` declares.
fn Repeats(earlier: usize, step: &WorkflowStepPlan, state: &Traversal<'_>) -> bool
{
    let Some(WorkflowNode::Step { step: prior, .. }) = state.nodes.get(earlier)
    else
    {
        return false;
    };

    return state.Dispatched_Outcome(earlier).is_some() && prior.body == step.body && prior.declaration.input_schema == step.declaration.input_schema;
}

/// Chooses the branch at `index`'s arm, and skips the nodes of every arm it did not choose.
fn Decided_Branch(index: usize, state: &mut Traversal<'_>) -> Option<DispatchError>
{
    let nodes = state.nodes;
    let Some(WorkflowNode::Branch { name, on, arms }) = nodes.get(index)
    else
    {
        return None;
    };

    let observed = state.Produced_By_Name(on);
    let arm = observed.and_then(|seen| return arms.iter().position(|candidate| return candidate.when == seen));

    state.choices.push(BranchChoice { branch: name.clone(), on: on.clone(), observed, arm });
    Skipped_Arms(name, arms, arm, state);
    state.Settle(index, NodeDisposition::Settled { produced: observed.unwrap_or(ProducedState::Flagged) });

    return None;
}

/// Skips every node named by an arm this branch did not choose.
fn Skipped_Arms(branch: &str, arms: &[BranchArm], chosen: Option<usize>, state: &mut Traversal<'_>)
{
    for (position, arm) in arms.iter().enumerate()
    {
        if Some(position) != chosen
        {
            Skipped_Arm(branch, arm, state);
        }
    }
}

/// Skips every node `arm` names.
fn Skipped_Arm(branch: &str, arm: &BranchArm, state: &mut Traversal<'_>)
{
    for named in &arm.nodes
    {
        state.Skip(named, branch);
    }
}

/// Settles the join at `index` over the nodes it names.
fn Settled_Join(index: usize, state: &mut Traversal<'_>) -> Option<DispatchError>
{
    let nodes = state.nodes;
    let Some(WorkflowNode::Join { arms, .. }) = nodes.get(index)
    else
    {
        return None;
    };

    let produced = Joined_State(arms, state);
    state.Settle(index, NodeDisposition::Settled { produced });

    return None;
}

/// What a join publishes: clean only when at least one node it names published and every
/// one that published published clean.
///
/// A join over arms that were all skipped publishes [`ProducedState::Flagged`] rather than
/// clean, for the reason every other absence here answers that way: nothing ran, and
/// nothing having run is not a report that nothing needs acting on.
fn Joined_State(arms: &[String], state: &Traversal<'_>) -> ProducedState
{
    let published: Vec<ProducedState> = arms.iter().filter_map(|name| return state.Produced_By_Name(name)).collect();

    if published.is_empty()
    {
        return ProducedState::Flagged;
    }

    return State_Of(published.iter().all(|seen| return *seen == ProducedState::Clean));
}

/// What a run that reached the end of the definition reports.
fn Completed_Run(state: &Traversal<'_>, groups: Vec<DispatchGroup>) -> DefinitionRun
{
    return DefinitionRun {
        outcome: WorkflowOutcome::Completed { completed: state.Completed() },
        nodes: state.Reports(),
        groups,
        choices: state.choices.clone(),
        attempts: state.Attempts(),
        compensations: Vec::new(),
    };
}

/// What a run whose node at `failure` could not dispatch reports, the completed nodes
/// unwound.
fn Failed_Run<Fs: FileSystem>(failure: (usize, DispatchError), state: &Traversal<'_>, groups: Vec<DispatchGroup>, filesystem: &Fs) -> DefinitionRun
{
    let (index, error) = failure;

    return DefinitionRun {
        outcome: WorkflowOutcome::Failed { completed: state.Completed(), index, error },
        nodes: state.Reports(),
        groups,
        choices: state.choices.clone(),
        attempts: state.Attempts(),
        compensations: Compensated_Nodes(state, filesystem),
    };
}

/// Compensates every node that dispatched, in the reverse of the definition's declared
/// order.
fn Compensated_Nodes<Fs: FileSystem>(state: &Traversal<'_>, filesystem: &Fs) -> Vec<StepCompensation>
{
    let mut compensations = Vec::new();

    for index in (0..state.nodes.len()).rev()
    {
        if let Some(compensation) = Compensated_Node(index, state, filesystem)
        {
            compensations.push(compensation);
        }
    }

    return compensations;
}

/// What compensating the node at `index` did, or `None` when it dispatched nothing of its
/// own to undo -- a branch, a join, a skipped node, or one served an earlier result.
fn Compensated_Node<Fs: FileSystem>(index: usize, state: &Traversal<'_>, filesystem: &Fs) -> Option<StepCompensation>
{
    let WorkflowNode::Step { step, .. } = state.nodes.get(index)?
    else
    {
        return None;
    };
    let outcome = state.Dispatched_Outcome(index)?;

    return crate::run::Compensated_Step(step, outcome, index, filesystem);
}

/// The groups a run visited, named rather than indexed, for the report.
fn Grouped_Report(nodes: &[WorkflowNode], cut: &[(usize, Vec<usize>)]) -> Vec<DispatchGroup>
{
    return cut.iter().map(|(wave, members)| return Group_Report(nodes, *wave, members)).collect();
}

/// One group, named.
///
/// A member naming no declared node contributes no name, which cannot happen for a group
/// [`waves::Cut`] produced: every member of one is a declared index by construction.
fn Group_Report(nodes: &[WorkflowNode], wave: usize, members: &[usize]) -> DispatchGroup
{
    let named = members.iter().filter_map(|index| return nodes.get(*index)).map(|node| return node.Name().to_owned()).collect();

    return DispatchGroup { wave, nodes: named };
}
