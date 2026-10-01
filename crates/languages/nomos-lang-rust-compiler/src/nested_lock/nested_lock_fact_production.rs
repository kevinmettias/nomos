//! What this provider promises about repeating itself, for its second capability, stated
//! as a declaration and checked against what it actually does.
//!
//! [`crate::nested_lock_guarantee::Declared_Guarantee`] says how good this provider's
//! answer is. This says whether asking twice yields the same answer. The verification
//! owed is discharged the same seam `crate::clone_on_copy_fact_production` is discharged
//! through, per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.rust.nested_locks` fact for one crate: loading it through
/// `ra_ap_hir`, resolving `std::sync::Mutex`/`std::sync::RwLock` against the loaded
/// standard library, and encoding every `Mutex<T>`/`RwLock<T>` its analysis resolved `T`
/// to be another lock type.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `crate::clone_on_copy_fact_production`'s own `CloneOnCopyFactProduction`.
pub struct NestedLockFactProduction;

impl Strategy for NestedLockFactProduction
{
    /// `State`, not `StateTemporal`, the same reasoning
    /// `CloneOnCopyFactProduction::STRENGTH` already gives: [`crate::nested_lock_reading::Discover_Nested_Locks`]
    /// sorts every finding by its own rendered location before this provider ever
    /// encodes them, so two runs that discovered the same findings through `Vfs::iter`'s
    /// own order-agnostic iteration reach identical bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`, for the identical reason `CloneOnCopyFactProduction::SCOPE`
    /// already gives: `ra_ap_project_model::RustLibSource::Discover` resolves a sysroot
    /// path that is not the same string on every host, and this provider's own
    /// resolution of `std::sync::Mutex`/`RwLock` (`Lock_Structs`) walks that same loaded
    /// sysroot, so it carries the identical cross-platform risk, not a new one.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
