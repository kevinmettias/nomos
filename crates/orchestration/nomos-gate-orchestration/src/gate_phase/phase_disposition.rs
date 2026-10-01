//! One phase's own verdict, apart from the run's overall disposition.

/// One phase's own verdict, apart from the run's overall [`crate::GateRunOutcome`] -- a third
/// state, [`Self::Skipped`], that `GateRunOutcome` deliberately does not carry, because a run
/// either judged something or could not judge anything at all, while a phase can additionally
/// simply never have been reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseDisposition
{
    /// This phase's blocking findings did not exceed its threshold, or did and a
    /// [`crate::PhaseApproval`] covered them.
    Passed,
    /// This phase's blocking findings exceeded its threshold and nothing approved them.
    Failed,
    /// An earlier phase already failed unapproved, so this phase was never judged.
    Skipped,
}
