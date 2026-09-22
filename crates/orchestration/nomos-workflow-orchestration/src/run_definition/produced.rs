//! What a node published, derived from what its own dispatch target reported.

use nomos_agent_orchestration::AgentDispatchOutcome;
use nomos_check_orchestration::CheckOutcome;
use nomos_correction_orchestration::CorrectionOutcome;
use nomos_gate_orchestration::GateRunOutcome;

use crate::{ProducedState, StepOutcome};

/// The state `outcome` publishes for a later branch to read.
///
/// Each arm defers to the vocabulary the answering seam already published rather than
/// inventing a threshold of its own, and each one that cannot find a clean answer in that
/// vocabulary answers [`ProducedState::Flagged`] rather than guessing.
pub(super) fn Produced_State(outcome: &StepOutcome) -> ProducedState
{
    return match outcome
    {
        StepOutcome::Agent(answer) => Agent_State(answer),
        StepOutcome::Check(judged) => Check_State(judged),
        StepOutcome::Correction(corrected) => State_Of(matches!(corrected, CorrectionOutcome::Clean)),
        StepOutcome::Gate(result) => State_Of(result.disposition == GateRunOutcome::Passed),
    };
}

/// [`ProducedState::Clean`] when `clean`, [`ProducedState::Flagged`] otherwise.
pub(super) const fn State_Of(clean: bool) -> ProducedState
{
    if clean
    {
        return ProducedState::Clean;
    }

    return ProducedState::Flagged;
}

/// What an agent node publishes: the executor's own error flag, or a model backend's
/// answer, which carries no such flag and is therefore clean by its own vocabulary.
///
/// The other two variants of `AgentDispatchOutcome` never reach here, because
/// `run::Dispatched_Agent` reports each of them as a [`crate::DispatchError`] and a
/// dispatch error ends the run before any branch downstream could read a state.
fn Agent_State(answer: &AgentDispatchOutcome) -> ProducedState
{
    return match answer
    {
        AgentDispatchOutcome::Executed { execution, .. } => State_Of(!execution.is_error),
        AgentDispatchOutcome::Answered { .. } => ProducedState::Clean,
        AgentDispatchOutcome::Unavailable { .. } | AgentDispatchOutcome::NotSelected(_) => ProducedState::Flagged,
    };
}

/// What a check node publishes: whether the rules that ran found anything.
///
/// Every variant other than `Judged` is [`ProducedState::Flagged`], and deliberately so:
/// an unreadable tree, a self-contradictory registry, no source and no facts are four ways
/// of not having reached a judgment, and a branch reading one of them as "nothing to act
/// on" would be the exact collapse `CheckOutcome` keeps these variants apart to prevent.
fn Check_State(judged: &CheckOutcome) -> ProducedState
{
    let CheckOutcome::Judged { findings, .. } = judged
    else
    {
        return ProducedState::Flagged;
    };

    return State_Of(findings.is_empty());
}
