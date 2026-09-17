//! What carrying an agent's own plan through the correction lifecycle produced.

use nomos_corrections::{CommittedPlan, Preview};

/// What driving a [`nomos_agent_contracts::WorkResult`]'s plan through the correction
/// lifecycle produced.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatedCorrectionOutcome
{
    /// `result.plan` was `None` -- a judgment-only response, `OD-CONTRACTS-003`'s case.
    /// Not a refusal: nothing was proposed, so nothing was refused.
    NoPlan,
    /// The plan staged, validated and committed against `workspace`, carrying the preview
    /// rendered before either check ran.
    Committed
    {
        preview: Preview,
        committed: CommittedPlan,
    },
}
