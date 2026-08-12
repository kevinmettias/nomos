//! What an edit did.

use nomos_analysis::InvalidationReport;
use nomos_workspace::Applied;

/// What an edit did.
///
/// Two arms rather than a report and a flag, mirroring [`Applied`] for the same reason: a
/// caller has to be told which world it is in, and an empty [`InvalidationReport`] would
/// make "nothing was invalidated because nothing changed" indistinguishable from
/// "invalidation ran over a changed workspace and reached nothing", which is a bug.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edited
{
    /// The workspace is now something else, and the store was told.
    Advanced
    {
        applied: Applied,
        invalidated: InvalidationReport,
    },
    /// The edit said what the workspace already said, so nothing was invalidated.
    ///
    /// An editor saving an unmodified file arrives here. Invalidating anyway would discard
    /// every fact reachable from that file in order to recompute the answers the store
    /// already held.
    Unchanged
    {
        applied: Applied,
    },
}

impl Edited
{
    #[must_use]
    pub const fn Outcome(&self) -> &Applied
    {
        return match self
        {
            Self::Advanced { applied, .. } | Self::Unchanged { applied } => applied,
        };
    }
}
