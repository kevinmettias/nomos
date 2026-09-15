//! What changed between two runs' own findings, and what may be read into it.

use crate::{Comparability, DispositionChange};
use nomos_contracts::{Finding, RunId};

/// What changed between `baseline` and `candidate`'s own [`crate::GateFindings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateCompareResult
{
    /// The run compared against.
    pub baseline: RunId,
    /// The run being compared.
    pub candidate: RunId,
    /// Present in `candidate`, absent from `baseline` -- by occurrence identity.
    pub added: Vec<Finding>,
    /// Present in `baseline`, absent from `candidate`.
    pub removed: Vec<Finding>,
    /// Present in both, but which bucket it fell into changed.
    pub changed: Vec<DispositionChange>,
    /// What these two runs' own provenance says about attributing the difference above.
    ///
    /// Read this before the three lists. `OD-GATE-031`: a difference may be attributed to
    /// repository state only when the rest of the judgment was compatible, or its differences
    /// are represented — and they are represented here.
    pub comparability: Comparability,
}
