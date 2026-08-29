//! The in-memory store, and the walk an invalidation spreads through.

// The walk itself -- naming what a cause reaches, propagating to dependents, noting
// broadened guarantees, and settling the report -- is its own responsibility, split out of
// this file. It stays a sibling rather than a trait implementation's second home because
// none of it is part of this type's public surface.
#[path = "memory_store/invalidation.rs"] mod invalidation;

use crate::fact::sealed;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
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
    /// `&mut self` for `Try_Invalidate_One` and `self.keys`, which cannot coexist with a borrow
    /// of this field for the call that runs it. Always `Some` between calls; taking it and
    /// never restoring it is the one invariant this field asks a caller inside this file to
    /// keep.
    // Boxed as `dyn DependencyPropagation` rather than a generic parameter on `MemoryFactStore`
    // itself, because a type parameter here would spread into every public signature that
    // names this store; a swappable implementation behind one boxed trait object keeps that
    // seam local to this one field, which is exactly what `With_Propagation` below needs.
    propagation: Option<Box<dyn DependencyPropagation>>,
}

impl MemoryFactStore
{
    #[must_use]
    pub fn New() -> Self
    {
        use crate::propagation::LocalGraphPropagation;

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

    fn Try_Invalidate_One(&mut self, digest: Digest128, from: GenerationId, cause: &str) -> bool
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

    /// Whether `digest`'s latest entry is already invalidated, without mutating it -- the
    /// read-only half of what [`Self::Try_Invalidate_One`] checks before it mutates, split out
    /// so a walk can decide whether to keep spreading past a node under an immutable
    /// borrow, before any mutation happens. `OD-ANALYSIS-008` is why this exists as its own
    /// method rather than staying folded into `Try_Invalidate_One`.
    fn Is_Already_Invalidated(&self, digest: Digest128) -> bool
    {
        return self
            .entries
            .get(&digest)
            .and_then(|history| return history.last())
            .is_some_and(|entry| return entry.invalidated_at.is_some());
    }

    fn Latest(&self, digest: Digest128) -> Option<&Entry>
    {
        return self.entries.get(&digest).and_then(|history| return history.last());
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
        return invalidation::Invalidate_Reached(self, cause, from);
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
    // Accepts the substitute boxed as `dyn DependencyPropagation`, the same trait-object form
    // the `propagation` field stores, so a test can hand in any implementation without adding
    // a generic parameter to `MemoryFactStore`.
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

#[cfg(test)]
#[path = "memory_store/tests.rs"] mod tests;
