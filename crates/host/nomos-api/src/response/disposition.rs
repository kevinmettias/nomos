//! [`Disposition`], the reduced verdict [`super::gate_run_response::GateRunResponse`] carries.

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Map_Every_Gate_Run_Outcome_Variant_To_Its_Own_Disposition()
    {
        assert_eq!(Disposition::From(GateRunOutcome::Passed), Disposition::Passed);
        assert_eq!(Disposition::From(GateRunOutcome::Failed), Disposition::Failed);
        assert_eq!(Disposition::From(GateRunOutcome::Indeterminate), Disposition::Indeterminate);
    }
}
