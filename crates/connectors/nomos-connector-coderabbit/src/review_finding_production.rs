//! What this provider promises about repeating itself, stated as a declaration.
//!
//! [`crate::guarantee::Declared_Guarantee`] says how good this provider's answer is. This
//! says whether asking twice yields the same answer.
//!
//! # Why this declares over the translation, not over a live `gh` call
//!
//! `OD-CONNECTOR-002`'s own evidence rule is why: a live call is `Observed` evidence about
//! GitHub at the moment it ran, never a repeatable claim about bytes -- the comment's own
//! record can change or disappear between two calls, and a `Strategy` declaration this
//! crate cannot honestly meet is a declaration it must not make.
//! [`crate::translation::Translate_Review_Comment`] is the part that genuinely repeats: the
//! same recorded vendor bytes always translate to the same canonical bytes, which is
//! exactly what [`crate::provider::Fact_Of`] computes from them with no process run.
//!
//! The verification this declaration owes is discharged in
//! `tests/integration/tests/determinism/`, the same seam every other `Strategy` in this
//! workspace is discharged through, and for the same reason: a declaration verified only
//! against the real corpus is proven nowhere the gate can see, per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.review.finding` fact for one already-fetched review comment:
/// translating `CodeRabbit`'s own recorded comment and encoding what it reports.
pub struct ReviewFindingProduction;

impl Strategy for ReviewFindingProduction
{
    /// `State`, not `StateTemporal`. [`crate::translation::Translate_Review_Comment`] reads
    /// a fixed JSON object with no iteration order to depend on, and
    /// [`crate::payload::Encode_Payload`] writes its eight fields in one fixed order every
    /// time. Nothing about *when* this ran is part of what the fact claims; only the
    /// translated result is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// `Test_Fact_Of_Should_Materialize_The_Recorded_Fixture` already exercises implicitly
    /// and what this declaration commits to verifying explicitly. `CrossPlatform` would
    /// require a golden digest captured on a second real platform this crate has not been
    /// run on -- claiming it now would be an overclaim this workspace's own discipline
    /// elsewhere warns against.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by this crate's own writer, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
