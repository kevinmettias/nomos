//! The in-memory store, and the walk an invalidation spreads through.

// The walk itself -- naming what a cause reaches, propagating to dependents, noting
// broadened guarantees, and settling the report -- is its own responsibility, split out of
// this file. It stays a sibling rather than a trait implementation's second home because
// none of it is part of this type's public surface.
#[path = "memory_store/invalidation.rs"] mod invalidation;

// The written form of this store, and every refusal a build owes a file it did not write,
// is likewise its own responsibility and likewise a sibling: what a store *is* on disk is a
// separate question from what it answers in memory, and none of it is public surface
// either. `OD-ANALYSIS-009` is the decision that there is a written form at all.
#[path = "memory_store/persistence.rs"] mod persistence;

// Turning a fact identity into the number this store addresses it by, and back. A sibling
// for the same reason the two above are: what a caller addresses a fact by is unchanged, and
// nothing about the interning is reachable from outside this crate.
#[path = "memory_store/identities.rs"] mod identities;

use std::path::Path;
use nomos_contracts::SchemaId;
use crate::PersistenceError;
use crate::fact_store::sealed;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use nomos_contracts::GenerationId;
use self::identities::FactIdentities;
use self::identities::FactSlot;
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
/// How many entries of one key's history the store keeps.
///
/// One. [`MemoryFactStore::Push_Entry`] states why, where the history grows.
const RETAINED_HISTORY_ENTRIES: usize = 1;

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
    /// Every fact identity this store has been shown, and the one place a [`FactKey`]
    /// becomes the [`FactSlot`] the two structures below key on. `identities.rs` carries why
    /// a digest is not what they key on, and why there is no second table beside it.
    identities: FactIdentities,
    /// One key's history per slot, addressed by [`FactSlot::Position`].
    ///
    /// A vector rather than a map because a slot *is* a position: the interner mints them
    /// densely from zero, so a map would be a second way of saying what an index already
    /// says, and one that could disagree with the interner about which slots exist.
    entries: Vec<Vec<Entry>>,
    /// Which facts read each fact, slot to slot. Still an ordered map, because the walk that
    /// reads it is [`crate::propagation::DependencyPropagation`]'s and a deterministic
    /// iteration order is what makes two runs over one store agree; what changed is the
    /// width of the key it compares, from sixteen bytes to a machine word.
    dependents: BTreeMap<FactSlot, BTreeSet<FactSlot>>,
    materializations: u32,
    /// The strategy `Invalidate` spreads an invalidation with.
    ///
    /// A bare field rather than an `Option`: [the walk](self::invalidation::Propagate_To_Dependents)
    /// takes it by shared borrow, and the mutation it does afterwards
    /// (`Try_Invalidate_One`) sits in a second pass that runs after that borrow has ended, so
    /// there is no window in which the field has to be vacated and no absent state for a
    /// caller to trip over. `OD-ANALYSIS-008` is the two-pass split that removed the window;
    /// the `Option` and its `take` outlived it.
    // Boxed as `dyn DependencyPropagation` rather than a generic parameter on `MemoryFactStore`
    // itself, because a type parameter here would spread into every public signature that
    // names this store; a swappable implementation behind one boxed trait object keeps that
    // seam local to this one field, which is exactly what `With_Propagation` below needs.
    propagation: Box<dyn DependencyPropagation<FactSlot>>,
}

impl MemoryFactStore
{
    #[must_use]
    pub fn New() -> Self
    {
        use crate::propagation::LocalGraphPropagation;

        return Self::Spreading_With(Box::new(LocalGraphPropagation));
    }

    /// An empty store that spreads an invalidation with `propagation`.
    ///
    /// The one place this type's field list is written out, so a field added to it cannot be
    /// initialized two ways.
    fn Spreading_With(propagation: Box<dyn DependencyPropagation<FactSlot>>) -> Self
    {
        return Self {
            identities: FactIdentities::New(),
            entries: Vec::new(),
            dependents: BTreeMap::new(),
            materializations: 0,
            propagation,
        };
    }

