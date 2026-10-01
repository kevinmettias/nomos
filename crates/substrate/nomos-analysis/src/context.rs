//! The build a read is filed under.

use nomos_contracts::GenerationId;
use nomos_contracts::ConfigurationId;
use nomos_contracts::BuildVariantId;
use nomos_contracts::SnapshotId;
/// The situation an analysis is being performed in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context
{
    /// The workspace state being analyzed.
    ///
    /// Not a key component — [`Reader::Key_For`] does not read it, and
    /// `docs/records/OD-ANALYSIS-001` says why. It is what a caller stamps onto
    /// [`crate::MaterializedFact::snapshot`] when it writes a fact: the tree the
    /// measurement was taken from, recorded beside the fact rather than folded into what
    /// the fact is.
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}
