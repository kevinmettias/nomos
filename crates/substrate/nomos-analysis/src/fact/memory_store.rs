//! The in-memory store, and the walk an invalidation spreads through.

use crate::fact::sealed;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use nomos_contracts::IncrementalGranularity;
use nomos_contracts::Digest128;
use nomos_contracts::GenerationId;
use crate::Supersession;
use crate::FactIdentity;
use crate::FactStore;
use crate::InvalidationReport;
use crate::GenerationCause;
use crate::FactError;
use crate::FactKey;
use crate::Dependency;
use crate::MaterializedFact;
use crate::propagation::DependencyPropagation;
use crate::propagation::LocalGraphPropagation;
#[derive(Clone, Debug)]
struct Entry
{
    fact: MaterializedFact,
    invalidated_at: Option<GenerationId>,
    cause: Option<String>,
    dependencies: Vec<Dependency>,
}

pub struct MemoryFactStore
{
    entries: BTreeMap<Digest128, Vec<Entry>>,
    dependents: BTreeMap<Digest128, BTreeSet<Digest128>>,
    keys: BTreeMap<Digest128, FactKey>,
    materializations: u32,
    /// Held as `Option` rather than bare `Box<dyn DependencyPropagation>` so `Invalidate`
    /// can take it out with [`Option::take`] before the walk: the walk's callback needs
    /// `&mut self` for `Invalidate_One` and `self.keys`, which cannot coexist with a borrow
    /// of this field for the call that runs it. Always `Some` between calls; taking it and
    /// never restoring it is the one invariant this field asks a caller inside this file to
    /// keep.
    propagation: Option<Box<dyn DependencyPropagation>>,
}

impl MemoryFactStore
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self {
            entries: BTreeMap::new(),
            dependents: BTreeMap::new(),
            keys: BTreeMap::new(),
            materializations: 0,
            propagation: Some(Box::new(LocalGraphPropagation)),
        };
    }

    /// # Errors
    ///
    /// Returns [`FactError::Backdated`] if a fact is already materialized under this
    /// identity at a newer generation than `fact` carries.
    pub fn Materialize(
        &mut self,
        fact: MaterializedFact,
        dependencies: &[Dependency],
    ) -> Result<(), FactError>
    {
        let digest = fact.Key().Digest();
        self.Refuse_Backdated(digest, &fact)?;

        for dependency in dependencies
        {
            let read = dependency.key.Digest();
            self.keys.insert(read, dependency.key.clone());
            self.dependents.entry(read).or_default().insert(digest);
        }

        self.keys.insert(digest, fact.Key().clone());
        self.entries.entry(digest).or_default().push(Entry {
            fact,
            invalidated_at: None,
            cause: None,
            dependencies: dependencies.to_vec(),
        });
        self.materializations = self.materializations.saturating_add(1);

        return Ok(());
    }

    /// A write into a generation the store has already left.
    ///
    /// Refused rather than accepted as a second history entry, because the store answers
    /// reads at a generation and a fact filed behind the current one would be reachable at
    /// a generation it was never true at.
    fn Refuse_Backdated(
        &self,
        digest: Digest128,
        fact: &MaterializedFact,
    ) -> Result<(), FactError>
    {
        let Some(latest) = self.Latest(digest)
        else
        {
            return Ok(());
        };

        if latest.fact.Generation() <= fact.Generation()
        {
            return Ok(());
        }

        return Err(FactError::Backdated {
            identity: Box::new(fact.identity.clone()),
            current: latest.fact.Generation(),
        });
    }

    #[must_use]
    pub fn Materializations(&self) -> u32
    {
        return self.materializations;
    }

    #[must_use]
    pub fn Live(&self) -> usize
    {
        return self
            .entries
            .values()
            .filter(|history| {
                return history
                    .last()
                    .is_some_and(|entry| return entry.invalidated_at.is_none());
            })
            .count();
    }

    #[must_use]
    pub fn Dependencies_Of(&self, key: &FactKey) -> Vec<Dependency>
    {
        return self
            .Latest(key.Digest())
            .map(|entry| return entry.dependencies.clone())
            .unwrap_or_default();
    }

    pub(crate) fn Lookup(&self, key: &FactKey, at: GenerationId) -> Result<&MaterializedFact, ()>
    {
        let Some(entry) = self.Latest(key.Digest())
        else
        {
            return Err(());
        };

        if entry.invalidated_at.is_some_and(|when| return when <= at)
        {
            return Err(());
        }
        if entry.fact.Generation() > at
        {
            return Err(());
        }

        return Ok(&entry.fact);
    }

    pub(crate) fn Superseded_At(&self, key: &FactKey) -> Option<GenerationId>
    {
        return self.Latest(key.Digest()).and_then(|entry| return entry.invalidated_at);
    }

    fn Latest(&self, digest: Digest128) -> Option<&Entry>
    {
        return self.entries.get(&digest).and_then(|history| return history.last());
    }

    fn Invalidate_One(&mut self, digest: Digest128, from: GenerationId, cause: &str) -> bool
    {
        let Some(entry) = self.entries.get_mut(&digest).and_then(|history| return history.last_mut())
        else
        {
            return false;
        };
        if entry.invalidated_at.is_some()
        {
            return false;
        }

        entry.invalidated_at = Some(from);
        entry.cause = Some(cause.to_owned());

        return true;
    }
}

