use super::*;
use crate::Dependency;
use crate::FactPayload;
use crate::GuaranteeDigest;
use crate::InputDigest;
use crate::ReadOutcome;
use nomos_contracts::Assurance;
use nomos_contracts::BuildVariantId;
use nomos_contracts::CapabilityId;
use nomos_contracts::ConfigurationId;
use nomos_contracts::ContractVersion;
use nomos_contracts::Digest128;
use nomos_contracts::EvidenceClass;
use nomos_contracts::FactVariant;
use nomos_contracts::Guarantee;
use nomos_contracts::IncrementalGranularity;
use nomos_contracts::ProviderId;
use nomos_contracts::SchemaId;
use nomos_contracts::SnapshotId;
use nomos_contracts::SubjectId;
use std::collections::VecDeque;

/// Seed bytes for the identifiers a key names. Each is distinct so that two components
/// never collide on the same digest and hide a disagreement between them.
const SNAPSHOT_SEED: u8 = 2;
const VARIANT_SEED: u8 = 3;
const CONFIGURATION_SEED: u8 = 4;

/// The dependent fact's subject, deliberately not `upstream`'s: the only thing that
/// reaches this fact is the dependency edge below, which is the walk these tests drive.
const DERIVED_SUBJECT_SEED: u8 = 7;

fn Digest_From_Seed(seed: u8) -> Digest128
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

fn Key_For(subject_seed: u8) -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.syntax.tree"),
        contract_version: ContractVersion::New(1, 0),
        subject: SubjectId::From_Digest(Digest_From_Seed(subject_seed)),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.test"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&File_Guarantee()),
        variant: BuildVariantId::From_Digest(Digest_From_Seed(VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Digest_From_Seed(CONFIGURATION_SEED)),
    };
}

fn Fact_For(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: SnapshotId::From_Digest(Digest_From_Seed(SNAPSHOT_SEED)),
        evidence: EvidenceClass::Derived,
        guarantee: File_Guarantee(),
        payload: FactPayload::New(SchemaId::New("nomos.syntax.v1"), b"tree".to_vec()),
    };
}

/// A second, distinct [`DependencyPropagation`]: a proper FIFO queue rather than
/// [`LocalGraphPropagation`]'s stack, so the two reach downstream nodes in a
/// different order while computing the same reachable set. What this proves is the
/// reviewed correction's own point: the substitution boundary is `DependencyPropagation`,
/// not `FactStore` — this type never implements `FactStore` itself, `MemoryFactStore`
/// does, unchanged.
struct QueueOrderPropagation;

impl<Node> DependencyPropagation<Node> for QueueOrderPropagation
where
    Node: Copy + Ord,
{
    fn Spread(
        &self,
        dependents: &BTreeMap<Node, BTreeSet<Node>>,
        roots: Vec<Node>,
        on_reach: &mut dyn FnMut(Node) -> bool,
    )
    {
        let mut seen: BTreeSet<Node> = roots.iter().copied().collect();
        let mut frontier: VecDeque<Node> = roots.into();

        while let Some(node) = frontier.pop_front()
        {
            let Some(downstream) = dependents.get(&node)
            else
            {
                continue;
            };

            for consumer in downstream.iter().copied().collect::<Vec<_>>()
            {
                if seen.insert(consumer) && on_reach(consumer)
                {
                    frontier.push_back(consumer);
                }
            }
        }
    }
}

/// The store a test drives, and the two keys it holds: `upstream`, which the report's
/// cause names directly, and `derived`, which only the dependency edge reaches.
struct TwoFactStore
{
    store: MemoryFactStore,
    upstream: FactKey,
    derived: FactKey,
}

fn TwoFactStore(propagation: Box<dyn DependencyPropagation<FactSlot>>) -> TwoFactStore
{
    let upstream = Key_For(1);
    // A different subject than `upstream`, so `derived` is reached only by following
    // the dependency edge below — the walk this test exists to exercise — rather than
    // also matching `GenerationCause::SubjectChanged` directly.
    let derived = Key_For(DERIVED_SUBJECT_SEED);

    let derived_fact = Fact_For(&derived, GenerationId::INITIAL);
    let store = Store_With_Edge(propagation, &upstream, derived_fact);

    return TwoFactStore {
        store,
        upstream,
        derived,
    };
}

/// A store built over `propagation` holding `upstream` alone, and `derived` reading it — the
/// one edge that reaches `derived`.
fn Store_With_Edge(
    propagation: Box<dyn DependencyPropagation<FactSlot>>,
    upstream: &FactKey,
    derived: MaterializedFact,
) -> MemoryFactStore
{
    let mut store = MemoryFactStore::With_Propagation(propagation);
    let upstream_fact = Fact_For(upstream, GenerationId::INITIAL);
    store.Materialize(upstream_fact, &[]).expect("materializes");
    store
        .Materialize(
            derived,
            &[Dependency {
                key: upstream.clone(),
                outcome: ReadOutcome::Materialized,
            }],
        )
        .expect("materializes");

    return store;
}

#[test]
fn Test_An_Alternate_Propagation_Implementation_Should_Produce_The_Same_Report()
{
    use crate::propagation::LocalGraphPropagation;

    let mut default_fixture = TwoFactStore(Box::new(LocalGraphPropagation));
    let mut alternate_fixture = TwoFactStore(Box::new(QueueOrderPropagation));
    let next = GenerationId::INITIAL.Next();
    let cause = GenerationCause::SubjectChanged {
        subject: default_fixture.upstream.subject,
        granularity: IncrementalGranularity::File,
    };

    let default_report = default_fixture.store.Invalidate(&cause, next);
    let alternate_report = alternate_fixture.store.Invalidate(&cause, next);

    assert_eq!(
        default_report, alternate_report,
        "swapping the DependencyPropagation implementation changed FactStore's own \
         output, so MemoryFactStore is not actually independent of which one it holds"
    );
}
