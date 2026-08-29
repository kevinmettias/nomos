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
use nomos_contracts::EvidenceClass;
use nomos_contracts::FactVariant;
use nomos_contracts::Guarantee;
use nomos_contracts::IncrementalGranularity;
use nomos_contracts::ProviderId;
use nomos_contracts::SchemaId;
use nomos_contracts::SnapshotId;
use nomos_contracts::SubjectId;
use std::collections::VecDeque;

fn Seeded(seed: u8) -> Digest128
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
        subject: SubjectId::From_Digest(Seeded(subject_seed)),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.test"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&File_Guarantee()),
        variant: BuildVariantId::From_Digest(Seeded(3)),
        configuration: ConfigurationId::From_Digest(Seeded(4)),
    };
}

fn Fact_For(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: SnapshotId::From_Digest(Seeded(2)),
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

impl DependencyPropagation for QueueOrderPropagation
{
    fn Spread(
        &self,
        dependents: &BTreeMap<Digest128, BTreeSet<Digest128>>,
        roots: Vec<Digest128>,
        on_reach: &mut dyn FnMut(Digest128) -> bool,
    )
    {
        let mut seen: BTreeSet<Digest128> = roots.iter().copied().collect();
        let mut frontier: VecDeque<Digest128> = roots.into();

        while let Some(digest) = frontier.pop_front()
        {
            let Some(downstream) = dependents.get(&digest)
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

fn Two_Fact_Store(propagation: Box<dyn DependencyPropagation>) -> (MemoryFactStore, FactKey, FactKey)
{
    let upstream = Key_For(1);
    // A different subject than `upstream`, so `derived` is reached only by following
    // the dependency edge below — the walk this test exists to exercise — rather than
    // also matching `GenerationCause::SubjectChanged` directly.
    let derived = Key_For(7);

    let mut store = MemoryFactStore::With_Propagation(propagation);
    let upstream_fact = Fact_For(&upstream, GenerationId::INITIAL);
    store.Materialize(upstream_fact, &[]).expect("materializes");
    let derived_fact = Fact_For(&derived, GenerationId::INITIAL);
    store
        .Materialize(
            derived_fact,
            &[Dependency {
                key: upstream.clone(),
                outcome: ReadOutcome::Materialized,
            }],
        )
        .expect("materializes");

    return (store, upstream, derived);
}

#[test]
fn Test_An_Alternate_Propagation_Implementation_Should_Produce_The_Same_Report()
{
    use crate::propagation::LocalGraphPropagation;

    let (mut default_store, upstream, _) = Two_Fact_Store(Box::new(LocalGraphPropagation));
    let (mut alternate_store, _, _) = Two_Fact_Store(Box::new(QueueOrderPropagation));
    let next = GenerationId::INITIAL.Next();
    let cause = GenerationCause::SubjectChanged {
        subject: upstream.subject,
        granularity: IncrementalGranularity::File,
    };

    let default_report = default_store.Invalidate(&cause, next);
    let alternate_report = alternate_store.Invalidate(&cause, next);

    assert_eq!(
        default_report, alternate_report,
        "swapping the DependencyPropagation implementation changed FactStore's own \
         output, so MemoryFactStore is not actually independent of which one it holds"
    );
}
