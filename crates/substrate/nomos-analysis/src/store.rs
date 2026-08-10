use crate::fact::{FactError, MaterializedFact, Supersession};
use crate::identity::{FactIdentity, FactKey};
use crate::reader::Dependency;
use nomos_contracts::{
    BuildVariantId, ConfigurationId, Digest128, GenerationId, IncrementalGranularity, ProviderId,
    SnapshotId, SubjectId,
};
use std::collections::{BTreeMap, BTreeSet};

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

    fn Names(&self, key: &FactKey) -> bool
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Broadening
{
    pub key: FactKey,
    pub requested: IncrementalGranularity,
    pub applied: IncrementalGranularity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidationReport
{
    pub cause: GenerationCause,
    pub from: GenerationId,
    pub direct: Vec<FactKey>,
    pub dependent: Vec<FactKey>,
    pub broadened: Vec<Broadening>,
    pub retained: u32,
}

impl InvalidationReport
{
    #[must_use]
    pub fn Invalidated(&self) -> usize
    {
        return self.direct.len().saturating_add(self.dependent.len());
    }

    #[must_use]
    pub fn Report(&self) -> String
    {
        return format!(
            "{} invalidated {} directly and {} through dependency edges at {}, retaining {}",
            self.cause.Describe(),
            self.direct.len(),
            self.dependent.len(),
            self.from,
            self.retained
        );
    }
}

mod sealed
{
    pub trait Sealed {}
}

pub trait FactStore: sealed::Sealed
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>;

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>;

    fn Invalidate(&mut self, cause: &GenerationCause, from: GenerationId) -> InvalidationReport;
}

#[derive(Clone, Debug)]
struct Entry
{
    fact: MaterializedFact,
    invalidated_at: Option<GenerationId>,
    cause: Option<String>,
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Default)]
pub struct MemoryFactStore
{
    entries: BTreeMap<Digest128, Vec<Entry>>,
    dependents: BTreeMap<Digest128, BTreeSet<Digest128>>,
    keys: BTreeMap<Digest128, FactKey>,
    materializations: u32,
}

impl MemoryFactStore
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    pub fn Materialize(
        &mut self,
        fact: MaterializedFact,
        dependencies: &[Dependency],
    ) -> Result<(), FactError>
    {
        let digest = fact.Key().Digest();

        if let Some(latest) = self.Latest(digest)
        {
            if latest.fact.Generation() > fact.Generation()
            {
                return Err(FactError::Backdated {
                    identity: Box::new(fact.identity.clone()),
                    current: latest.fact.Generation(),
                });
            }
        }

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

impl sealed::Sealed for MemoryFactStore {}

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
        let mut report = InvalidationReport {
            cause: cause.clone(),
            from,
            direct: Vec::new(),
            dependent: Vec::new(),
            broadened: Vec::new(),
            retained: 0,
        };

        let named: Vec<Digest128> = self
            .keys
            .iter()
            .filter(|(_, key)| return cause.Names(key))
            .map(|(digest, _)| return *digest)
            .collect();

        let mut frontier: Vec<Digest128> = Vec::new();
        for digest in named
        {
            if self.Invalidate_One(digest, from, &described)
            {
                if let Some(key) = self.keys.get(&digest)
                {
                    report.direct.push(key.clone());
                }
                frontier.push(digest);
            }
        }

        let mut seen: BTreeSet<Digest128> = frontier.iter().copied().collect();
        while let Some(digest) = frontier.pop()
        {
            let downstream: Vec<Digest128> = self
                .dependents
                .get(&digest)
                .map(|set| return set.iter().copied().collect())
                .unwrap_or_default();

            for consumer in downstream
            {
                if !seen.insert(consumer)
                {
                    continue;
                }
                if !self.Invalidate_One(consumer, from, &described)
                {
                    continue;
                }

                if let Some(key) = self.keys.get(&consumer)
                {
                    report.dependent.push(key.clone());
                }
                frontier.push(consumer);
            }
        }

        let requested = cause.Granularity();
        for digest in &seen
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

        report.direct.sort();
        report.dependent.sort();
        report.broadened.sort_by(|first, second| return first.key.cmp(&second.key));
        report.retained = u32::try_from(self.Live()).unwrap_or(u32::MAX);

        return report;
    }
}
