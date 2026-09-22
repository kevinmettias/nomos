//! The store as bytes on disk, and everything a build must refuse rather than believe.
//!
//! A child of [`super`] rather than a crate-root module, for the reason the invalidation
//! walk beside it is one: it reads and rebuilds [`MemoryFactStore`]'s own fields, and none
//! of that is part of the type's public surface.
//!
//! # What is written, and what is rebuilt
//!
//! The file carries what the store answers from — its keys, the retained entry of each
//! key's history, and the dependents graph — together with the count of writes the store has
//! taken. It carries all three addressed by *digest*, never by the [`super::identities::FactSlot`]
//! the running store keys on: a slot is a position in one process's interning table and means
//! nothing in the next one, which is exactly why the interner keeps the digest beside the key.
//! A file's own order is digest order for the same reason — written from a sort rather than
//! from the order this process happened to intern in, so two processes that materialized the
//! same facts in different orders write the same file. It does not carry the [`crate::propagation::DependencyPropagation`] the
//! store spreads an invalidation with: a strategy is the composing build's choice, not
//! state a previous process measured, so a reloaded store takes this build's default. What
//! is written is what a later process could not recompute without the corpus; what is not
//! written is what it decides for itself.
//!
//! The dependents graph is written rather than recomputed from the entries it could be
//! derived from, and that is deliberate. [`super::MemoryFactStore::Materialize`] adds an
//! edge per dependency and never removes one, so a store holds edges recorded by writes
//! whose entries the retention rule has since dropped. Rebuilding the graph from the
//! retained entries would quietly drop those edges, and a reloaded store would then
//! invalidate *less* than the store that wrote it — which is exactly the disagreement
//! `OD-ANALYSIS-009` asks this not to have.
//!
//! # Why `std::fs` and not the platform port
//!
//! `nomos_platform::FileSystem` is this workspace's seam onto the machine, and
//! `nomos-ledger` takes it for the file it owns. This does not, for two reasons that are
//! about this item rather than about the seam: the port's surface is text-only and has no
//! operation that creates a directory, and a store written *to a directory* must create the
//! one it is given; and `nomos-platform` is not this item's territory, so widening the port
//! here would be deciding a platform question inside an analysis one. Which host composes a
//! persisted store, and therefore which filesystem it should be handed, is the question
//! `OD-ANALYSIS-009`'s amendment leaves to the item that builds a process outliving an
//! invocation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use nomos_contracts::Applicability;
use nomos_contracts::BuildVariantId;
use nomos_contracts::CapabilityId;
use nomos_contracts::ConfigurationId;
use nomos_contracts::ContractVersion;
use nomos_contracts::Digest128;
use nomos_contracts::EvidenceClass;
use nomos_contracts::GenerationId;
use nomos_contracts::Guarantee;
use nomos_contracts::ProviderId;
use nomos_contracts::SchemaId;
use nomos_contracts::SnapshotId;
use nomos_contracts::SubjectId;

use super::Entry;
use super::MemoryFactStore;
use super::identities::FactSlot;
use crate::Component;
use crate::Dependency;
use crate::FactKey;
use crate::FactPayload;
use crate::GuaranteeDigest;
use crate::InputDigest;
use crate::MaterializedFact;
use crate::PersistenceError;
use crate::ReadOutcome;

/// The one file a store directory holds.
const STORE_FILE_NAME: &str = "fact-store.json";

/// The name the bytes are written under before they are renamed into place, so a reader
/// arriving mid-write sees the previous whole file rather than half of the next one. A
/// half-written file is still detectable — [`PersistenceError::Corrupt`] is what a reader
/// gets — but a format that can be torn by an ordinary write would make that the common
/// case rather than the damaged one.
const SCRATCH_FILE_NAME: &str = "fact-store.json.writing";

/// The version of the written form itself.
///
/// Raised when the file's own shape changes — a field added to [`StoredStore`], an entry
/// spelled differently — and read before anything else in the file is looked at, so a build
/// that does not understand the shape never reaches the entries. It is not a version of the
/// *key*: [`Understood_Key_Shape`] carries that, separately, because the two move for
/// different reasons and a reader needs to be told which one disagrees.
const FORMAT_VERSION: u32 = 1;

