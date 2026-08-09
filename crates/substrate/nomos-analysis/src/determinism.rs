//! What the fact cache promises about repeating itself.
//!
//! The fact-cache row of the domain table in [`nomos_contracts::Strategy`]'s module. It
//! is the row with the weakest strength and the second-strongest scope, and both halves
//! of that shape are deliberate.
//!
//! Verified in `tests/integration/tests/determinism.rs`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Serving a fact from the store instead of producing it again.
///
/// The subject is reuse, not production. What a provider computes is
/// [`nomos_contracts::Strategy`]'s question one band down; what this domain promises is
/// that asking the store a second time yields what the first ask yielded, and that
/// invalidating and rebuilding lands back on the same answer.
pub struct FactReuse;

impl Strategy for FactReuse
{
    /// `State`, not `StateTemporal`, and the difference is not modesty.
    ///
    /// [`crate::MemoryFactStore`] is keyed by [`crate::FactKey`], and the order in which
    /// facts were materialized into it is not recoverable from it — deliberately, because
    /// a cache that remembered arrival order would make two workspaces that reached the
    /// same state by different routes into different caches. So the set of live facts is
    /// promised and the sequence is not, and claiming `StateTemporal` here would be a
    /// promise about an order this domain does not retain.
    ///
    /// [`crate::InvalidationReport`] is the one thing here that has an order, and it is
    /// sorted before it is reported rather than being emitted in traversal order — which
    /// is what makes the report diffable without the cache having to be temporal.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, which is what an incremental cache actually needs and no more.
    ///
    /// Two processes on one machine must agree, because that is the case a cache is for:
    /// a build, then another build. `CrossPlatform` is not claimed — not because it is
    /// believed false, but because nothing here has ever been compared across platforms
    /// and a scope is a promise about verification performed rather than about behaviour
    /// hoped for. The producers one band down do claim it, and are checked for it.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. A reused fact is byte-equal to the one it stands in for, or it is
    /// not the same fact — there is no tolerance under which a cache hit could be
    /// approximately the value it cached.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
