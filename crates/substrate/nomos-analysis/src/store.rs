use crate::FactKey;
use crate::InvalidationReport;
use crate::MemoryFactStore;
use nomos_contracts::{
    BuildVariantId, ConfigurationId, IncrementalGranularity, ProviderId,
    SnapshotId, SubjectId,
};
use nomos_contracts::Digest128;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationCause
{
    SubjectChanged
    {
        subject: SubjectId,
        granularity: IncrementalGranularity,
    },
    ConfigurationChanged
    {
        configuration: ConfigurationId,
    },
    ProviderChanged
    {
        provider: ProviderId,
    },
    /// The workspace was replaced wholesale — a checkout, a reopened store, a tree that
    /// moved under a running process.
    ///
    /// # Why it names the members that differ
    ///
    /// It used to name only the new snapshot and invalidate every fact whose key carried
    /// the old one. That worked because a fact key carried a snapshot, and it stopped
    /// working for a good reason: a key that names a workspace state changes for every fact
    /// in the corpus when one file is edited. The component is gone, so there is nothing on
    /// a key left to match a snapshot against.
    ///
    /// What replaces it is what the caller doing the replacing actually has. Something
    /// swapped one workspace state for another, and both are content-addressed maps of
    /// path to digest — the difference between them is a set of paths, computable without
    /// consulting the store at all. Invalidating by that set is also *narrower* than the
    /// old behaviour: a checkout that touched four files no longer discards a corpus.
    ///
    /// An empty `differing` set is not refused. Two snapshots that hold identical members
    /// and differ in variant or configuration are a real thing, and those have causes of
    /// their own. [`GenerationCause::Describe`] says how many members differed, so a
    /// replacement that invalidated nothing reads as a replacement that invalidated
    /// nothing rather than as a clean result.
    SnapshotReplaced
    {
        from: SnapshotId,
        to: SnapshotId,
        /// The subjects whose content is not the same in both states.
        differing: BTreeSet<SubjectId>,
    },
    VariantChanged
    {
        variant: BuildVariantId,
    },
}

impl GenerationCause
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::SubjectChanged {
                subject,
                granularity,
            } => format!("{subject} changed at {granularity:?} granularity"),
            Self::ConfigurationChanged { configuration } => {
                format!("configuration {configuration} was resolved differently")
            }
            Self::ProviderChanged { provider } => format!("provider {provider} changed"),
            Self::SnapshotReplaced {
                from,
                to,
                differing,
            } => format!(
                "snapshot {from} was replaced by {to}, in which {} member(s) differ",
                differing.len()
            ),
            Self::VariantChanged { variant } => format!("build variant {variant} changed"),
        };
    }

    #[must_use]
    pub const fn Granularity(&self) -> IncrementalGranularity
    {
        return match self
        {
            Self::SubjectChanged { granularity, .. } => *granularity,
            // A replacement that names its differing members is a statement about files,
            // the same as an edit is. It was `WholeWorkspace` while the cause could only
            // say "the snapshot is different" — and a cause reported coarser than what
            // happened makes every provider's broadening record read as unavoidable.
            Self::SnapshotReplaced { .. } => IncrementalGranularity::File,
            Self::ConfigurationChanged { .. }
            | Self::ProviderChanged { .. }
            | Self::VariantChanged { .. } => IncrementalGranularity::WholeWorkspace,
        };
    }

    /// Whether this cause reaches the fact `key` names.
    ///
    /// Crate-visible rather than public: the store asks it while spreading an
    /// invalidation, and a caller outside that walk asking it would be deciding for
    /// itself what a change reaches.
    pub(crate) fn Names(&self, key: &FactKey) -> bool
    {
        return match self
        {
            Self::SubjectChanged { subject, .. } => key.subject == *subject,
            Self::ConfigurationChanged { configuration } => key.configuration == *configuration,
            Self::ProviderChanged { provider } => key.provider == *provider,
            Self::SnapshotReplaced { differing, .. } => differing.contains(&key.subject),
            Self::VariantChanged { variant } => key.variant == *variant,
        };
    }
}

/// A rematerialization order over some of the facts one invalidation reached.
///
/// A group of one is an ordinary fact: nothing it depends on is itself in the invalidated
/// set, or everything it depends on has already been placed in an earlier group. A group of
/// more than one is a set of facts that depend on each other — directly, or through others
/// in the same group — for which no rematerialization order exists, because computing any
/// one of them needs one of the others first. That is not a fault in the graph; mutually
/// recursive subjects are the ordinary shape once facts are computed from other facts, and a
/// caller is entitled to see the group named as one rather than handed a false order over
/// it. Member order inside such a group carries no meaning; only membership does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RematerializationGroup
{
    pub members: Vec<FactKey>,
}

