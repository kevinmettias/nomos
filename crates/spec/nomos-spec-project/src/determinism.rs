//! What the projection engine promises about repeating itself.
//!
//! The fourth row of the domain table in [`nomos_contracts::Strategy`]'s module —
//! "projection engine output" — which had no implementation anywhere in the workspace
//! until this file. `docs/records/OD-DETERMINISM-001` named it as one of two rows
//! `P9-DETERMINISM` could not reach, and `docs/records/OD-DETERMINISM-002` is the record
//! for reaching it.
//!
//! Verified in `tests/integration/tests/determinism.rs`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Rendering a [`crate::Projection`] into the bytes of a generated document.
///
/// The domain is [`crate::Select`] followed by [`crate::Render`], taken together, which is
/// what [`crate::Build`] is. Selection decides which items a profile reaches and in what
/// order; rendering decides how each is spelled. The promise that matters spans both,
/// because a generated document is checked into a tree and compared against a rebuild —
/// [`crate::Check`] is that comparison — and a difference in either half reads to a
/// reviewer as an edit somebody made.
pub struct ProjectionOutput;

impl Strategy for ProjectionOutput
{
    /// `State`. A selection is ordered by identity rather than by arrival, so the items in
    /// a section are a function of the store's content and not of the order it was written
    /// in — `Test_A_Selection_Should_Order_By_Identity_Rather_Than_By_Arrival` and
    /// `Test_Insertion_Order_Should_Not_Reach_The_Output` are the two halves of that.
    ///
    /// Not `StateTemporal`, for the reason the fact cache is not: the sequence of a
    /// projection's lines is decided by the profile, which is an input, and a promise about
    /// sequence would be a promise about every profile anybody ever writes rather than
    /// about this engine.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossPlatform`. A projection is generated on one machine and reviewed on others.
    ///
    /// The output is committed to a tree and its freshness is decided by comparing a
    /// content digest in a sidecar against the file — so a platform that rendered the same
    /// store differently would report every generated document as hand-edited, on a
    /// machine where nobody had edited anything. `Test_No_Body_Should_Carry_This_Machine`
    /// is the assertion that keeps the environment out of the bytes, and it is written
    /// against the environment this build actually runs in rather than against a pattern
    /// guessing what one looks like.
    ///
    /// Not `CrossBinary`, which is one step further and is not claimed. A projection is
    /// regenerated from the store by the build that reads it, so a recompile that changed
    /// the spelling is a stale-output diff somebody regenerates — recoverable, unlike a
    /// bundle, which is the authority itself and has no store behind it to regenerate from.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossPlatform;

    /// `BitIdentical`. Freshness is decided by a digest of the body, so there is no
    /// tolerance available to define: two bodies that differ by a byte are two different
    /// documents as far as [`crate::Check`] is concerned, and pretending otherwise would
    /// mean a projection could be stale and fresh at once.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
