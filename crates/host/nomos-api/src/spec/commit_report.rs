//! [`CommitReportResponse`], carried only by [`super::commit::SpecCommitResponse`].

use nomos_spec_store::CommitReport;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::CommitReport`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct CommitReportResponse
{
    pub node_id: String,
    pub path: String,
    pub blocks: usize,
    pub blocks_removed: usize,
    pub relations_added: usize,
    pub relations_removed: usize,
    pub renamed: bool,
}

impl CommitReportResponse
{
    pub(crate) fn From(report: CommitReport) -> Self
    {
        return Self {
            node_id: report.node_id,
            path: report.path,
            blocks: report.blocks,
            blocks_removed: report.blocks_removed,
            relations_added: report.relations_added,
            relations_removed: report.relations_removed,
            renamed: report.renamed,
        };
    }
}
