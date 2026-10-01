//! [`ComplexityFactProduction`], what this provider promises about repeating itself.
//!
//! [`crate::Declared_Guarantee`] says how good the answer is; this says whether asking twice
//! yields the same one. The declaration is discharged in `tests/integration/tests/determinism/`,
//! over fixtures held in this repository.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.metric.complexity` fact for one file by parsing it -- the
/// analysis-kernel row of the domain table in [`nomos_contracts::Strategy`]'s module, the row
/// `nomos_lang_rust::SyntaxFactProduction` holds for the same kind of producer.
///
/// Its own type rather than a reuse of that one: a provider names no sibling in its band, and a
/// promise borrowed from another crate would be kept or broken by code this crate does not hold.
pub struct ComplexityFactProduction;

impl Strategy for ComplexityFactProduction
{
    /// `StateTemporal`: the payload lists functions in the order each one opens, so two runs
    /// that met the same functions in another order would write different bytes -- a different
    /// fact about the same file, which `State` alone could not express.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`: the fact is keyed by a digest of its bytes and a shared cache is wrong
    /// rather than slow if one file digests differently on two machines. Nothing on the path
    /// reads a clock, a path separator, an environment variable or an unordered collection: the
    /// walk is `syn`'s, in source order, the descriptor is constant text, and the digest is
    /// blake3.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`: the output is bytes, compared for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
