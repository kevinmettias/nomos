//! One attempt at one step's dispatch.

use crate::{DispatchError, StepTiming};

/// One attempt at one step's dispatch: which step it was, which attempt, whether the
/// dispatch failed, and what the step's declared `Timeout` measured over it.
///
/// A run appends one of these per dispatch, in the order the dispatches happened, so a
/// step whose `nomos_contracts::RetryPolicy::Retry` re-dispatched it twice leaves two
/// entries carrying the same [`Self::index`] and successive [`Self::attempt`] numbers.
/// That ordered list is the whole of how a caller reads what retrying did.
#[derive(Clone, Debug, PartialEq)]
pub struct StepAttempt
{
    /// The position in the plan of the step this attempt dispatched.
    pub index: usize,
    /// Which attempt this was, counting from one.
    ///
    /// `RetryPolicy::Retry::max_attempts` counts the first attempt, so attempt one is
    /// never itself a retry and a step declaring `RetryPolicy::NoRetry` never reports an
    /// attempt past it.
    pub attempt: u32,
    /// Why this attempt's dispatch failed, or `None` when it succeeded.
    ///
    /// The successful attempt's own outcome is not copied here: it is the one
    /// [`crate::WorkflowOutcome`]'s own `completed` already carries at this step's
    /// position, and carrying it twice would make two answers to one question. What this
    /// field adds is the failures of the attempts that did not survive, which nothing
    /// else keeps.
    pub failure: Option<DispatchError>,
    /// What this step's declared `nomos_contracts::Timeout` measured over this attempt.
    pub timing: StepTiming,
}
