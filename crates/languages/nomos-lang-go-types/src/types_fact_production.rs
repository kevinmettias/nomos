//! [`TypesFactProduction`], what this provider promises about repeating itself.
//!
//! [`crate::Declared_Guarantee`] says how good the answer is; this says whether asking twice yields
//! the same one. The declaration is discharged in `tests/integration/tests/determinism/`, over a
//! scripted helper answer, so the claim is about this provider's reading and filing and not about
//! the toolchain behind it.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.go.discarded_values` fact for each Go source from what the helper
/// printed for its module.
pub struct TypesFactProduction;

impl Strategy for TypesFactProduction
{
    /// `State`: a file's values are put in position order before they are encoded, so a helper that
    /// printed the same values in another order -- packages come in whatever order `go list` gives
    /// them, and a file's values in the order the syntax tree is walked -- files the same bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossPlatform`: given the same answer, nothing on the path reads a clock or an unordered
    /// collection, every path is made relative to the root with forward slashes before it is kept,
    /// and the digest is blake3. The harness's golden holds it to that across platforms.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`: the output is bytes, compared for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
