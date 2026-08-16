//! What correction planning and staging promises about repeating itself.
//!
//! The "Correction planning and staging" row of the domain table in
//! [`nomos_contracts::Strategy`]'s module. Verified in
//! `tests/integration/tests/determinism/domains.rs`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Staging, validating, committing and rolling back a [`crate::CorrectionPlan`].
///
/// The whole lifecycle rather than one step, because a caller's actual obligation is that
/// the sequence agrees with itself end to end — a preview that agreed with a commit that
/// silently diverged would be a promise about the wrong half of the mechanism.
pub struct CorrectionStaging;

impl Strategy for CorrectionStaging
{
    /// `StateTemporal`. A plan's forward and reverse changes are built by iterating its
    /// candidates and their edits in order, so an implementation that reordered them would
    /// still reach the same end state but not necessarily the same sequence of effects —
    /// and a caller diffing what a correction did, not merely what it left behind, is
    /// owed that sequence.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossRun`. A plan is staged, validated, committed and rolled back within one
    /// analysis process against workspace state that process holds; nothing here is
    /// stored as a baseline compared across a recompile, which is what would raise this to
    /// `CrossPlatform` or `CrossBinary`.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. Every candidate's identity and every changeset's bytes are content
    /// digests, so there is no tolerance to define.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
