//! What this provider promises about repeating itself, stated as a declaration and checked
//! against what it actually does.
//!
//! [`crate::Ceiling`] says how good this provider's answer is. This says whether asking
//! twice yields the same answer. The verification owed is discharged in
//! `tests/integration/tests/determinism/`, the same seam every `crates/repository/`-shaped
//! provider's own `Strategy` is discharged through, and for the same reason: a declaration
//! verified only against the real corpus is proven nowhere the gate can see, per
//! `docs/records/OD-GATE-001`.

use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// Producing the `nomos.cap.requirement.trace` fact for a workspace: reading every committed
/// `tests/contract/requirements/*.assessment` entry, comparing it against the real tree, and
/// encoding what was found.
///
/// The analysis-kernel row of the domain table, the same triple every `crates/repository/`
/// policy provider's own `*FactProduction` occupies.
pub struct RequirementTraceFactProduction;

impl Strategy for RequirementTraceFactProduction
{
    /// `State`, not `StateTemporal`. `crate::registry::Entries` sorts every assessment it
    /// reads by requirement before any predicate runs, and each predicate then walks a
    /// sorted assessment's own declared sites, gaps and record in the order they are
    /// written — nothing about *when* this provider runs is part of what the fact claims,
    /// only the committed text is.
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;

    /// `CrossRun`, not `CrossPlatform`. Two runs on this machine are what
    /// `crate::provider`'s own `Test_Materialize_Workspace_Should_Materialize_This_
    /// Repositorys_Own_Committed_Corpus` already exercises implicitly and what this
    /// declaration commits to verifying explicitly. `CrossPlatform` would require a golden
    /// digest captured on a second real platform this crate has not been run on.
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossRun;

    /// `BitIdentical`. The output is bytes, encoded by [`crate::Encode_Payload`], and a
    /// consumer compares them for equality.
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}
