//! What compensating one completed step of a failed run did, refused, or owes.

/// What compensating one completed step of a failed run did, refused, or owes.
///
/// One variant per answer `nomos_contracts::Compensation` admits once a run has actually
/// failed: a declaration this crate could honor, a declaration it could not, and a
/// declaration that was never this crate's to honor in the first place. A step declaring
/// `Compensation::None` produces no entry at all, because it asked for nothing and
/// reporting "nothing was asked" per step would bury the three answers that matter.
///
/// `Compensation`'s own doc is deliberate that it does not name *which* step provides
/// external compensation -- that is a workflow definition's concern, not a step's. So
/// [`Self::Owed`] is the whole of what this crate can say about
/// `Compensation::ExternallyCompensated`: the compensating run is owed by whatever
/// assembled the plan, and this run states the debt rather than silently skipping it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepCompensation
{
    /// The step declared `Compensation::SelfCompensating` and its body's own compensating
    /// mode ran.
    Compensated
    {
        /// The position in the plan of the step that was compensated.
        index: usize,
        /// Every path put back to the content the step's own body carried before it ran.
        ///
        /// Empty when the body's compensating mode ran and found nothing to put back --
        /// a correction step that staged without committing wrote no file, so undoing it
        /// restores nothing. That is not a refusal: the mode the body supports ran, and
        /// reported that the step had left nothing behind.
        restored: Vec<String>,
    },
    /// The step declared `Compensation::SelfCompensating` and the declaration could not be
    /// honored: its body supports no compensating mode this crate can reach, or the mode
    /// it supports refused.
    Refused
    {
        /// The position in the plan of the step whose declaration was refused.
        index: usize,
        /// Why the declaration could not be honored.
        reason: String,
    },
    /// The step declared `Compensation::ExternallyCompensated`, so its compensating run is
    /// owed by whatever assembled this workflow rather than performed here.
    Owed
    {
        /// The position in the plan of the step whose compensation is owed.
        index: usize,
    },
}
