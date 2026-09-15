//! A run whose findings do not yield one identity each, so it cannot be compared.

use nomos_contracts::RunId;
use nomos_model::OccurrenceCollision;

/// A run whose findings do not yield one identity each, so it cannot be compared.
///
/// # Why this refuses instead of coping
///
/// Because every way of coping is a way of losing a finding quietly. Two findings sharing a
/// [`nomos_model::FindingOccurrenceId`] cannot both be a map's value at that key, and the
/// mechanism that would decide between them -- `BTreeMap::insert` returning the displaced
/// value into a `let _` -- is exactly how 103 of this repository's 253 findings were being
/// discarded before this type existed. Not by a decision anybody made: by an ignored return
/// value.
///
/// A collision means the identity material is no longer sufficient for the findings this
/// workspace now produces, which is a defect in the identity rather than a condition a caller
/// can sensibly handle. `FindingOccurrenceId`'s own documentation states the injectivity claim
/// narrowly and names this as the thing that turns a future model change from something
/// somebody has to remember into something that fails. So the refusal is the feature.
///
/// # Why it names the run and both findings
///
/// Naming the run says which side of the comparison to look at -- a caller holds two, and a
/// collision in one says nothing about the other. Naming both findings is the only report that
/// can be acted on: "some identity occurred twice" leaves a reader to re-derive which two from a
/// population of hundreds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollidingOccurrences
{
    /// The run whose findings collided.
    pub run: RunId,
    /// Every colliding pair, each naming both findings that reached one identity.
    pub collisions: Vec<OccurrenceCollision>,
}
