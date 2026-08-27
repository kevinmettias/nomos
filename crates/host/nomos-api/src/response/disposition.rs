//! [`Disposition`], the reduced verdict [`super::run::GateRunResponse`] carries.

use nomos_gate_orchestration::GateRunOutcome;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::GateRunOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason `crate::response`'s own doc gives; kept to the same three variants, in the
/// same order, so a mismatch between the two is a compile error in [`Disposition::From`]
/// rather than a silent divergence.
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
    pub(crate) fn From(outcome: GateRunOutcome) -> Self
    {
        return match outcome
        {
            GateRunOutcome::Passed => Disposition::Passed,
            GateRunOutcome::Failed => Disposition::Failed,
            GateRunOutcome::Indeterminate => Disposition::Indeterminate,
        };
    }
}
