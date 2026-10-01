//! What snapshot serialization promises about repeating itself.
//!
//! The serialization row of the domain table in [`nomos_contracts::Strategy`]'s module.
//! That row reads "snapshot and spec-bundle serialization" and this declaration covers
//! only the first half — the spec bundle is a different crate in a band this one does not
//! reach, and it is named here so the half that is undeclared is visible rather than
//! implied.
//!
//! Verified in `tests/integration/tests/determinism.rs`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Encoding a [`crate::WorkspaceSnapshot`] to bytes.
///
/// This is the domain [`crate::WorkspaceSnapshot::Id`] rests on. A snapshot identity is a
/// digest of the encoded bytes, so every fact keyed against a workspace state inherits
/// whatever this domain promises — which is why the scope here is the strongest one in
/// the table rather than the one that would be cheapest to hold.
pub struct SnapshotSerialization;

impl Strategy for SnapshotSerialization
{
    /// `State`. A snapshot is a set of members held in a `BTreeMap`, so the encoding's
    /// order is a function of the paths rather than of the order they arrived in, and
    /// there is no sequence left to promise that the keys do not already determine.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossBinary`, the most expensive claim in the table, and the one a stored
    /// baseline needs.
    ///
    /// A snapshot's bytes are an identity that outlives the process that produced it and
    /// is compared against identities produced by other builds of this analyzer. If a
    /// recompile changed them, every fact in every store would re-address at once and the
    /// caches would all miss without anything being wrong — a failure that looks like
    /// slowness and is actually a broken identity.
    ///
    /// What makes it holdable is that the encoding names nothing about the machine or the
    /// build: paths are relative, content is a digest, and there is no timestamp, no
    /// pointer, no `HashMap` iteration and no `#[derive(Hash)]` in the path.
    /// `tests/portable.rs` asserts the machine half of that over a real tree.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossBinary;

    /// `BitIdentical`. The bytes are hashed to form an identity, so any tolerance at all
    /// would be a tolerance on identity.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
