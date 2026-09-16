//! The walk an invalidation spreads through: naming what a cause reaches directly, propagating
//! to what depends on it, noting where a provider's guarantee broadened it, and settling the
//! report into the one order two runs over one store both produce.
//!
//! Split out of `memory_store.rs` by responsibility: [`super::MemoryFactStore`]'s storage
//! surface (materializing, reading, invalidating one entry) is one concern, and this — what
//! [`super::MemoryFactStore::Invalidate`] actually does once it has decided to invalidate
//! something — is another. Everything here is `pub(super)` or private; the type these
//! functions operate on, and the trait surface that calls into [`Invalidate`], stay in
//! `memory_store.rs` because that is the one file the crate's public-surface reader resolves
//! `MemoryFactStore` against.

use std::collections::BTreeSet;
use nomos_contracts::IncrementalGranularity;
use nomos_contracts::Digest128;
use nomos_contracts::GenerationId;
use crate::InvalidationReport;
use crate::GenerationCause;

use super::MemoryFactStore;

/// The body of [`super::MemoryFactStore::Invalidate`], which keeps the trait's documentation
/// and signature.
pub(super) fn Invalidate_Reached(store: &mut MemoryFactStore, cause: &GenerationCause, from: GenerationId) -> InvalidationReport
{
    let described = cause.Describe();
    let invalidating = Invalidating { from, described: &described };
    let mut report = Opened_Report(cause, from);
    let roots = Invalidate_Named(store, cause, invalidating, &mut report);

    let seen = Propagate_To_Dependents(store, roots, invalidating, &mut report);

    Note_Broadening(store, cause.Granularity(), &seen, &mut report);
    Settle_Report(store, &mut report);

    return report;
}

/// An empty report of what this cause is about to invalidate.
fn Opened_Report(cause: &GenerationCause, from: GenerationId) -> InvalidationReport
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

/// Invalidates every fact the cause names directly, and returns them as the frontier.
fn Invalidate_Named(
    store: &mut MemoryFactStore,
    cause: &GenerationCause,
    invalidating: Invalidating<'_>,
    report: &mut InvalidationReport,
) -> Vec<Digest128>
{
    let named: Vec<Digest128> = store
        .keys
        .iter()
        .filter(|(_, key)| return cause.Is_Naming(key))
        .map(|(digest, _)| return *digest)
        .collect();

    let mut frontier: Vec<Digest128> = Vec::new();
    for digest in named
    {
        if !store.Try_Invalidate_One(digest, invalidating.from, invalidating.described)
        {
            continue;
        }
        if let Some(key) = store.keys.get(&digest)
        {
            report.direct.push(key.clone());
        }
        frontier.push(digest);
    }

    return frontier;
}

/// Spreads invalidation from `roots` through `store.dependents`, recording each reached
/// dependent's key into `report`. Returns every digest visited, roots included.
///
/// Two passes, not one, and the split is why this reads `store.dependents` directly rather
/// than cloning it first (`OD-ANALYSIS-008`, whose cost was `O(the whole store's
/// dependents map)` on every call, not just what a given invalidation's frontier actually
/// reaches). The first pass borrows `store` immutably to walk `dependents` and decide,
/// through [`MemoryFactStore::Is_Already_Invalidated`] -- the read-only half of what
/// [`MemoryFactStore::Try_Invalidate_One`] checks before it mutates -- which nodes to keep
/// spreading past and to collect. Nothing mutates during that pass, so the immutable
/// borrow `Spread` needs for `dependents` coexists with the immutable reads
/// `Is_Already_Invalidated` needs; neither coexists with the mutation `Try_Invalidate_One` needs,
/// which is exactly why the original clone existed. Only the second pass, after the walk
/// and its borrow have ended, calls `Try_Invalidate_One` and writes into `report`.
///
/// The split is sound because a digest is visited exactly once per call -- the trait's own
/// invariant, pinned by `propagation.rs`'s own suite -- so no digest's `Is_Already_Invalidated`
/// read in the first pass can be stale from a mutation this same call made to a *different*
/// digest: invalidating one digest's entry never touches another digest's entry.
fn Propagate_To_Dependents(
    store: &mut MemoryFactStore,
    roots: Vec<Digest128>,
    invalidating: Invalidating<'_>,
    report: &mut InvalidationReport,
) -> BTreeSet<Digest128>
{
    let walk = Walked_Dependents(store, roots);

    for consumer in walk.reached
    {
        Apply_To_Consumer(store, consumer, invalidating, report);
    }

    return walk.seen;
}