/// Writes `store` into `directory`, creating it if it is not there.
pub(super) fn Write(store: &MemoryFactStore, directory: &Path) -> Result<(), PersistenceError>
{
    let file = directory.join(STORE_FILE_NAME);
    fs::create_dir_all(directory).map_err(|error| return Unwritable(&file, &error))?;

    let bytes =
        serde_json::to_vec(&Stored_From_Store(store)).map_err(|error| return Unwritable(&file, &error))?;

    let scratch = directory.join(SCRATCH_FILE_NAME);
    fs::write(&scratch, &bytes).map_err(|error| return Unwritable(&scratch, &error))?;
    fs::rename(&scratch, &file).map_err(|error| return Unwritable(&file, &error))?;

    return Ok(());
}

/// Reads the store `directory` holds, or refuses the whole file.
///
/// `understood_schemas` is what the reading build can interpret a payload as. This crate
/// holds payload bytes opaquely and cannot decide the question for itself, so the build
/// that composed the providers declares it; a schema outside that set is refused here
/// rather than handed on to something that would read the bytes as a schema they were not
/// written in.
pub(super) fn Read(
    directory: &Path,
    understood_schemas: &[SchemaId],
) -> Result<MemoryFactStore, PersistenceError>
{
    let file = directory.join(STORE_FILE_NAME);
    let bytes = fs::read(&file).map_err(|error| return Unreadable(&file, &error))?;

    let stored: StoredStore =
        serde_json::from_slice(&bytes).map_err(|error| return Corrupt(&file, &error))?;

    // Both refusals run over the whole file before one entry is built, which is what
    // "refused wholesale" means here: there is no state to have half-populated.
    Refuse_Foreign_Header(&file, &stored)?;
    Refuse_Unknown_Schemas(&file, &stored.entries, understood_schemas)?;

    return Store_From_Stored(&file, &stored);
}

/// The key components this build addresses a fact by, in order.
///
/// [`Component`] is this crate's own declared mirror of [`FactKey`]'s fields — its `Label`
/// match is exhaustive and `tests/fact_identity/key_identity.rs` holds the two in step — so
/// a component added, removed or renamed changes this list, and every store written under
/// the old one is refused whole.
fn Understood_Key_Shape() -> Vec<String>
{
    return Component::All().iter().map(|component| return component.Label().to_owned()).collect();
}

/// Refuses a file whose format version or key shape is not this build's.
fn Refuse_Foreign_Header(file: &Path, stored: &StoredStore) -> Result<(), PersistenceError>
{
    if stored.format_version != FORMAT_VERSION
    {
        return Err(PersistenceError::ForeignFormat {
            file: file.to_path_buf(),
            written: stored.format_version,
            understood: FORMAT_VERSION,
        });
    }

    let understood = Understood_Key_Shape();
    if stored.key_shape != understood
    {
        return Err(PersistenceError::ForeignKeyShape {
            file: file.to_path_buf(),
            written: stored.key_shape.clone(),
            understood,
        });
    }

    return Ok(());
}

/// Refuses a file carrying a fact under a payload schema the reading build did not declare.
fn Refuse_Unknown_Schemas(
    file: &Path,
    entries: &[StoredEntry],
    understood_schemas: &[SchemaId],
) -> Result<(), PersistenceError>
{
    for entry in entries
    {
        if !understood_schemas.contains(&entry.schema)
        {
            return Err(PersistenceError::UnknownPayloadSchema {
                file: file.to_path_buf(),
                schema: entry.schema.clone(),
            });
        }
    }

    return Ok(());
}