    /// The slot `key` is addressed by, minting one -- and making room for its history -- the
    /// first time this store is shown it.
    ///
    /// The only thing that mints a slot. A read goes through [`FactIdentities::Slot_Of`]
    /// instead, which answers nothing for a key the store was never given: minting on a read
    /// would grow the store by a row per question asked of it.
    fn Slot_For(&mut self, key: &FactKey) -> FactSlot
    {
        let slot = self.identities.Intern(key);
        if self.entries.len() < self.identities.Count()
        {
            self.entries.resize_with(self.identities.Count(), Vec::new);
        }

        return slot;
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
        let slot = self.Slot_For(fact.Key());
        self.Refuse_Backdated(slot, &fact)?;

        for dependency in dependencies
        {
            let read = self.Slot_For(&dependency.key);
            self.dependents.entry(read).or_default().insert(slot);
        }

        self.Push_Entry(slot, Entry {
            fact,
            invalidated_at: None,
            cause: None,
            dependencies: dependencies.to_vec(),
        });
        self.materializations = self.materializations.saturating_add(1);

        return Ok(());
    }

    /// Appends `entry` to its key's history and applies the retention rule.
    ///
    /// # The retention rule
    ///
    /// **A key's history keeps its newest entry and drops everything behind it.**
    ///
    /// Every answer this store gives reaches its entry through [`Self::Latest`], which is
    /// `history.last()`: [`FactStore::Current`] through [`Self::Lookup`],
    /// [`FactStore::Historical`], [`Self::Dependencies_Of`], [`Self::Superseded_At`],
    /// [`Self::Live`], [`Self::Refuse_Backdated`], and the invalidation walk through
    /// [`Self::Is_Already_Invalidated`] and [`Self::Try_Invalidate_One`]. Nothing inside
    /// this crate or outside it can address an earlier entry, so one is unreachable rather
    /// than merely unused.
    ///
    /// The historical read is not the exception it sounds like. `Historical` answers from
    /// the newest entry's own `invalidated_at` and `cause` — supersession is recorded *on*
    /// the entry it superseded, not as a second entry behind it — so what it needs is
    /// exactly what is kept.
    ///
    /// A store that ends with its process could afford to keep the rest anyway; one that
    /// outlives it cannot, because an unbounded history is an unbounded file, growing by a
    /// write per materialization forever. That is why the rule is stated here, at the one
    /// place a history grows, rather than at the writer that would otherwise have to decide
    /// it a second time.
    fn Push_Entry(&mut self, slot: FactSlot, entry: Entry)
    {
        let Some(history) = self.entries.get_mut(slot.Position())
        else
        {
            // Unreachable: every slot reaching here came from `Slot_For`, which grows
            // `entries` to the interner's own count before it returns. Written as a refusal
            // rather than an index because a panic on the analysis path is a determinism
            // defect and not merely a crash -- a replay would have to reach the same panic
            // at the same step.
            return;
        };
        history.push(entry);

        let behind = history.len().saturating_sub(RETAINED_HISTORY_ENTRIES);
        if behind > 0
        {
            history.drain(..behind);
        }
    }

    /// Writes this store into `directory`, creating it if it is not there.
    ///
    /// # Errors
    ///
    /// Returns a [`PersistenceError`] naming the file, when the directory or the file
    /// cannot be written.
    pub fn Write_To_Directory(&self, directory: &Path) -> Result<(), PersistenceError>
    {
        return persistence::Write(self, directory);
    }

    /// Reads back a store written into `directory` by any process, or refuses the file
    /// whole.
    ///
    /// `understood_schemas` is what the reading build can interpret a payload as; a stored
    /// fact under any other schema is refused rather than served. A file written under a
    /// format version or a [`FactKey`] shape this build does not know is refused whole
    /// rather than partly read, because a key whose components moved does not mean what its
    /// bytes say and a rescue entry by entry would decide that silently.
    ///
    /// # Errors
    ///
    /// Returns a [`PersistenceError`] naming the file, for every one of those refusals and
    /// for a file that is missing, unreadable, corrupt or truncated.
    pub fn Read_From_Directory(
        directory: &Path,
        understood_schemas: &[SchemaId],
    ) -> Result<Self, PersistenceError>
    {
        return persistence::Read(directory, understood_schemas);
    }

    /// A write into a generation the store has already left.
    ///
    /// Refused rather than accepted as a second history entry, because the store answers
    /// reads at a generation and a fact filed behind the current one would be reachable at
    /// a generation it was never true at.
    fn Refuse_Backdated(
        &self,
        slot: FactSlot,
        fact: &MaterializedFact,
    ) -> Result<(), FactError>
    {
        let Some(latest) = self.Latest(slot)
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
            .iter()
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
            .Slot_Held_By(key)
            .and_then(|slot| return self.Latest(slot))
            .map(|entry| return entry.dependencies.clone())
            .unwrap_or_default();
    }

    /// The slot `key` is addressed by, or nothing if this store was never shown it.
    fn Slot_Held_By(&self, key: &FactKey) -> Option<FactSlot>
    {
        return self.identities.Slot_Of(key);
    }

