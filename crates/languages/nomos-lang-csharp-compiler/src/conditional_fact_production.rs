//! [`ConditionalFactProduction`], what this provider promises about repeating itself.
//!
//! [`crate::Declared_Guarantee`] says how good the answer is; this says whether asking twice
//! yields the same one. The declaration is discharged in `tests/integration/tests/determinism/`
//! over a fixture and a stated definition set.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.csharp.conditional_compilation` fact for one file from its text and a
/// build's definition set -- the analysis-kernel row of the domain table in
/// [`nomos_contracts::Strategy`]'s module.
///
/// # What this covers, and what it does not
///
/// It covers the fact, given the definition set. Evaluating the set runs the host's .NET SDK,
/// whose answer is the SDK's and can differ between two installed versions; that is why the set
/// itself -- the build and every symbol -- is in the fact's payload, so two hosts that evaluated
/// different sets produce different facts rather than one fact that means two things.
pub struct ConditionalFactProduction;

impl Strategy for ConditionalFactProduction
{
    /// `StateTemporal`: branches are listed in the order their directives appear, so a run that
    /// met them in another order would write different bytes.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`: given the same text and the same set, nothing on the path reads a clock,
    /// a path separator, an environment variable or an unordered collection -- the set is a
    /// `BTreeSet` -- and the digest is blake3.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`: the output is bytes, compared for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
