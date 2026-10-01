//! What this provider promises about repeating itself, stated as a declaration and
//! checked against what it actually does.
//!
//! [`crate::Declared_Guarantee`] says how good this provider's answer is. This says
//! whether asking twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism/`, the same seam `crate::limits::
//! LimitsPolicyFactProduction` is discharged through, and for the same reason: a
//! declaration verified only against the real corpus is proven nowhere the gate can see,
//! per `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.naming.policy` fact for a workspace: reading `standards.json`'s
/// own declared naming convention and encoding what it declares.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `crate::limits::LimitsPolicyFactProduction`.
pub struct NamingPolicyFactProduction;

impl Strategy for NamingPolicyFactProduction
{
    /// `State`, not `StateTemporal`. `reading::Canonical_Order` sorts every row by
    /// `(scope, symbol)` before this provider ever encodes them, so two runs that read
    /// `standards.json`'s own object in a different key order — which neither `serde_json`'s
    /// map representation nor this reader's own traversal promises against — reach identical
    /// bytes. Nothing about *when* a row was read is part of what this fact claims; only the
    /// final, sorted set is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// `Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_
    /// Convention` already exercises implicitly and what this declaration commits to
    /// verifying explicitly. `CrossPlatform` would require a golden digest captured on a
    /// second real platform this crate has not been run on — claiming it now would be the
    /// same overclaim `docs/records/OD-ANALYSIS-004` warns against elsewhere.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by `nomos_cap_naming_policy::
    /// Encode_Payload`, and a consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