impl RematerializationGroup
{
    /// Whether this group is a mutual dependency rather than a single ordinary fact.
    #[must_use]
    pub fn Is_Cycle(&self) -> bool
    {
        return self.members.len() > 1;
    }
}

/// The facts `report` invalidated (`direct` together with `dependent`), condensed into
/// groups a caller can rematerialize in the order returned: a group never depends, through
/// `store`'s own dependency edges, on a group that comes after it.
///
/// This is computed fresh from `store`'s edges among exactly the keys `report` named, not
/// from the order the walk that produced `report` happened to reach them in — `direct` and
/// `dependent` are a record of what became stale, this is a separate answer to how the stale
/// facts depend on one another, and two invalidations that reach the same facts by different
/// paths produce the same groups in the same order.
///
/// A free function taking `report` and `store` together, rather than a method on
/// `InvalidationReport` alone, because the report by itself names only *which* facts became
/// stale — the edges between them are the store's, read back through its own
/// `Dependencies_Of`, and asking for them again here is cheaper than the report carrying a
/// second copy of a graph the store already keeps.
///
/// A dependency that leads outside the facts `report` named is not an edge here: that fact
/// was not invalidated, so it is read as-is rather than rematerialized, and it forms no
/// group.
#[must_use]
pub fn Condensation_Of(report: &InvalidationReport, store: &MemoryFactStore) -> Vec<RematerializationGroup>
{
    let nodes = Named_Nodes(report);
    let edges = Dependency_Edges(&nodes, store);

    return Condense(&nodes, &edges);
}

/// Every fact `report` named, keyed by digest.
fn Named_Nodes(report: &InvalidationReport) -> BTreeMap<Digest128, FactKey>
{
    let mut nodes: BTreeMap<Digest128, FactKey> = BTreeMap::new();
    for key in report.direct.iter().chain(report.dependent.iter())
    {
        nodes.insert(key.Digest(), key.clone());
    }

    return nodes;
}

/// The dependency edges among `nodes`, read back from `store`, with any edge leading
/// outside `nodes` dropped — that target was not invalidated, so it forms no edge here.
fn Dependency_Edges(
    nodes: &BTreeMap<Digest128, FactKey>,
    store: &MemoryFactStore,
) -> BTreeMap<Digest128, BTreeSet<Digest128>>
{
    let mut edges: BTreeMap<Digest128, BTreeSet<Digest128>> = BTreeMap::new();
    for (digest, key) in nodes
    {
        let targets: BTreeSet<Digest128> = store
            .Dependencies_Of(key)
            .iter()
            .map(|dependency| return dependency.key.Digest())
            .filter(|target| return nodes.contains_key(target))
            .collect();
        edges.insert(*digest, targets);
    }

    return edges;
}

/// Runs Tarjan's SCC algorithm over `nodes`/`edges` and resolves the components it finds
/// into rematerialization groups, each with its members sorted for a deterministic order.
fn Condense(
    nodes: &BTreeMap<Digest128, FactKey>,
    edges: &BTreeMap<Digest128, BTreeSet<Digest128>>,
) -> Vec<RematerializationGroup>
{
    let components = Strongly_Connected_Components(nodes, edges);

    return Resolved_Groups(nodes, components);
}

/// Runs Tarjan's algorithm over `nodes`/`edges`, visiting every node not already reached
/// from an earlier one, and returns its components in finishing order.
fn Strongly_Connected_Components(
    nodes: &BTreeMap<Digest128, FactKey>,
    edges: &BTreeMap<Digest128, BTreeSet<Digest128>>,
) -> Vec<Vec<Digest128>>
{
    let mut tarjan = Tarjan {
        edges,
        counter: 0,
        indices: BTreeMap::new(),
        lowlink: BTreeMap::new(),
        on_stack: BTreeSet::new(),
        stack: Vec::new(),
        components: Vec::new(),
    };

    for digest in nodes.keys()
    {
        if !tarjan.indices.contains_key(digest)
        {
            tarjan.Visit(*digest);
        }
    }

    return tarjan.components;
}

/// Each component resolved back into the fact keys `nodes` names, sorted for a
/// deterministic member order.
fn Resolved_Groups(
    nodes: &BTreeMap<Digest128, FactKey>,
    components: Vec<Vec<Digest128>>,
) -> Vec<RematerializationGroup>
{
    return components
        .into_iter()
        .map(|members| {
            let mut resolved: Vec<FactKey> = members
                .into_iter()
                .filter_map(|digest| return nodes.get(&digest).cloned())
                .collect();
            resolved.sort();

            return RematerializationGroup { members: resolved };
        })
        .collect();
}

