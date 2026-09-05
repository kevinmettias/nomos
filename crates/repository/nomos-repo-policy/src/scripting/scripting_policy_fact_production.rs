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

/// Producing the `nomos.cap.scripting.policy` fact for a workspace: reading
/// `standards.json`'s `scripting` block and encoding what it declares.
///
/// The analysis-kernel row of the domain table this crate occupies, alongside
/// `crate::limits::LimitsPolicyFactProduction`.
pub struct ScriptingPolicyFactProduction;

impl Strategy for ScriptingPolicyFactProduction
{
    /// `State`, not `StateTemporal`. `tooling_language` is a single value, not a
    /// collection an iteration order could disturb, and `Forbidden_Extensions_In` sorts
    /// `forbidden_extensions` before returning it — so two runs that read `standards.json`'s
    /// array in a different order reach identical bytes regardless. Nothing about *when* the
    /// file was read is part of what this fact claims; only the two fields it declares are.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what this crate's own
    /// `Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Declared_Policy`
    /// already exercises implicitly and what this declaration commits to verifying
    /// explicitly. `CrossPlatform` would require a golden digest captured on a second real
    /// platform this crate has not been run on — claiming it now would be the same overclaim
    /// `docs/records/OD-ANALYSIS-004` warns against elsewhere.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by `nomos_cap_scripting_policy::
    /// Encode_Payload`, and a consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
