//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism.rs`, the same seam `nomos_lang_rust::SyntaxFactProduction`
//! is discharged through, and for the same reason: a declaration verified only against
//! the real corpus is proven nowhere the gate can see, per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.dependency.edges` fact for a workspace: running `cargo
/// metadata` and encoding what it reports.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `nomos_lang_rust::SyntaxFactProduction` and `nomos_lang_rust_scan::ScanFactProduction`.
pub struct DependencyFactProduction;

impl Strategy for DependencyFactProduction
{
    /// `State`, not `StateTemporal`. `metadata::Dependency_Edges` sorts every package's
    /// edges canonically before this provider ever encodes them, so two runs that
    /// discovered the same edges through cargo's own JSON in a different order — which
    /// `serde_json`'s object representation does not promise against — reach identical
    /// bytes. Nothing about *when* an edge was observed is part of what this fact claims;
    /// only the final, sorted set is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what
    /// `Discover_Workspace`'s own tests already exercise implicitly and what this
    /// declaration commits to verifying explicitly. `CrossPlatform` would require a
    /// golden digest captured on a second real platform this crate has not been run on —
    /// claiming it now would be exactly the overclaim `docs/records/OD-ANALYSIS-004`
    /// warns against elsewhere: a declared strength nobody has exercised is a comment,
    /// not a promise.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
