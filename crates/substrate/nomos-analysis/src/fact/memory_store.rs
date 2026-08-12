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
use crate::Broadening;
use crate::InvalidationReport;
use crate::GenerationCause;
use crate::FactError;
use crate::FactKey;
use crate::Dependency;
use crate::MaterializedFact;
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

/// The state an invalidation spreads through: what it has already reached, what it has yet
/// to walk, and what it is reporting.
///
/// Carried as one value because the three change together at every step, and passing them
/// separately would put the edge walk over the argument budget.
struct Spreading<'a>
{
    /// Every digest already reached, so a cycle terminates.
    seen: &'a mut BTreeSet<Digest128>,
    /// Digests reached and not yet walked out of.
    frontier: &'a mut Vec<Digest128>,
    /// What the caller is told.
    report: &'a mut InvalidationReport,
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

    /// Invalidates everything that read this fact, and queues each of them in turn.
    ///
    /// A consumer already seen is not walked again. Without that, a store holding a cycle
    /// of dependency edges would never finish reporting one.
    fn Follow_Edges(
        &mut self,
        digest: Digest128,
        from: GenerationId,
        described: &str,
        spreading: &mut Spreading<'_>,
    )
    {
        let downstream: Vec<Digest128> = self
            .dependents
            .get(&digest)
            .map(|set| return set.iter().copied().collect())
            .unwrap_or_default();

        for consumer in downstream
        {
            if spreading.seen.insert(consumer)
            {
                self.Reach(consumer, from, described, spreading);
            }
        }
    }

    /// One consumer reached for the first time.
    fn Reach(
        &mut self,
        consumer: Digest128,
        from: GenerationId,
        described: &str,
        spreading: &mut Spreading<'_>,
    )
    {
        if !self.Invalidate_One(consumer, from, described)
        {
            return;
        }

        if let Some(key) = self.keys.get(&consumer)
        {
            spreading.report.dependent.push(key.clone());
        }
        spreading.frontier.push(consumer);
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
        let mut frontier = self.Invalidate_Named(cause, from, &described, &mut report);
        let mut seen: BTreeSet<Digest128> = frontier.iter().copied().collect();

        while let Some(digest) = frontier.pop()
        {
            self.Follow_Edges(digest, from, &described, &mut Spreading {
                seen: &mut seen,
                frontier: &mut frontier,
                report: &mut report,
            });
        }

        self.Note_Broadening(cause.Granularity(), &seen, &mut report);

        self.Settle(&mut report);

        return report;
    }
}