    pub(crate) fn Lookup(&self, key: &FactKey, at: GenerationId) -> Result<&MaterializedFact, ()>
    {
        let Some(entry) = self.Slot_Held_By(key).and_then(|slot| return self.Latest(slot))
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
        return self
            .Slot_Held_By(key)
            .and_then(|slot| return self.Latest(slot))
            .and_then(|entry| return entry.invalidated_at);
    }

    fn Try_Invalidate_One(&mut self, slot: FactSlot, from: GenerationId, cause: &str) -> bool
    {
        let Some(entry) = self
            .entries
            .get_mut(slot.Position())
            .and_then(|history| return history.last_mut())
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

    /// Whether `slot`'s latest entry is already invalidated, without mutating it -- the
    /// read-only half of what [`Self::Try_Invalidate_One`] checks before it mutates, split out
    /// so a walk can decide whether to keep spreading past a node under an immutable
    /// borrow, before any mutation happens. `OD-ANALYSIS-008` is why this exists as its own
    /// method rather than staying folded into `Try_Invalidate_One`.
    fn Is_Already_Invalidated(&self, slot: FactSlot) -> bool
    {
        return self.Latest(slot).is_some_and(|entry| return entry.invalidated_at.is_some());
    }

    fn Latest(&self, slot: FactSlot) -> Option<&Entry>
    {
        return self.entries.get(slot.Position()).and_then(|history| return history.last());
    }
}

impl sealed::Sealed for MemoryFactStore
{}

impl FactStore for MemoryFactStore
{
    fn Current(&self, identity: &FactIdentity, at: GenerationId) -> Option<MaterializedFact>
    {
        return self.Current_Borrowed(identity, at).cloned();
    }

    fn Current_Borrowed(&self, identity: &FactIdentity, at: GenerationId) -> Option<&MaterializedFact>
    {
        return self.Lookup(&identity.key, at).ok();
    }

    fn Historical(&self, key: &FactKey) -> Option<(MaterializedFact, Supersession)>
    {
        let entry = self.Slot_Held_By(key).and_then(|slot| return self.Latest(slot))?;
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
    pub(crate) fn With_Propagation(propagation: Box<dyn DependencyPropagation<FactSlot>>) -> Self
    {
        return Self::Spreading_With(propagation);
    }
}

#[cfg(test)]
#[path = "memory_store/tests.rs"] mod tests;

// `mod tests` above is a SEPARATE file (`memory_store/tests.rs`). check-test-coverage's Rust
// front end keys a test's companion unit off the literal file it is textually written in, so
// a test living in that separate file can never address a function declared here, however it
// is named. This second, LITERAL inline module gives each method here the one-file address
// the check reads, without disturbing `memory_store/tests.rs`'s own broader behavioural suite.
#[cfg(test)]
mod local_tests
{
    use super::*;
    use crate::{Dependency, FactPayload, GuaranteeDigest, InputDigest, ReadOutcome};
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
        EvidenceClass, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SchemaId,
        SnapshotId, SubjectId,
    };

    /// The generation the accepted write in the backdating test is made at. It has to be
    /// later than the rewrite that follows, which is what the store refuses.
    const CURRENT_GENERATION: u64 = 5;

    /// The generation the store is invalidated at, later than every write here so the
    /// invalidation has something to reach.
    const INVALIDATION_GENERATION: u64 = 2;

    /// The subject of the second key a store writes, distinct from the first key's `1`.
    const SECOND_SUBJECT_SEED: u8 = 2;

    /// The subject of the key the dependency test names: nothing else in that test reads it,
    /// so it only has to differ from the key that carries the dependency.
    const DEPENDENCY_SUBJECT_SEED: u8 = 9;

    /// How many writes the counting test makes; one per `Materialize` call above it.
    const EXPECTED_MATERIALIZATIONS: u32 = 2;

    /// How many times the retention tests rewrite one key, each at a later generation than
    /// the last. More than the rule retains, which is the whole point of writing them.
    const HISTORY_WRITES: u64 = 4;

    /// The generation the retention tests invalidate at: later than every write above, so
    /// the invalidation reaches the entry that survived rather than one behind it.
    const INVALIDATION_AFTER_LAST_WRITE: u64 = 5;

    /// The variant component of every key in this module.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of every key in this module.
    const CONFIGURATION_SEED: u8 = 4;

    /// The workspace state the synthesized facts were measured against.
    const SNAPSHOT_SEED: u8 = 2;

