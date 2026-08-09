//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer, which is a different question and was
//! until now an unstated one: the reproduction property held, and nothing claimed it.
//!
//! The verification owed follows mechanically from the triple below and is discharged in
//! `tests/integration/tests/determinism.rs`, over fixtures held in this repository —
//! deliberately not over the corpus. `tests/corpus.rs` already asserts this property at
//! far greater scale, and reports `ok` having read nothing on every machine that lacks
//! `NOMOS_RUST_CORPUS`, which is every machine CI runs on. A declaration proven only
//! there is proven nowhere the gate can see. See `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.syntax.items` fact for one file, by parsing it.
///
/// The analysis-kernel row of the domain table in [`nomos_contracts::Strategy`]'s module,
/// and this crate is the first thing in the workspace that occupies it against input it
/// did not write.
pub struct SyntaxFactProduction;

impl Strategy for SyntaxFactProduction
{
    /// `StateTemporal`, and the temporal half is the load-bearing one.
    ///
    /// The payload encodes each item with its ordinal, so two runs that found the same
    /// items in a different order are not merely differently ordered — they produce
    /// different bytes and therefore a different [`nomos_analysis::FactPayload::Digest`],
    /// which is a different fact about the same file. `State` alone would be a claim this
    /// provider's own encoding cannot express.
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;

    /// `CrossPlatform`, because the fact this produces is keyed by a digest of its own
    /// bytes and cached under that key.
    ///
    /// A fact cache shared between a Linux runner and a Windows workstation — which is
    /// what a fact cache is for — is wrong rather than slow if the same file digests
    /// differently on the two. The claim costs nothing to hold here because nothing in
    /// the path touches a clock, a path separator, an environment variable or an
    /// unordered collection: the walk is `syn`'s, in source order, and the digest is
    /// blake3.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`. There is no tolerance to define: the output is bytes, and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