/// The first pass of [`Propagate_To_Dependents`]'s walk.
// Reaches the strategy through `store.propagation` rather than taking it as a parameter, and
// reads it by shared borrow: the walk only reads `store` (`dependents`, `keys` through
// `Is_Already_Invalidated`), so the borrow it holds ends with the returned [`Walk`], which is
// owned, and the second pass's `&mut store` is free to start then.
fn Walked_Dependents(store: &MemoryFactStore, roots: Vec<Digest128>) -> Walk
{
    let mut seen: BTreeSet<Digest128> = roots.iter().copied().collect();
    let mut reached: Vec<Digest128> = Vec::new();

    store.propagation.Spread(&store.dependents, roots, &mut |consumer| {
        seen.insert(consumer);
        if store.Is_Already_Invalidated(consumer)
        {
            return false;
        }
        reached.push(consumer);

        return true;
    });

    return Walk { seen, reached };
}

/// [`Walked`]'s own result: every digest visited (`seen`, roots included), and, in visit
/// order, those reached but not yet invalidated (`reached`) -- the frontier the second pass
/// still has to apply. Named so the two `BTreeSet`/`Vec` results are not told apart only by
/// position.
struct Walk
{
    seen: BTreeSet<Digest128>,
    reached: Vec<Digest128>,
}

/// One node the first pass decided to keep: invalidated for real, and — if it was live —
/// named in `report`.
fn Apply_To_Consumer(store: &mut MemoryFactStore, consumer: Digest128, invalidating: Invalidating<'_>, report: &mut InvalidationReport)
{
    if !store.Try_Invalidate_One(consumer, invalidating.from, invalidating.described)
    {
        return;
    }
    if let Some(key) = store.keys.get(&consumer)
    {
        report.dependent.push(key.clone());
    }
}

/// Every invalidated fact whose own guarantee is coarser than the cause was.
///
/// Reported rather than silently applied: a provider that can only answer at whole-
/// workspace granularity turns a one-file edit into a full rebuild, and the caller is
/// entitled to know which provider did that and to how many facts.
fn Note_Broadening(
    store: &MemoryFactStore,
    requested: IncrementalGranularity,
    seen: &BTreeSet<Digest128>,
    report: &mut InvalidationReport,
)
{
    use crate::Broadening;

    for digest in seen
    {
        let (Some(key), Some(entry)) = (store.keys.get(digest), store.Latest(*digest))
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

/// Puts the report into the one order two runs over one store both produce.
///
/// The retained count is taken last, after everything the cause reaches has been
/// invalidated, because it is the answer to "what survived" and not to "what was here".
fn Settle_Report(store: &MemoryFactStore, report: &mut InvalidationReport)
{
    report.direct.sort();
    report.dependent.sort();
    report.broadened.sort_by(|first, second| return first.key.cmp(&second.key));
    report.retained = u32::try_from(store.Live()).unwrap_or(u32::MAX);
}

/// The generation and description a walk is invalidating under, threaded through
/// [`Reached`] as one value so the function stays under this crate's own parameter-count
/// ceiling.
#[derive(Clone, Copy)]
struct Invalidating<'a>
{
    from: GenerationId,
    described: &'a str,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, EvidenceClass,
        FactVariant, Guarantee, ProviderId, SchemaId, SnapshotId, SubjectId,
    };

    /// The generation the invalidation under test is applied at. Ahead of the generation the
    /// fact was materialized at (the exempt `1`), because a cause cannot invalidate a fact the
    /// store has not been told about yet.
    const INVALIDATION_GENERATION: u64 = 2;

    /// The variant component of [`Key_For`]'s key, seeded apart from that key's configuration
    /// and its subject so a fixture that mixes two of them up still asserts unequal.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of [`Key_For`]'s key; distinct from [`VARIANT_SEED`] for
    /// the reason given there.
    const CONFIGURATION_SEED: u8 = 4;

    /// The workspace state [`Fact_For`] measures its fact against. Provenance rather than part
    /// of the fact's identity (`OD-ANALYSIS-001`), so one value serves every fixture here.
    const SNAPSHOT_SEED: u8 = 2;

    #[test]
    fn Test_Invalidate_Reached_Should_Name_The_Fact_A_Cause_Reaches_Directly()
    {
        let mut store = MemoryFactStore::New();
        let key = Key_For(1);
        let fact = Fact_For(&key, GenerationId::From_Raw(1));
        store
            .Materialize(fact, &[])
            .expect("first materialization cannot conflict");

        let cause = GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::File,
        };

        let report = Invalidate_Reached(&mut store, &cause, GenerationId::From_Raw(INVALIDATION_GENERATION));

        assert_eq!(report.direct, vec![key]);
        assert_eq!(report.retained, 0);
    }

    fn Key_For(subject_seed: u8) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.invalidation"),
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
            payload: FactPayload::New(SchemaId::New("nomos.test.invalidation.v1"), b"tree".to_vec()),
        };
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