    #[test]
    fn Test_New_Should_Start_Completely_Empty()
    {
        let store = MemoryFactStore::New();

        assert_eq!(store.Materializations(), 0);
        assert_eq!(store.Live(), 0);
    }

    #[test]
    fn Test_Materialize_Should_Refuse_A_Backdated_Rewrite()
    {
        let mut store = MemoryFactStore::New();
        let key = Key_For(1);
        let current = Fact_For(&key, GenerationId::From_Raw(CURRENT_GENERATION));
        store
            .Materialize(current, &[])
            .expect("first write cannot conflict");

        let backdated_fact = Fact_For(&key, GenerationId::From_Raw(1));
        let backdated = store.Materialize(backdated_fact, &[]);

        assert!(matches!(backdated, Err(FactError::Backdated { .. })));
    }

    #[test]
    fn Test_Materializations_Should_Count_Every_Successful_Write()
    {
        let mut store = MemoryFactStore::New();
        let first = Fact_For(&Key_For(1), GenerationId::From_Raw(1));
        store
            .Materialize(first, &[])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");
        let second = Fact_For(&Key_For(SECOND_SUBJECT_SEED), GenerationId::From_Raw(1));
        store
            .Materialize(second, &[])
            .expect("second write");

        assert_eq!(store.Materializations(), EXPECTED_MATERIALIZATIONS);
    }

    #[test]
    fn Test_Live_Should_Not_Count_An_Entry_Once_It_Is_Invalidated()
    {
        let mut store = MemoryFactStore::New();
        let key = Key_For(1);
        let fact = Fact_For(&key, GenerationId::From_Raw(1));
        store
            .Materialize(fact, &[])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");
        assert_eq!(store.Live(), 1);

        let slot = store.Slot_Held_By(&key).expect("the write above interned this key");
        store.Try_Invalidate_One(slot, GenerationId::From_Raw(INVALIDATION_GENERATION), "test");
        assert_eq!(store.Live(), 0);
    }

    #[test]
    fn Test_Dependencies_Of_Should_Return_What_The_Latest_Write_Named()
    {
        let mut store = MemoryFactStore::New();
        let dependency = Dependency {
            key: Key_For(DEPENDENCY_SUBJECT_SEED),
            outcome: ReadOutcome::Materialized,
        };
        let key = Key_For(1);
        let fact = Fact_For(&key, GenerationId::From_Raw(1));
        store
            .Materialize(fact, &[dependency.clone()])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");

        assert_eq!(store.Dependencies_Of(&key), vec![dependency]);
    }

    #[test]
    fn Test_Lookup_Should_Refuse_A_Key_Nobody_Wrote()
    {
        let store = MemoryFactStore::New();

        // `Lookup`'s error is `()` by design (every failure path collapses to it on
        // purpose -- `Reader` reads its own richer applicability instead), so there is no
        // variant to name here. Waived in suppressions.json rather than with an inline
        // `error-tests: allow` marker, which this repository's policy does not honour.
        assert!(store.Lookup(&Key_For(1), GenerationId::From_Raw(1)).is_err());
    }

    #[test]
    fn Test_Superseded_At_Should_Be_None_Before_Any_Invalidation()
    {
        let mut store = MemoryFactStore::New();
        let key = Key_For(1);
        let fact = Fact_For(&key, GenerationId::From_Raw(1));
        store
            .Materialize(fact, &[])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");

        assert_eq!(store.Superseded_At(&key), None);
    }

    #[test]
    fn Test_Push_Entry_Should_Keep_Only_What_The_Retention_Rule_Retains()
    {
        let key = Key_For(1);
        let mut store = MemoryFactStore::New();

        for generation in 1..=HISTORY_WRITES
        {
            store
                .Materialize(Fact_For(&key, GenerationId::From_Raw(generation)), &[])
                .expect("each write is at a later generation than the one before it");
        }

        let slot = store.Slot_Held_By(&key).expect("the writes above interned this key");
        assert_eq!(
            store.entries.get(slot.Position()).map(Vec::len),
            Some(RETAINED_HISTORY_ENTRIES),
            "the history grew past what the retention rule keeps"
        );
    }

