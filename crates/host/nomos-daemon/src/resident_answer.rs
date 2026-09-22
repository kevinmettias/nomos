//! What a resident answers one request with.

use nomos_check_orchestration::CheckOutcome;

use crate::{ResidencyCost, StopReport, TreeReading};

/// What came of one request.
///
/// One variant per [`crate::ResidentRequest`], and the judging one carries the seam's own
/// `CheckOutcome` unchanged rather than a shape of this crate's. That is the point of the
/// whole crate: a client holding this holds what a cold `nomos check` holds, plus what
/// residency is able to say about how it was reached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentAnswer
{
    /// The root, judged through the held workspace, fact store and reassessment cache.
    Judged
    {
        /// `nomos_check_orchestration::Run_Reassessing`'s own outcome, carried out whole.
        outcome: CheckOutcome,
        /// What this request read of the tree before it judged.
        reading: TreeReading,
        /// What answering it cost the store.
        cost: ResidencyCost,
    },
    /// The tree, read and compared against what the resident holds, and judged not at all.
    Observed
    {
        /// What this request read of the tree.
        reading: TreeReading,
    },
    /// Everything held was dropped.
    Forgotten
    {
        /// Facts the store was serving before it was dropped.
        released_facts: usize,
    },
    /// The residency ended.
    Stopped(StopReport),
}