/// Rebuilds the store's structures and its write count, once the header has been believed.
///
/// The keys are interned first, because everything else the file carries names a key by its
/// digest and the running store addresses one by its slot: interning is what turns the
/// file's vocabulary into this process's. A store rebuilt this way mints slots in the file's
/// own order, which is digest order, so a reload is as repeatable as the write was.
fn Store_From_Stored(file: &Path, stored: &StoredStore) -> Result<MemoryFactStore, PersistenceError>
{
    let mut store = MemoryFactStore::New();
    let slots = Interned_Slots(file, &mut store, &stored.keys)?;

    let entries = Entries_From_Stored(file, &stored.entries, &store, &slots)?;
    let dependents = Dependents_From_Stored(file, &stored.dependents, &slots)?;

    store.entries = entries;
    store.dependents = dependents;
    store.materializations = stored.materializations;

    return Ok(store);
}

/// Interns every key the file carries, refusing the file if this build does not address one
/// of them the way the writing build did.
fn Interned_Slots(
    file: &Path,
    store: &mut MemoryFactStore,
    stored: &[StoredKey],
) -> Result<BTreeMap<Digest128, FactSlot>, PersistenceError>
{
    let mut slots = BTreeMap::new();

    for written in stored
    {
        let key = Key_From_Stored(written);
        let recomputed = key.Digest();
        if recomputed != written.digest
        {
            return Err(PersistenceError::ForeignKeyDigest {
                file: file.to_path_buf(),
                written: written.digest,
                recomputed,
            });
        }
        slots.insert(recomputed, store.Slot_For(&key));
    }

    return Ok(slots);
}