/// A separate, wholly test-only `impl` block rather than one more method on the block
/// above: `With_Propagation` exists only to let a test substitute a `DependencyPropagation`
/// implementation, and keeping it out of the block real callers read keeps that block a
/// description of what this crate's other consumers can actually reach.
#[cfg(test)]
impl MemoryFactStore
{
    /// Built with a chosen `DependencyPropagation` rather than the default
    /// `LocalGraphPropagation` — crate-private because nothing outside this crate has a
    /// second implementation to offer yet. Exists to prove the seam `docs/records/D-138`
    /// promises: a test can substitute an alternate implementation and observe
    /// `FactStore::Invalidate` produce the same `InvalidationReport` without this type or
    /// `FactStore` changing.
    pub(crate) fn With_Propagation(propagation: Box<dyn DependencyPropagation>) -> Self
    {
        return Self {
            entries: BTreeMap::new(),
            dependents: BTreeMap::new(),
            keys: BTreeMap::new(),
            materializations: 0,
            propagation: Some(propagation),
        };
    }
}

/// An empty report of what this cause is about to invalidate.
fn Opened(cause: &GenerationCause, from: GenerationId) -> InvalidationReport
{
    return InvalidationReport {
        cause: cause.clone(),
        from,
        direct: Vec::new(),
        dependent: Vec::new(),
        broadened: Vec::new(),
        retained: 0,
    };
}

impl MemoryFactStore
{
    /// Invalidates every fact the cause names directly, and returns them as the frontier.
    fn Invalidate_Named(
        &mut self,
        cause: &GenerationCause,
        from: GenerationId,
        described: &str,
        report: &mut InvalidationReport,
    ) -> Vec<Digest128>
    {
        let named: Vec<Digest128> = self
            .keys
            .iter()
            .filter(|(_, key)| return cause.Names(key))
            .map(|(digest, _)| return *digest)
            .collect();

        let mut frontier: Vec<Digest128> = Vec::new();
        for digest in named
        {
            if !self.Invalidate_One(digest, from, described)
            {
                continue;
            }
            if let Some(key) = self.keys.get(&digest)
            {
                report.direct.push(key.clone());
            }
            frontier.push(digest);
        }

        return frontier;
    }

    /// Puts the report into the one order two runs over one store both produce.
    ///
    /// The retained count is taken last, after everything the cause reaches has been
    /// invalidated, because it is the answer to "what survived" and not to "what was here".
    fn Settle(&self, report: &mut InvalidationReport)
    {
        report.direct.sort();
        report.dependent.sort();
        report.broadened.sort_by(|first, second| return first.key.cmp(&second.key));
        report.retained = u32::try_from(self.Live()).unwrap_or(u32::MAX);
    }

    /// Every invalidated fact whose own guarantee is coarser than the cause was.
    ///
    /// Reported rather than silently applied: a provider that can only answer at whole-
    /// workspace granularity turns a one-file edit into a full rebuild, and the caller is
    /// entitled to know which provider did that and to how many facts.
    fn Note_Broadening(
        &self,
        requested: IncrementalGranularity,
        seen: &BTreeSet<Digest128>,
        report: &mut InvalidationReport,
    )
    {
        use crate::Broadening;

        for digest in seen
        {
            let (Some(key), Some(entry)) = (self.keys.get(digest), self.Latest(*digest))
            else
            {
                continue;
            };

            let applied = requested.Broadened_To(entry.fact.guarantee.incremental);
            if applied != requested
            {
                report.broadened.push(Broadening {
                    key: key.clone(),
                    requested,
                    applied,
                });
            }
        }
    }
}

impl sealed::Sealed for MemoryFactStore
{}

impl FactStore for MemoryFactStore
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>
    {
        return self.Lookup(&identity.key, at).ok().cloned();
    }

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>
    {
        let entry = self.Latest(key.Digest())?;
        let invalidated_at = entry.invalidated_at?;

        return Some((
            entry.fact.clone(),
            Supersession {
                invalidated_at,
                cause: entry.cause.clone().unwrap_or_default(),
            },
        ));
    }

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport
    {
        let described = cause.Describe();
        let mut report = Opened(cause, from);
        let roots = self.Invalidate_Named(cause, from, &described, &mut report);
        let mut seen: BTreeSet<Digest128> = roots.iter().copied().collect();

        // Cloned once, and `propagation` taken out of `self`, rather than either held as a
        // borrow across the walk: the callback below needs `&mut self` for `Invalidate_One`
        // and `self.keys`, which cannot coexist with a borrow of `self.dependents` or
        // `self.propagation` for the call that runs it. `DependencyPropagation` has no
        // Nomos-specific reason to know about that conflict — see `docs/records/D-135` and
        // `docs/records/D-138`.
        let dependents = self.dependents.clone();
        let propagation = self
            .propagation
            .take()
            .expect("propagation implementation is always present between calls");
        propagation.Spread(&dependents, roots, &mut |consumer| {
            seen.insert(consumer);
            if !self.Invalidate_One(consumer, from, &described)
            {
                return false;
            }
            if let Some(key) = self.keys.get(&consumer)
            {
                report.dependent.push(key.clone());
            }
            return true;
        });
        self.propagation = Some(propagation);

        self.Note_Broadening(cause.Granularity(), &seen, &mut report);

        self.Settle(&mut report);

        return report;
    }
}

#[cfg(test)]
mod tests
{
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
        store
            .Materialize(Fact_For(&upstream, GenerationId::INITIAL), &[])
            .expect("materializes");
        store
            .Materialize(
                Fact_For(&derived, GenerationId::INITIAL),
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
}
