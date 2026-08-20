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
use std::collections::BTreeMap;
use nomos_contracts::IncrementalGranularity;
use nomos_contracts::Digest128;
use nomos_contracts::GenerationId;
use crate::InvalidationReport;
use crate::GenerationCause;
use crate::propagation::DependencyPropagation;

use super::MemoryFactStore;

/// The body of [`super::MemoryFactStore::Invalidate`], which keeps the trait's documentation
/// and signature.
pub(super) fn Invalidate(store: &mut MemoryFactStore, cause: &GenerationCause, from: GenerationId) -> InvalidationReport
{
    let described = cause.Describe();
    let mut report = Opened(cause, from);
    let roots = Invalidate_Named(store, cause, from, &described, &mut report);

    let seen = Propagate(store, roots, from, &described, &mut report);

    Note_Broadening(store, cause.Granularity(), &seen, &mut report);
    Settle(store, &mut report);

    return report;
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

/// Invalidates every fact the cause names directly, and returns them as the frontier.
fn Invalidate_Named(
    store: &mut MemoryFactStore,
    cause: &GenerationCause,
    from: GenerationId,
    described: &str,
    report: &mut InvalidationReport,
) -> Vec<Digest128>
{
    let named: Vec<Digest128> = store
        .keys
        .iter()
        .filter(|(_, key)| return cause.Names(key))
        .map(|(digest, _)| return *digest)
        .collect();

    let mut frontier: Vec<Digest128> = Vec::new();
    for digest in named
    {
        if !store.Invalidate_One(digest, from, described)
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

/// Puts the report into the one order two runs over one store both produce.
///
/// The retained count is taken last, after everything the cause reaches has been
/// invalidated, because it is the answer to "what survived" and not to "what was here".
fn Settle(store: &MemoryFactStore, report: &mut InvalidationReport)
{
    report.direct.sort();
    report.dependent.sort();
    report.broadened.sort_by(|first, second| return first.key.cmp(&second.key));
    report.retained = u32::try_from(store.Live()).unwrap_or(u32::MAX);
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

/// Spreads invalidation from `roots` through `store.dependents`, recording each reached
/// dependent's key into `report`. Returns every digest visited, roots included.
fn Propagate(
    store: &mut MemoryFactStore,
    roots: Vec<Digest128>,
    from: GenerationId,
    described: &str,
    report: &mut InvalidationReport,
) -> BTreeSet<Digest128>
{
    let mut seen: BTreeSet<Digest128> = roots.iter().copied().collect();
    let (dependents, propagation) = Taken_Propagation(store);
    let invalidating = Invalidating { from, described };

    propagation.Spread(&dependents, roots, &mut |consumer| {
        return Reached(store, consumer, invalidating, &mut seen, report);
    });
    store.propagation = Some(propagation);

    return seen;
}

/// Takes `store.propagation` out, alongside a snapshot of `store.dependents`, so the walk
/// in [`Propagate`] can hold `&mut MemoryFactStore` inside its own callback without a live
/// borrow of either field conflicting with it.
///
/// Cloned once, and `propagation` taken out of `store`, rather than either held as a
/// borrow across the walk: the callback needs `&mut MemoryFactStore` for
/// [`MemoryFactStore::Invalidate_One`] and `store.keys`, which cannot coexist with a borrow
/// of `store.dependents` or `store.propagation` for the call that runs it.
/// `DependencyPropagation` has no Nomos-specific reason to know about that conflict — see
/// `docs/records/D-135` and `docs/records/D-138`.
fn Taken_Propagation(store: &mut MemoryFactStore) -> (BTreeMap<Digest128, BTreeSet<Digest128>>, Box<dyn DependencyPropagation>)
{
    let dependents = store.dependents.clone();
    let propagation = store
        .propagation
        .take()
        .expect("propagation implementation is always present between calls");

    return (dependents, propagation);
}

/// One node the walk reached: recorded into `seen`, invalidated, and — if it was live —
/// named in `report`. Returns whether the walk should continue past it.
fn Reached(
    store: &mut MemoryFactStore,
    consumer: Digest128,
    invalidating: Invalidating<'_>,
    seen: &mut BTreeSet<Digest128>,
    report: &mut InvalidationReport,
) -> bool
{
    seen.insert(consumer);
    if !store.Invalidate_One(consumer, invalidating.from, invalidating.described)
    {
        return false;
    }
    if let Some(key) = store.keys.get(&consumer)
    {
        report.dependent.push(key.clone());
    }

    return true;
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
