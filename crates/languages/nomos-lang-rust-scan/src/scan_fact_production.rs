//! What the scanner promises about repeating itself.
//!
//! The same triple as `nomos-lang-rust`, and that is the point worth stating. The two
//! providers differ in how good their answers are — [`crate::Declared_Guarantee`] says
//! `Approximate` where the parser says `Verified` — and they do not differ at all in
//! whether they repeat themselves. Determinism and quality are orthogonal axes, and a
//! weaker answer given identically every time is exactly what makes this provider usable
//! as a fallback rather than merely present.
//!
//! Verified in `tests/integration/tests/determinism.rs`, over in-repository fixtures.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.syntax.items` fact for one file, by reading its lines.
pub struct ScanFactProduction;

impl Strategy for ScanFactProduction
{
    /// `StateTemporal`. The encoding carries ordinals for the same reason the parser's
    /// does, and the scan visits lines in file order, which is the only order it has.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`. A line reader has one platform-shaped hazard the parser does not
    /// — line endings — and it resolves that by splitting on `\n` and trimming, so a file
    /// checked out with CRLF scans to the same items as the same file with LF.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
