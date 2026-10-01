//! [`LintFactProduction`], what this provider promises about repeating itself.
//!
//! [`crate::Declared_Guarantee`] says how good the answer is; this says whether asking twice yields
//! the same one. The declaration is discharged in `tests/integration/tests/determinism/`, over a
//! scripted `go vet` answer, so the claim is about this provider's reading and filing and not about
//! the toolchain behind it.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.lint.diagnostics` fact for one Go module from what `go vet -json`
/// printed for it.
pub struct LintFactProduction;

impl Strategy for LintFactProduction
{
    /// `State`: a module's diagnostics are sorted and deduplicated before they are encoded, so a
    /// `go vet` that reported the same diagnostics in another order -- packages vet concurrently,
    /// and a JSON object promises no key order -- files the same bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossPlatform`: given the same answer, nothing on the path reads a clock or an unordered
    /// collection, every position is made relative to the root with forward slashes before it is
    /// kept, and the digest is blake3. The harness's golden holds it to that across platforms.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`: the output is bytes, compared for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