    /// The retention rule's sufficiency, stated as the property that makes it safe: a store
    /// that wrote a key many times answers every question it can be asked exactly as one
    /// that wrote the same final fact once. If a dropped entry were reachable by any read,
    /// one of these would disagree.
    #[test]
    fn Test_A_Trimmed_History_Should_Answer_What_A_Single_Write_Answers()
    {
        let key = Key_For(1);
        let last = GenerationId::From_Raw(HISTORY_WRITES);
        let dependency = Dependency {
            key: Key_For(DEPENDENCY_SUBJECT_SEED),
            outcome: ReadOutcome::Materialized,
        };

        let mut rewritten = MemoryFactStore::New();
        for generation in 1..=HISTORY_WRITES
        {
            rewritten
                .Materialize(Fact_For(&key, GenerationId::From_Raw(generation)), &[dependency.clone()])
                .expect("each write is at a later generation than the one before it");
        }
        let mut written_once = MemoryFactStore::New();
        written_once
            .Materialize(Fact_For(&key, last), &[dependency])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");

        assert_ne!(
            rewritten.Materializations(),
            written_once.Materializations(),
            "both stores took the same number of writes, so this compares nothing"
        );
        Assert_Both_Stores_Answer_Alike(&rewritten, &written_once, &key, last);
    }

    /// Every read the store offers, asked of both stores, before and after an invalidation.
    fn Assert_Both_Stores_Answer_Alike(
        rewritten: &MemoryFactStore,
        written_once: &MemoryFactStore,
        key: &FactKey,
        at: GenerationId,
    )
    {
        assert_eq!(rewritten.Live(), written_once.Live());
        assert_eq!(rewritten.Dependencies_Of(key), written_once.Dependencies_Of(key));
        assert_eq!(rewritten.Superseded_At(key), written_once.Superseded_At(key));
        assert_eq!(
            rewritten.Current(&key.clone().At(at), at),
            written_once.Current(&key.clone().At(at), at)
        );
        assert_eq!(rewritten.Historical(key), written_once.Historical(key));
    }

    #[test]
    fn Test_A_Trimmed_History_Should_Invalidate_And_Read_Historically_Like_A_Single_Write()
    {
        let key = Key_For(1);
        let last = GenerationId::From_Raw(HISTORY_WRITES);
        let cause = GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: File_Guarantee().incremental,
        };

        let mut rewritten = MemoryFactStore::New();
        for generation in 1..=HISTORY_WRITES
        {
            rewritten
                .Materialize(Fact_For(&key, GenerationId::From_Raw(generation)), &[])
                .expect("each write is at a later generation than the one before it");
        }
        let mut written_once = MemoryFactStore::New();
        written_once
            .Materialize(Fact_For(&key, last), &[])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");

        let after = GenerationId::From_Raw(INVALIDATION_AFTER_LAST_WRITE);
        assert_eq!(
            rewritten.Invalidate(&cause, after).direct,
            written_once.Invalidate(&cause, after).direct
        );
        Assert_Both_Stores_Answer_Alike(&rewritten, &written_once, &key, last);
    }

    #[test]
    fn Test_Write_To_Directory_Should_Leave_A_Store_Behind_For_Another_Process()
    {
        let directory = Temporary_Directory("write");
        let mut store = MemoryFactStore::New();
        store
            .Materialize(Fact_For(&Key_For(1), GenerationId::From_Raw(1)), &[])
            .expect("Refuse_Backdated finds no entry under this key on a store this test just built");

        store.Write_To_Directory(&directory).expect("a fresh temporary directory is writable");

        assert!(persistence::Store_File(&directory).exists());
    }

    #[test]
    fn Test_Read_From_Directory_Should_Refuse_A_Directory_Holding_No_Store()
    {
        let directory = Temporary_Directory("absent");
        let _ = std::fs::remove_dir_all(&directory);

        let refused = MemoryFactStore::Read_From_Directory(&directory, &[]);

        assert!(matches!(refused, Err(PersistenceError::Unreadable { .. })));
    }

    fn Temporary_Directory(name: &str) -> std::path::PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-analysis-store-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
        }
        std::fs::create_dir_all(&path).expect("test needs a temporary directory");

        return path;
    }

    #[test]
    fn Test_With_Propagation_Should_Build_A_Store_That_Starts_Empty()
    {
        use crate::propagation::LocalGraphPropagation;

        let store = MemoryFactStore::With_Propagation(Box::new(LocalGraphPropagation));

        assert_eq!(store.Materializations(), 0);
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

    fn Key_For(subject_seed: u8) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.memory_fact_store"),
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

    fn Fact_For(key: &FactKey, generation: GenerationId) -> MaterializedFact
    {
        return MaterializedFact {
            identity: key.clone().At(generation),
            snapshot: SnapshotId::From_Digest(Seeded_Digest(SNAPSHOT_SEED)),
            evidence: EvidenceClass::Derived,
            guarantee: File_Guarantee(),
            payload: FactPayload::New(SchemaId::New("nomos.test.memory_fact_store.v1"), b"tree".to_vec()),
        };
    }
}
