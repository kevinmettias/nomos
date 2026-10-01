//! What this provider promises about repeating itself, stated as a declaration and checked
//! against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says whether
//! asking twice yields the same answer — the identical split `nomos-lang-rust`'s own
//! `determinism.rs` draws, and this crate's version of the same declaration.
//!
//! The name is the one its two siblings already use, deliberately: the analysis-kernel row of
//! the domain table is occupied by "producing the syntax-items fact by parsing", and a third
//! producer of that fact occupies the same row rather than minting a row of its own.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.syntax.items` fact for one C# file, by parsing it.
///
/// The analysis-kernel row of the domain table in [`nomos_contracts::Strategy`]'s module —
/// the same row `nomos-lang-rust`'s and `nomos-lang-go`'s own `SyntaxFactProduction` occupy,
/// for a third producer of the identical capability.
pub struct SyntaxFactProduction;

impl Strategy for SyntaxFactProduction
{
    /// `StateTemporal`, for the identical reason `nomos-lang-rust`'s declaration gives: the
    /// payload encodes each item with its ordinal, so a run that found the same items in a
    /// different order produces different bytes and a different
    /// [`nomos_analysis::FactPayload::Digest`] — a different fact about the same file.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`. Nothing on this path touches a clock, a path separator, an environment
    /// variable or an unordered collection: the walk is `tree-sitter`'s, in source order, and
    /// the digest is blake3.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`. The output is bytes, and a consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