/// Rebuilds each key's retained history, in the order the file lists it, as one row per
/// interned slot.
fn Entries_From_Stored(
    file: &Path,
    stored: &[StoredEntry],
    store: &MemoryFactStore,
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<Vec<Vec<Entry>>, PersistenceError>
{
    let mut entries: Vec<Vec<Entry>> =
        (0..store.identities.Count()).map(|_| return Vec::new()).collect();

    for written in stored
    {
        let slot = Named_Slot(file, written.key, slots)?;
        let entry = Entry_From_Stored(file, written, store, slots)?;
        if let Some(history) = entries.get_mut(slot.Position())
        {
            history.push(entry);
        }
    }

    return Ok(entries);
}

/// One entry, with the fact and the dependencies it recorded.
fn Entry_From_Stored(
    file: &Path,
    stored: &StoredEntry,
    store: &MemoryFactStore,
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<Entry, PersistenceError>
{
    let key = Named_Key(file, stored.key, store, slots)?;

    return Ok(Entry {
        fact: Fact_From_Stored(stored, key),
        invalidated_at: stored.invalidated_at,
        cause: stored.cause.clone(),
        dependencies: Dependencies_From_Stored(file, &stored.dependencies, store, slots)?,
    });
}

/// The slot a digest names, or a refusal: a file that references a key it does not carry is
/// damaged, not merely foreign.
fn Named_Slot(
    file: &Path,
    digest: Digest128,
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<FactSlot, PersistenceError>
{
    return slots.get(&digest).copied().ok_or_else(|| {
        return PersistenceError::Corrupt {
            file: file.to_path_buf(),
            detail: format!("an entry names key {digest}, which the file does not carry"),
        };
    });
}

/// The key a digest names, read back out of the interner the slots were minted in.
///
/// Two refusals rather than one lookup, because they are different damage: the file may name
/// a key it does not carry, which [`Named_Slot`] refuses, or the interner may not hold a slot
/// it just minted, which cannot happen and is written as a refusal rather than an index so
/// that the impossible case cannot panic on the analysis path.
fn Named_Key<'store>(
    file: &Path,
    digest: Digest128,
    store: &'store MemoryFactStore,
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<&'store FactKey, PersistenceError>
{
    let slot = Named_Slot(file, digest, slots)?;

    return store.identities.Key_Of(slot).ok_or_else(|| {
        return PersistenceError::Corrupt {
            file: file.to_path_buf(),
            detail: format!("key {digest} was interned and then could not be read back"),
        };
    });
}

fn Fact_From_Stored(stored: &StoredEntry, key: &FactKey) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(stored.generation),
        snapshot: stored.snapshot,
        evidence: stored.evidence,
        guarantee: stored.guarantee,
        payload: FactPayload::New(stored.schema.clone(), stored.payload.clone()),
    };
}

fn Dependencies_From_Stored(
    file: &Path,
    stored: &[StoredDependency],
    store: &MemoryFactStore,
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<Vec<Dependency>, PersistenceError>
{
    let mut dependencies = Vec::new();

    for written in stored
    {
        dependencies.push(Dependency {
            key: Named_Key(file, written.key, store, slots)?.clone(),
            outcome: written.outcome.Outcome(),
        });
    }

    return Ok(dependencies);
}

/// The dependents graph, edge by edge, with every digest resolved to the slot the running
/// store addresses it by.
///
/// An edge naming a key the file does not carry is refused as corruption rather than dropped.
/// [`super::MemoryFactStore::Materialize`] interns both ends of every edge it records, so no
/// store this build wrote can carry one; a file that does is damaged, and the alternative —
/// dropping the edge — would make a reloaded store invalidate less than the store that wrote
/// it, which is the one disagreement `OD-ANALYSIS-009` asks this not to have.
fn Dependents_From_Stored(
    file: &Path,
    stored: &[StoredDependents],
    slots: &BTreeMap<Digest128, FactSlot>,
) -> Result<BTreeMap<FactSlot, BTreeSet<FactSlot>>, PersistenceError>
{
    let mut dependents: BTreeMap<FactSlot, BTreeSet<FactSlot>> = BTreeMap::new();

    for written in stored
    {
        let read = Named_Slot(file, written.read, slots)?;
        let mut by: BTreeSet<FactSlot> = BTreeSet::new();
        for consumer in &written.by
        {
            by.insert(Named_Slot(file, *consumer, slots)?);
        }
        dependents.insert(read, by);
    }

    return Ok(dependents);
}

fn Stored_From_Store(store: &MemoryFactStore) -> StoredStore
{
    let addressed = Addressed_By_Digest(store);

    return StoredStore {
        format_version: FORMAT_VERSION,
        key_shape: Understood_Key_Shape(),
        materializations: store.materializations,
        keys: Stored_Keys(store, &addressed),
        entries: Stored_Entries(store, &addressed),
        dependents: Stored_Dependents(store),
    };
}

/// Every identity the store holds, in digest order.
///
/// Sorted rather than taken in the order the store interned them, and that is what keeps the
/// file a function of what the store holds rather than of the order one process was shown
/// it: two processes that materialized the same facts in different orders write the same
/// file, and a store written, reloaded and written again writes the same bytes. A digest is
/// also the only one of the two addresses that means anything to the next process.
fn Addressed_By_Digest(store: &MemoryFactStore) -> Vec<(Digest128, FactSlot)>
{
    let mut addressed: Vec<(Digest128, FactSlot)> = store
        .identities
        .Iter()
        .filter_map(|(slot, _)| return Some((store.identities.Digest_Of(slot)?, slot)))
        .collect();
    addressed.sort_unstable();

    return addressed;
}

fn Stored_Keys(store: &MemoryFactStore, addressed: &[(Digest128, FactSlot)]) -> Vec<StoredKey>
{
    let mut written = Vec::new();

    for (digest, slot) in addressed
    {
        if let Some(key) = store.identities.Key_Of(*slot)
        {
            written.push(Stored_Key(*digest, key));
        }
    }

    return written;
}

fn Stored_Entries(store: &MemoryFactStore, addressed: &[(Digest128, FactSlot)]) -> Vec<StoredEntry>
{
    let mut written = Vec::new();

    for (digest, slot) in addressed
    {
        let Some(history) = store.entries.get(slot.Position())
        else
        {
            continue;
        };
        written.extend(history.iter().map(|entry| return Stored_Entry(*digest, entry)));
    }

    return written;
}

/// The dependents graph as digests, in digest order on both ends.
///
/// Sorted for the reason [`Addressed_By_Digest`] is: the running store holds these edges in
/// slot order, which is the order one process interned in, and a file that carried that
/// order would differ between two processes holding identical stores.
fn Stored_Dependents(store: &MemoryFactStore) -> Vec<StoredDependents>
{
    let mut written: Vec<StoredDependents> = Vec::new();

    for (read, by) in &store.dependents
    {
        if let Some(digest) = store.identities.Digest_Of(*read)
        {
            written.push(StoredDependents { read: digest, by: Sorted_Digests(store, by) });
        }
    }
    written.sort_unstable_by_key(|edge| return edge.read);

    return written;
}

fn Sorted_Digests(store: &MemoryFactStore, slots: &BTreeSet<FactSlot>) -> Vec<Digest128>
{
    let mut digests: Vec<Digest128> =
        slots.iter().filter_map(|slot| return store.identities.Digest_Of(*slot)).collect();
    digests.sort_unstable();

    return digests;
}

fn Stored_Key(digest: Digest128, key: &FactKey) -> StoredKey
{
    return StoredKey {
        digest,
        contract: key.contract.clone(),
        contract_version: key.contract_version,
        subject: key.subject,
        semantic_inputs: key.semantic_inputs.Digest(),
        provider: key.provider.clone(),
        provider_version: key.provider_version,
        guarantee: key.guarantee.Digest(),
        variant: key.variant,
        configuration: key.configuration,
    };
}

fn Key_From_Stored(stored: &StoredKey) -> FactKey
{
    return FactKey {
        contract: stored.contract.clone(),
        contract_version: stored.contract_version,
        subject: stored.subject,
        semantic_inputs: InputDigest::From_Digest(stored.semantic_inputs),
        provider: stored.provider.clone(),
        provider_version: stored.provider_version,
        guarantee: GuaranteeDigest::From_Digest(stored.guarantee),
        variant: stored.variant,
        configuration: stored.configuration,
    };
}

fn Stored_Entry(key: Digest128, entry: &Entry) -> StoredEntry
{
    return StoredEntry {
        key,
        generation: entry.fact.Generation(),
        snapshot: entry.fact.snapshot,
        evidence: entry.fact.evidence,
        guarantee: entry.fact.guarantee,
        schema: entry.fact.payload.schema.clone(),
        payload: entry.fact.payload.bytes.clone(),
        invalidated_at: entry.invalidated_at,
        cause: entry.cause.clone(),
        dependencies: entry
            .dependencies
            .iter()
            .map(|dependency| {
                return StoredDependency {
                    key: dependency.key.Digest(),
                    outcome: StoredOutcome::Of(dependency.outcome),
                };
            })
            .collect(),
    };
}

fn Unwritable(file: &Path, error: &dyn core::fmt::Display) -> PersistenceError
{
    return PersistenceError::Unwritable { file: file.to_path_buf(), detail: error.to_string() };
}

fn Unreadable(file: &Path, error: &dyn core::fmt::Display) -> PersistenceError
{
    return PersistenceError::Unreadable { file: file.to_path_buf(), detail: error.to_string() };
}

fn Corrupt(file: &Path, error: &dyn core::fmt::Display) -> PersistenceError
{
    return PersistenceError::Corrupt { file: file.to_path_buf(), detail: error.to_string() };
}

/// The file `directory` holds a store in, for a test that has to damage or inspect it.
#[cfg(test)]
pub(super) fn Store_File(directory: &Path) -> std::path::PathBuf
{
    return directory.join(STORE_FILE_NAME);
}

/// The written form of the whole store.
///
/// A private mirror rather than `serde` derived onto [`MemoryFactStore`]'s own vocabulary,
/// deliberately. The public types say what a fact *is* to a caller in this process; this
/// says what one *was written as*, and the two are allowed to move apart — a derive would
/// make every rename of a field a silent format change, and would put `Serialize` on this
/// crate's published surface where nothing asked for it.
#[derive(Serialize, Deserialize)]
struct StoredStore
{
    format_version: u32,
    key_shape: Vec<String>,
    materializations: u32,
    keys: Vec<StoredKey>,
    entries: Vec<StoredEntry>,
    dependents: Vec<StoredDependents>,
}

/// One key, written whole, beside the digest the writing build addressed it by.
#[derive(Serialize, Deserialize)]
struct StoredKey
{
    digest: Digest128,
    contract: CapabilityId,
    contract_version: ContractVersion,
    subject: SubjectId,
    semantic_inputs: Digest128,
    provider: ProviderId,
    provider_version: ContractVersion,
    guarantee: Digest128,
    variant: BuildVariantId,
    configuration: ConfigurationId,
}

/// One retained entry of one key's history.
#[derive(Serialize, Deserialize)]
struct StoredEntry
{
    key: Digest128,
    generation: GenerationId,
    snapshot: SnapshotId,
    evidence: EvidenceClass,
    guarantee: Guarantee,
    schema: SchemaId,
    payload: Vec<u8>,
    invalidated_at: Option<GenerationId>,
    cause: Option<String>,
    dependencies: Vec<StoredDependency>,
}

/// One edge an entry recorded, by the digest of the key it read.
#[derive(Serialize, Deserialize)]
struct StoredDependency
{
    key: Digest128,
    outcome: StoredOutcome,
}

/// One row of the dependents graph: a key that was read, and every key that read it.
#[derive(Serialize, Deserialize)]
struct StoredDependents
{
    read: Digest128,
    by: Vec<Digest128>,
}

/// [`ReadOutcome`]'s written mirror, converted by an exhaustive match in each direction so
/// a variant added there fails to compile here rather than being written as another one.
#[derive(Clone, Copy, Serialize, Deserialize)]
enum StoredOutcome
{
    Materialized,
    Absent,
    Superseded,
    Degraded(Applicability),
}

impl StoredOutcome
{
    fn Of(outcome: ReadOutcome) -> Self
    {
        return match outcome
        {
            ReadOutcome::Materialized => Self::Materialized,
            ReadOutcome::Absent => Self::Absent,
            ReadOutcome::Superseded => Self::Superseded,
            ReadOutcome::Degraded(applicability) => Self::Degraded(applicability),
        };
    }

    const fn Outcome(self) -> ReadOutcome
    {
        return match self
        {
            Self::Materialized => ReadOutcome::Materialized,
            Self::Absent => ReadOutcome::Absent,
            Self::Superseded => ReadOutcome::Superseded,
            Self::Degraded(applicability) => ReadOutcome::Degraded(applicability),
        };
    }
}

// The behavioural suite — round trips, the refusals, and the two equivalence proofs — is a
// separate file. This inline module is the one that can address the functions declared
// above, because check-test-coverage's Rust front end keys a test's companion unit off the
// literal file it is written in; `memory_fact_store.rs` carries the same pair for the same
// reason.
#[cfg(test)]
mod local_tests
{
    use super::*;

    /// The subject seed every sample key in this module is built from.
    const SAMPLE_SEED: u8 = 1;

    /// The variant component of the sample key, seeded apart from the subject and the
    /// configuration so a key assembled with the wrong one still asserts unequal.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of the sample key; distinct from [`VARIANT_SEED`] for the
    /// reason given there.
    const CONFIGURATION_SEED: u8 = 4;

    #[test]
    fn Test_Understood_Key_Shape_Should_Name_Every_Part_A_Key_Digest_Is_Built_From()
    {
        let shape = Understood_Key_Shape();

        assert_eq!(
            shape.len(),
            Sample_Key().Parts().len(),
            "the written key shape and the parts a key is addressed by have drifted apart"
        );
        assert!(shape.contains(&"contract".to_owned()));
    }

    #[test]
    fn Test_Refuse_Foreign_Header_Should_Refuse_A_Format_Version_This_Build_Does_Not_Know()
    {
        let mut stored = Empty_Stored();
        stored.format_version = FORMAT_VERSION.saturating_add(1);

        let refused = Refuse_Foreign_Header(Path::new(STORE_FILE_NAME), &stored);

        assert!(matches!(refused, Err(PersistenceError::ForeignFormat { .. })));
    }

    #[test]
    fn Test_Refuse_Foreign_Header_Should_Refuse_A_Key_Shape_This_Build_Does_Not_Know()
    {
        let mut stored = Empty_Stored();
        stored.key_shape.push("snapshot".to_owned());

        let refused = Refuse_Foreign_Header(Path::new(STORE_FILE_NAME), &stored);

        assert!(matches!(refused, Err(PersistenceError::ForeignKeyShape { .. })));
    }

    #[test]
    fn Test_Refuse_Foreign_Header_Should_Accept_This_Builds_Own_Header()
    {
        assert!(Refuse_Foreign_Header(Path::new(STORE_FILE_NAME), &Empty_Stored()).is_ok());
    }

    #[test]
    fn Test_Refuse_Unknown_Schemas_Should_Refuse_A_Schema_The_Reader_Did_Not_Declare()
    {
        let entries = vec![Stored_Entry_With_Schema(SchemaId::New("nomos.test.persistence.v1"))];

        let refused = Refuse_Unknown_Schemas(Path::new(STORE_FILE_NAME), &entries, &[]);

        assert!(matches!(refused, Err(PersistenceError::UnknownPayloadSchema { .. })));
    }

    #[test]
    fn Test_Refuse_Unknown_Schemas_Should_Accept_A_Schema_The_Reader_Declared()
    {
        let schema = SchemaId::New("nomos.test.persistence.v1");
        let entries = vec![Stored_Entry_With_Schema(schema.clone())];

        assert!(Refuse_Unknown_Schemas(Path::new(STORE_FILE_NAME), &entries, &[schema]).is_ok());
    }

    #[test]
    fn Test_Interned_Slots_Should_Refuse_A_Key_This_Build_Addresses_Differently()
    {
        let key = Sample_Key();
        let mut written = Stored_Key(key.Digest(), &key);
        written.digest = Digest128::From_Bytes([SAMPLE_SEED; Digest128::BYTE_LENGTH]);
        let mut store = MemoryFactStore::New();

        let refused = Interned_Slots(Path::new(STORE_FILE_NAME), &mut store, &[written]);

        assert!(matches!(refused, Err(PersistenceError::ForeignKeyDigest { .. })));
    }

    /// The other half of the same question: a key this build addresses the way the file says
    /// gets a slot, and the digest the interner answers with is the one the file carried.
    #[test]
    fn Test_Interned_Slots_Should_Address_A_Key_The_File_Agrees_About()
    {
        let key = Sample_Key();
        let written = Stored_Key(key.Digest(), &key);
        let mut store = MemoryFactStore::New();

        let slots = Interned_Slots(Path::new(STORE_FILE_NAME), &mut store, &[written])
            .expect("this build addresses the key exactly as the file says it does");

        let slot = *slots.get(&key.Digest()).expect("the key was interned under its own digest");
        assert_eq!(store.identities.Key_Of(slot), Some(&key));
    }

    #[test]
    fn Test_Key_From_Stored_Should_Return_The_Key_Stored_Key_Was_Built_From()
    {
        let key = Sample_Key();

        assert_eq!(Key_From_Stored(&Stored_Key(key.Digest(), &key)), key);
    }

    #[test]
    fn Test_Named_Slot_Should_Refuse_A_Digest_The_File_Does_Not_Carry()
    {
        let digest = Digest128::From_Bytes([SAMPLE_SEED; Digest128::BYTE_LENGTH]);
        let carries_nothing = BTreeMap::new();

        let refused = Named_Slot(Path::new(STORE_FILE_NAME), digest, &carries_nothing);

        assert!(matches!(refused, Err(PersistenceError::Corrupt { .. })));
    }

    #[test]
    fn Test_Named_Key_Should_Refuse_A_Digest_The_File_Does_Not_Carry()
    {
        let digest = Digest128::From_Bytes([SAMPLE_SEED; Digest128::BYTE_LENGTH]);
        let carries_nothing = BTreeMap::new();
        let store = MemoryFactStore::New();

        let refused = Named_Key(Path::new(STORE_FILE_NAME), digest, &store, &carries_nothing);

        assert!(matches!(refused, Err(PersistenceError::Corrupt { .. })));
    }

    #[test]
    fn Test_Stored_Outcome_Should_Round_Trip_Every_Read_Outcome()
    {
        let every = [
            ReadOutcome::Materialized,
            ReadOutcome::Absent,
            ReadOutcome::Superseded,
            ReadOutcome::Degraded(Applicability::SupportedWithFallback),
        ];

        for outcome in every
        {
            assert_eq!(StoredOutcome::Of(outcome).Outcome(), outcome);
        }
    }

    #[test]
    fn Test_Store_File_Should_Name_The_File_Inside_The_Directory_Given()
    {
        assert_eq!(Store_File(Path::new("store")), Path::new("store").join(STORE_FILE_NAME));
    }

    #[test]
    fn Test_Dependents_From_Stored_Should_Rebuild_Every_Row()
    {
        let key = Sample_Key();
        let read = key.Digest();
        let mut store = MemoryFactStore::New();
        let slots = Interned_Slots(Path::new(STORE_FILE_NAME), &mut store, &[Stored_Key(read, &key)])
            .expect("this build addresses the key exactly as the file says it does");
        let slot = *slots.get(&read).expect("the key was interned under its own digest");

        let rebuilt = Dependents_From_Stored(
            Path::new(STORE_FILE_NAME),
            &[StoredDependents { read, by: vec![read] }],
            &slots,
        )
        .expect("every digest in the row is a key the file carries");

        assert_eq!(rebuilt.get(&slot), Some(&BTreeSet::from([slot])));
    }

    /// An edge naming a key the file does not carry is corruption, not an edge to drop: a
    /// reloaded store that quietly lost it would invalidate less than the store that wrote
    /// it.
    #[test]
    fn Test_Dependents_From_Stored_Should_Refuse_An_Edge_Naming_A_Key_The_File_Does_Not_Carry()
    {
        let absent = Digest128::From_Bytes([SAMPLE_SEED; Digest128::BYTE_LENGTH]);

        let refused = Dependents_From_Stored(
            Path::new(STORE_FILE_NAME),
            &[StoredDependents { read: absent, by: vec![absent] }],
            &BTreeMap::new(),
        );

        assert!(matches!(refused, Err(PersistenceError::Corrupt { .. })));
    }

    #[test]
    fn Test_Refusal_Builders_Should_Each_Name_The_File_And_Carry_The_Reason()
    {
        let file = Path::new(STORE_FILE_NAME);
        let reason = "no such file";

        for refusal in [Unwritable(file, &reason), Unreadable(file, &reason), Corrupt(file, &reason)]
        {
            assert_eq!(refusal.File(), file);
            assert!(refusal.to_string().contains(reason), "{refusal:?} dropped the reason");
        }
    }

    fn Empty_Stored() -> StoredStore
    {
        return StoredStore {
            format_version: FORMAT_VERSION,
            key_shape: Understood_Key_Shape(),
            materializations: 0,
            keys: Vec::new(),
            entries: Vec::new(),
            dependents: Vec::new(),
        };
    }

    fn Stored_Entry_With_Schema(schema: SchemaId) -> StoredEntry
    {
        let key = Sample_Key();
        let fact = MaterializedFact {
            identity: key.clone().At(GenerationId::INITIAL),
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes(
                [SAMPLE_SEED; Digest128::BYTE_LENGTH],
            )),
            evidence: EvidenceClass::Derived,
            guarantee: Sample_Guarantee(),
            payload: FactPayload::New(schema, Vec::new()),
        };

        return Stored_Entry(
            key.Digest(),
            &Entry { fact, invalidated_at: None, cause: None, dependencies: Vec::new() },
        );
    }

    fn Sample_Key() -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.persistence"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded_Digest(SAMPLE_SEED)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Sample_Guarantee()),
            variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
        };
    }

    fn Sample_Guarantee() -> Guarantee
    {
        use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }
}

#[cfg(test)]
#[path = "persistence/tests.rs"] mod tests;
