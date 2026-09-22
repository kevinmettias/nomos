//! Why a replay was refused rather than run.

use crate::WorkflowDefinitionId;

/// Why a replay was refused rather than run.
///
/// Three refusals rather than one, because they are three different mistakes and the
/// remedy for each is different. Offering the wrong definition entirely is a caller error.
/// Offering a later version is a question about which version was meant. Offering the same
/// identity at the same version with different content is neither -- it is a definition
/// that was edited in place without being republished, and it is the one this whole
/// comparison exists to catch: it is the only one of the three that would otherwise have
/// run silently and produced a result attributed to a definition that no longer says what
/// it said.
///
/// A refusal is never a fallback to the recorded definition. `OD-ROADMAP-006` decision 2's
/// `WF-011` clause asks for pinned historical replay, and a replay that quietly ran the
/// recorded definition after being handed a different one would be pinned in the worst
/// way: correct, and silent about the fact that the caller and the record disagreed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayRefusal
{
    /// The definition offered is not the one the run recorded.
    Identity
    {
        /// The identity the run recorded.
        recorded: WorkflowDefinitionId,
        /// The identity offered now.
        offered: WorkflowDefinitionId,
    },
    /// The definition offered carries the recorded identity at a different version.
    Version
    {
        /// The version the run recorded.
        recorded: u32,
        /// The version offered now.
        offered: u32,
    },
    /// The definition offered carries the recorded identity and version, and different
    /// content.
    Content
    {
        /// The identity both the record and the offer claim.
        id: WorkflowDefinitionId,
        /// The version both the record and the offer claim.
        version: u32,
    },
}
