//! Condensing an invalidation into a rematerialization order.
//!
//! [`RematerializationGroup`] and the walk that produces it, split out of `store.rs` for the
//! same reason `docs/records/OD-AGENT-001` gives generally: a second public type sharing a
//! file with [`crate::GenerationCause`] paid for the folding in every name, and the boundary
//! this file draws was already named — Tarjan's algorithm over the facts one invalidation
//! reached, condensed into groups a caller can rematerialize in order.

use crate::FactKey;
use crate::InvalidationReport;
use crate::MemoryFactStore;
use nomos_contracts::Digest128;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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

    return Condense_Into_Groups(&nodes, &edges);
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
fn Condense_Into_Groups(
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

    /// Advances `frame` past `target`: folds `target`'s lowlink into `frame`'s node if it
    /// is already discovered and still on the SCC stack, or opens it as a new frame for the
    /// caller to push. The part of the original recursive `Visit` that ran once per edge.
    fn Advance(&mut self, frame: &mut Frame, target: Digest128) -> Option<Frame>
    {
        frame.next = frame.next.saturating_add(1);

        if self.indices.contains_key(&target)
        {
            let node = frame.node;
            self.Fold_If_On_Stack(node, target);
            return None;
        }

        return Some(self.Opened(target));
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Dependency, FactPayload, GenerationCause, GuaranteeDigest, InputDigest, MaterializedFact, ReadOutcome};
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, EvidenceClass,
        FactVariant, Guarantee, GenerationId, IncrementalGranularity, ProviderId, SchemaId, SnapshotId, SubjectId,
    };

    /// The subject of the second key in a pair. The first key's subject is `1`, which is
    /// identity rather than a choice; the two differ so a group of one key cannot pass for a
    /// group of two.
    const SECOND_SUBJECT_SEED: u8 = 2;

    /// The variant component of every key in this module.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of every key in this module.
    const CONFIGURATION_SEED: u8 = 4;

    /// The workspace state the synthesized facts were measured against.
    const SNAPSHOT_SEED: u8 = 2;

    /// The variant the invalidation report blames. Seeded apart from [`VARIANT_SEED`] so a
    /// report whose cause drifted to the key's own variant would not compare equal.
    const CAUSE_VARIANT_SEED: u8 = 9;

    /// The generation the report widens from, one past the generation the facts were
    /// materialized at, which is what makes them worth invalidating.
    const REPORT_GENERATION: u64 = 2;

    #[test]
    fn Test_Is_Cycle_Should_Be_True_Only_For_A_Group_Of_More_Than_One_Member()
    {
        assert!(!RematerializationGroup { members: vec![Key_For(1)] }.Is_Cycle());
        assert!(RematerializationGroup { members: vec![Key_For(1), Key_For(SECOND_SUBJECT_SEED)] }.Is_Cycle());
    }

    #[test]
    fn Test_Condensation_Of_Should_Group_Two_Facts_That_Depend_On_Each_Other()
    {
        let a = Key_For(1);
        let b = Key_For(SECOND_SUBJECT_SEED);
        let store = Mutually_Dependent(&a, &b);
        let report = Report_Naming(&[a.clone(), b.clone()]);

        let groups = Condensation_Of(&report, &store);

        assert_eq!(groups.len(), 1, "a mutual dependency is one group, not two: {groups:?}");
        let group = groups.first().expect("the assertion above found exactly one group");
        assert!(group.Is_Cycle());
        assert_eq!(Sorted_Members(&group.members), Sorted_Pair(&a, &b));
    }

    fn Key_For(subject_seed: u8) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.rematerialization_group"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded_Digest(subject_seed)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&File_Guarantee()),
            variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
        };
    }

    /// A store in which `first` reads `second` and `second` reads `first`, so condensing the
    /// pair yields one strongly connected group rather than two singleton ones.
    fn Mutually_Dependent(first: &FactKey, second: &FactKey) -> MemoryFactStore
    {
        let mut store = MemoryFactStore::New();
        let reading_second = Fact_For(first, GenerationId::From_Raw(1));
        store
            .Materialize(
                reading_second,
                &[Dependency { key: second.clone(), outcome: ReadOutcome::Materialized }],
            )
            .expect("the first reads the second");
        let reading_first = Fact_For(second, GenerationId::From_Raw(1));
        store
            .Materialize(
                reading_first,
                &[Dependency { key: first.clone(), outcome: ReadOutcome::Materialized }],
            )
            .expect("the second reads the first");

        return store;
    }

    fn Fact_For(key: &FactKey, generation: GenerationId) -> MaterializedFact
    {
        return MaterializedFact {
            identity: key.clone().At(generation),
            snapshot: SnapshotId::From_Digest(Seeded_Digest(SNAPSHOT_SEED)),
            evidence: EvidenceClass::Derived,
            guarantee: File_Guarantee(),
            payload: FactPayload::New(SchemaId::New("nomos.test.rematerialization_group.v1"), b"tree".to_vec()),
        };
    }

    fn Report_Naming(keys: &[FactKey]) -> InvalidationReport
    {
        return InvalidationReport {
            cause: GenerationCause::VariantChanged { variant: BuildVariantId::From_Digest(Seeded_Digest(CAUSE_VARIANT_SEED)) },
            from: GenerationId::From_Raw(REPORT_GENERATION),
            direct: keys.to_vec(),
            dependent: Vec::new(),
            broadened: Vec::new(),
            retained: 0,
        };
    }

    fn Sorted_Pair(first: &FactKey, second: &FactKey) -> Vec<FactKey>
    {
        return Sorted_Members(&[first.clone(), second.clone()]);
    }

    fn Sorted_Members(members: &[FactKey]) -> Vec<FactKey>
    {
        let mut sorted = members.to_vec();
        sorted.sort();
        return sorted;
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn File_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );
    }
}
