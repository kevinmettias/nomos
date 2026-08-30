//! [`CommitReportResponse`], carried only by [`super::commit_response::CommitResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Commit_Report()
    {
        let report = CommitReport {
            node_id: "D-132".to_owned(),
            path: "docs/records/d-132.md".to_owned(),
            blocks: 4,
            blocks_removed: 1,
            relations_added: 2,
            relations_removed: 0,
            renamed: true,
        };

        let response = CommitReportResponse::From(report.clone());

        assert_eq!(response.node_id, report.node_id);
        assert_eq!(response.path, report.path);
        assert_eq!(response.blocks, report.blocks);
        assert_eq!(response.blocks_removed, report.blocks_removed);
        assert_eq!(response.relations_added, report.relations_added);
        assert_eq!(response.relations_removed, report.relations_removed);
        assert_eq!(response.renamed, report.renamed);
    }
}
