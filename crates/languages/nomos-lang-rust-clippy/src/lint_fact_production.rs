//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism/`, the same seam `nomos_lang_rust_cargo::
//! DependencyFactProduction` is discharged through, and for the same reason: a declaration
//! verified only against the real corpus is proven nowhere the gate can see, per
//! `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.lint.diagnostics` fact for a workspace: running `cargo
/// clippy --message-format=json` and encoding what it reports.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `nomos_lang_rust_cargo::DependencyFactProduction`.
pub struct LintFactProduction;

impl Strategy for LintFactProduction
{
    /// `State`, not `StateTemporal`. `reading::Canonical_Order` sorts and deduplicates
    /// every member's own diagnostics before this provider ever encodes them, so two runs
    /// that discovered the same diagnostics through `cargo clippy`'s own JSON stream in a
    /// different order — which neither `cargo`'s own scheduling nor `serde_json`'s object
    /// representation promises against — reach identical bytes. Nothing about *when* a
    /// diagnostic was reported is part of what this fact claims; only the final, sorted,
    /// deduplicated set is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// `Test_Discover_Workspace_And_Materialize_Workspace_Should_Find_Every_Real_Workspace_Member`
    /// already exercises implicitly and what this declaration commits to verifying
    /// explicitly.
    /// `CrossPlatform` would require a golden digest captured on a second real platform
    /// this crate has not been run on — claiming it now would be exactly the overclaim
    /// `docs/records/OD-ANALYSIS-004` warns against elsewhere.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
