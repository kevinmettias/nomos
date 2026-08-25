//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism`, the same seam
//! `nomos_lang_rust_cargo::DependencyFactProduction` is discharged through, and for the
//! same reason: a declaration verified only against a real fixture is proven nowhere the
//! gate can see, per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.dependency.edges` fact for a Go workspace: reading
/// `go.work`/`go.mod` text and encoding what it declares.
///
/// A second, distinct `Strategy` type from `nomos_lang_rust_cargo::DependencyFactProduction`
/// rather than a reuse of it, the same way `nomos_lang_go::SyntaxFactProduction` is its own
/// type rather than a reuse of `nomos_lang_rust::SyntaxFactProduction`: this is a different
/// provider crate, with its own real read path to measure, even though both providers
/// answer the same capability.
pub struct DependencyFactProduction;

impl Strategy for DependencyFactProduction
{
    /// `State`, not `StateTemporal`. `discovery::Discovered` sorts every module's edges
    /// canonically before this provider ever encodes them, so two runs that read the same
    /// `require` lines in a different order — which nothing here promises against, since a
    /// `go.mod`'s own line order is an authoring detail — reach identical bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// tests already exercise, and `CrossPlatform` would need a golden digest captured on
    /// a second real platform this crate has not been run on — an unexercised claim is a
    /// comment, not a promise, the identical reasoning
    /// `nomos_lang_rust_cargo::DependencyFactProduction`'s own doc gives.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
