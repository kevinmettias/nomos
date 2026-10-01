//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::guarantee::Declared_Guarantee`] says how good this provider's answer is.
//! This says whether asking twice yields the same answer. The verification owed is
//! discharged the same seam `nomos_lang_rust_deny::DependencyPolicyFactProduction` is
//! discharged through, per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.rust.copy_clones` fact for one crate: loading it through
/// `ra_ap_hir` and encoding every `.clone()` call its analysis resolved to a `Copy`
/// type.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `nomos_lang_rust_deny::DependencyPolicyFactProduction`.
pub struct CloneOnCopyFactProduction;

impl Strategy for CloneOnCopyFactProduction
{
    /// `State`, not `StateTemporal`. [`crate::reading::Discover_Crate`] sorts every
    /// finding by its own rendered location before this provider ever encodes them, so
    /// two runs that discovered the same findings through `Vfs::iter`'s own iteration
    /// order -- which promises nothing about the order files arrive in -- reach
    /// identical bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's
    /// own `Test_Discover_Crate_Should_Find_Exactly_The_Real_Clone_On_Copy_Call` already
    /// exercises implicitly and what this declaration commits to verifying explicitly.
    /// `CrossPlatform` would require a golden digest captured on a second real platform
    /// this crate has not been run on -- claiming it now would be the same overclaim
    /// `docs/records/OD-ANALYSIS-004` warns against elsewhere, and a real risk here
    /// specifically: `ra_ap_project_model::RustLibSource::Discover` resolves a sysroot
    /// path that is not the same string on every host.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