/// Tarjan's strongly-connected-components algorithm, run over the induced subgraph one
/// invalidation reached.
///
/// Completion order is a property of the algorithm itself, not an artifact of which node is
/// visited first: a component only finishes once every component reachable from it has
/// finished, so appending components to `components` in finishing order always yields a
/// valid order over the condensation, regardless of which undiscovered node
/// `Condensation_Of` hands `Visit` next.
struct Tarjan<'a>
{
    edges: &'a BTreeMap<Digest128, BTreeSet<Digest128>>,
    counter: usize,
    indices: BTreeMap<Digest128, usize>,
    lowlink: BTreeMap<Digest128, usize>,
    on_stack: BTreeSet<Digest128>,
    stack: Vec<Digest128>,
    components: Vec<Vec<Digest128>>,
}

/// One node's position in the iterative walk `Visit` runs: which node it names, the
/// (already sorted, from the `BTreeSet` edge set) targets its edges reach, and how many of
/// them have been folded into its lowlink so far. Stands in for a recursive `Visit` call's
/// own stack frame, so a chain of dependencies does not recurse the native call stack one
/// level per fact.
struct Frame
{
    node: Digest128,
    targets: Vec<Digest128>,
    next: usize,
}

impl Tarjan<'_>
{
    /// Runs the walk from `start` to completion, iteratively: `frames` holds one entry per
    /// node currently open, in the same order a chain of recursive `Visit` calls would hold
    /// them on the native stack, and each iteration advances exactly the frame on top.
    fn Visit(&mut self, start: Digest128)
    {
        let mut frames: Vec<Frame> = vec![self.Opened(start)];

        while let Some(frame) = frames.last_mut()
        {
            if let Some(target) = frame.targets.get(frame.next).copied()
            {
                if let Some(opened) = self.Advance(frame, target)
                {
                    frames.push(opened);
                }

                continue;
            }

            let node = frame.node;
            self.Finish_Frame(&mut frames, node);
        }
    }

    /// Pops the finished top frame named `node`, closes it, and — if a parent frame is still
    /// open beneath it — folds its lowlink into that parent's.
    fn Finish_Frame(&mut self, frames: &mut Vec<Frame>, node: Digest128)
    {
        frames.pop();
        self.Finish(node);

        let Some(parent) = frames.last()
        else
        {
            return;
        };
        let parent_node = parent.node;
        let Some(child_low) = self.lowlink.get(&node).copied()
        else
        {
            return;
        };
        if let Some(entry) = self.lowlink.get_mut(&parent_node)
        {
            *entry = (*entry).min(child_low);
        }
    }

    /// Advances `frame` past `target`: folds `target`'s lowlink into `frame`'s node if it
    /// is already discovered and still on the SCC stack, or opens it as a new frame for the
    /// caller to push. The part of the original recursive `Visit` that ran once per edge.
    fn Advance(&mut self, frame: &mut Frame, target: Digest128) -> Option<Frame>
    {
        let node = frame.node;
        frame.next = frame.next.saturating_add(1);

        if self.indices.contains_key(&target)
        {
            self.Fold_If_On_Stack(node, target);
            return None;
        }

        return Some(self.Opened(target));
    }

    /// Assigns `node` its index and lowlink, places it on the SCC stack, and reads its edge
    /// targets — the part of the original recursive `Visit` that ran once per call before it
    /// walked edges.
    fn Opened(&mut self, node: Digest128) -> Frame
    {
        let index = self.counter;
        self.counter = self.counter.saturating_add(1);
        self.indices.insert(node, index);
        self.lowlink.insert(node, index);
        self.stack.push(node);
        self.on_stack.insert(node);

        let targets: Vec<Digest128> = self
            .edges
            .get(&node)
            .map(|set| return set.iter().copied().collect())
            .unwrap_or_default();

        return Frame { node, targets, next: 0 };
    }

    /// Lowers `node`'s lowlink against a back edge to `target`, an already-indexed node —
    /// only if `target` is still on the SCC stack, meaning it is part of an in-progress
    /// component rather than a finished, unrelated one.
    fn Fold_If_On_Stack(&mut self, node: Digest128, target: Digest128)
    {
        if !self.on_stack.contains(&target)
        {
            return;
        }

        let Some(target_index) = self.indices.get(&target).copied()
        else
        {
            return;
        };
        if let Some(entry) = self.lowlink.get_mut(&node)
        {
            *entry = (*entry).min(target_index);
        }
    }

    /// Pops `node`'s completed component off the SCC stack, if `node` is its root — the tail
    /// of the original recursive `Visit`, run once every edge out of `node` has been folded
    /// in.
    fn Finish(&mut self, node: Digest128)
    {
        let finished = self
            .lowlink
            .get(&node)
            .zip(self.indices.get(&node))
            .is_some_and(|(low, index)| return low == index);
        if !finished
        {
            return;
        }

        let mut component = Vec::new();
        while let Some(member) = self.stack.pop()
        {
            self.on_stack.remove(&member);
            component.push(member);
            if member == node
            {
                break;
            }
        }
        self.components.push(component);
    }
}
