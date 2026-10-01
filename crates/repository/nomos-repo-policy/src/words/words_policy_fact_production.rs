//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism/`, the same seam its three siblings' own
//! `XPolicyFactProduction` are discharged through, and for the same reason: a declaration
//! verified only against the real corpus is proven nowhere the gate can see, per
//! `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.words.policy` fact for a workspace: reading `standards.json`'s
/// `words.approved_abbreviations` and encoding what it declares.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside its three
/// siblings.
pub struct WordsPolicyFactProduction;

impl Strategy for WordsPolicyFactProduction
{
    /// `State`, not `StateTemporal`. `reading::Discover_Workspace` sorts the additions list
    /// before this provider ever encodes it, so two runs reach identical bytes regardless
    /// of `standards.json`'s own array order. Nothing about *when* a word was read is part
    /// of what this fact claims; only the final, sorted set is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// `Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_
    /// Additions` already exercises implicitly and what this declaration commits to
    /// verifying explicitly. `CrossPlatform` would require a golden digest captured on a
    /// second real platform this crate has not been run on.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by `nomos_cap_words_policy::
    /// Encode_Payload`, and a consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
