//! A JSON-serializable projection of `nomos_gate_orchestration::GateRunResult`.
//!
//! `GateRunResult` and `GateRunOutcome` do not derive `Serialize` -- nothing needed a wire
//! format for either before this crate existed. Adding it to them directly would grow
//! `nomos-gate-orchestration`'s own public surface on behalf of one caller's shape, before a
//! second transport exists to check that shape against -- the same premature-surface caution
//! this crate's own module doc names. This type is that shape, owned here and built by
//! conversion from the borrowed result, so `nomos-gate-orchestration` stays exactly as it
//! was.

use nomos_contracts::{Finding, RunId};
use nomos_gate_orchestration::{GateRunOutcome, GateRunResult};
use serde::Serialize;
use std::path::PathBuf;

/// What a real gate run over `root` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
pub struct GateRunResponse
{
    /// The identity of this execution. `RunId` already derives `Serialize` -- unlike
    /// [`GateRunOutcome`], it needs no local twin.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// The reduced verdict.
    pub disposition: Disposition,
    /// Exactly the findings that failed this build. Empty whenever `disposition` is not
    /// [`Disposition::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// Findings an `AdoptionPolicy` calibration kept from blocking.
    pub calibrated_findings: Vec<Finding>,
    /// Findings a `Suppression` kept from blocking.
    pub suppressed_findings: Vec<Finding>,
    /// Findings a `BaselineDebt` kept from blocking.
    pub baselined_findings: Vec<Finding>,
}

impl GateRunResponse
{
    pub(crate) fn From(result: GateRunResult) -> Self
    {
        return Self {
            run: result.run,
            root: result.root,
            disposition: Disposition::From(result.disposition),
            blocking_findings: result.blocking_findings,
            calibrated_findings: result.calibrated_findings,
            suppressed_findings: result.suppressed_findings,
            baselined_findings: result.baselined_findings,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::GateRunOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason this module's own doc gives; kept to the same three variants, in the same
/// order, so a mismatch between the two is a compile error in [`Disposition::From`] rather
/// than a silent divergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition
{
    /// No finding this run saw can fail a build.
    Passed,
    /// At least one finding can fail a build.
    Failed,
    /// The check behind this run could not produce an authoritative judgment.
    Indeterminate,
}

impl Disposition
{
    fn From(outcome: GateRunOutcome) -> Self
    {
        return match outcome
        {
            GateRunOutcome::Passed => Disposition::Passed,
            GateRunOutcome::Failed => Disposition::Failed,
            GateRunOutcome::Indeterminate => Disposition::Indeterminate,
        };
    }
}
