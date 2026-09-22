//! The state one definition run carries while it walks the nodes.

use super::produced::Produced_State;
use crate::{BranchChoice, NodeDisposition, NodeReport, ProducedState, StepAttempt, StepOutcome, WorkflowNode};

/// What a definition run knows as it goes: what each node settled as, what each dispatched
/// node produced, which arms were chosen and every attempt that was made.
///
/// Indexed by declared position throughout, never by visit position. That is what lets
/// every report this produces come out in the definition's own order regardless of which
/// order a group's members were visited in, without a sort that could be got wrong: the
/// order is the storage.
pub(super) struct Traversal<'a>
{
    /// The nodes this run is walking, in the order they were declared.
    pub(super) nodes: &'a [WorkflowNode],
    /// Which dependency wave each node falls into.
    waves: Vec<usize>,
    /// What each node settled as, or `None` until it settles.
    dispositions: Vec<Option<NodeDisposition>>,
    /// What each node that produced a result produced, by declared index.
    outcomes: Vec<Option<StepOutcome>>,
    /// Which arm each branch chose, in the order the branches were decided.
    pub(super) choices: Vec<BranchChoice>,
    /// Every attempt made, in the order the dispatches happened.
    pub(super) attempts: Vec<StepAttempt>,
}

impl<'a> Traversal<'a>
{
    /// A run of `nodes` that has settled nothing yet.
    pub(super) fn New(nodes: &'a [WorkflowNode], waves: Vec<usize>) -> Self
    {
        return Self {
            nodes,
            waves,
            dispositions: vec![None; nodes.len()],
            outcomes: vec![None; nodes.len()],
            choices: Vec::new(),
            attempts: Vec::new(),
        };
    }

    /// Records what became of the node at `index`.
    ///
    /// An `index` past the last declared node settles nothing, which is the same answer
    /// [`Self::Skip`] already gives a name no node answers to: there is no node to record
    /// anything about, so there is nothing to record.
    pub(super) fn Settle(&mut self, index: usize, disposition: NodeDisposition)
    {
        let Some(slot) = self.dispositions.get_mut(index)
        else
        {
            return;
        };

        *slot = Some(disposition);
    }

    /// Records that the node at `index` dispatched, and what it produced.
    pub(super) fn Dispatched(&mut self, index: usize, outcome: StepOutcome)
    {
        let produced = Produced_State(&outcome);
        let Some(slot) = self.outcomes.get_mut(index)
        else
        {
            return;
        };

        *slot = Some(outcome);
        self.Settle(index, NodeDisposition::Dispatched { produced });
    }

    /// Records that the node at `index` was served the result the node at `from` produced,
    /// without dispatching.
    pub(super) fn Serve(&mut self, index: usize, from: usize)
    {
        let Some(outcome) = self.Dispatched_Outcome(from).cloned()
        else
        {
            return;
        };
        let Some(source) = self.nodes.get(from)
        else
        {
            return;
        };
        let served = NodeDisposition::Served { from: source.Name().to_owned(), produced: Produced_State(&outcome) };
        let Some(slot) = self.outcomes.get_mut(index)
        else
        {
            return;
        };

        *slot = Some(outcome);
        self.Settle(index, served);
    }

    /// Records that the node named `name` will not run, because `by` chose another arm.
    pub(super) fn Skip(&mut self, name: &str, by: &str)
    {
        let Some(index) = self.Position(name)
        else
        {
            return;
        };

        self.Settle(index, NodeDisposition::Skipped { by: by.to_owned() });
    }

    /// Whether the node at `index` has already settled.
    pub(super) fn Is_Settled(&self, index: usize) -> bool
    {
        return matches!(self.dispositions.get(index), Some(Some(_)));
    }

    /// The dependency wave the node at `index` falls into, or `None` when `index` names no
    /// declared node.
    ///
    /// Absence is reported rather than answered with wave zero, because zero is a real
    /// wave: a caller comparing waves to decide what may stand in for what would read a
    /// fabricated zero as "earlier than everything" and substitute across a boundary the
    /// waves exist to hold.
    pub(super) fn Wave(&self, index: usize) -> Option<usize>
    {
        return self.waves.get(index).copied();
    }

    /// What the node at `index` dispatched, or `None` when it dispatched nothing of its
    /// own -- it was skipped, it was served an earlier node's result, or it dispatches
    /// nothing at all.
    pub(super) fn Dispatched_Outcome(&self, index: usize) -> Option<&StepOutcome>
    {
        if !matches!(self.dispositions.get(index), Some(Some(NodeDisposition::Dispatched { .. })))
        {
            return None;
        }

        return self.outcomes.get(index)?.as_ref();
    }

    /// The state the node named `name` published, or `None` when it published nothing
    /// because it was skipped or has not settled.
    pub(super) fn Produced_By_Name(&self, name: &str) -> Option<ProducedState>
    {
        let index = self.Position(name)?;

        return match self.dispositions.get(index)?.as_ref()?
        {
            NodeDisposition::Dispatched { produced } | NodeDisposition::Served { produced, .. } | NodeDisposition::Settled { produced } => Some(*produced),
            NodeDisposition::Skipped { .. } => None,
        };
    }

    /// What became of every node that settled, in the definition's declared order.
    pub(super) fn Reports(&self) -> Vec<NodeReport>
    {
        return self.nodes.iter().enumerate().filter_map(|(index, node)| return self.Report(index, node)).collect();
    }

    /// Every outcome this run produced, in the definition's declared order.
    pub(super) fn Completed(&self) -> Vec<StepOutcome>
    {
        return self.outcomes.iter().flatten().cloned().collect();
    }

    /// Every attempt, ordered by node and then by attempt rather than by dispatch.
    pub(super) fn Attempts(&self) -> Vec<StepAttempt>
    {
        let mut ordered = self.attempts.clone();
        ordered.sort_by_key(|attempt| return (attempt.index, attempt.attempt));

        return ordered;
    }

    /// What the node at `index` reported, or `None` when it never settled.
    fn Report(&self, index: usize, node: &WorkflowNode) -> Option<NodeReport>
    {
        let disposition = self.dispositions.get(index)?.clone()?;

        return Some(NodeReport { name: node.Name().to_owned(), index, disposition });
    }

    /// Where `name` is declared.
    fn Position(&self, name: &str) -> Option<usize>
    {
        return self.nodes.iter().position(|node| return node.Name() == name);
    }
}
