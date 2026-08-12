//! One row, carrying its own table name and only natural keys.

use serde::{Deserialize, Serialize};

use crate::blob::Blob;
use crate::lineage::Lineage;
use crate::node::Node;
use crate::node_alias::NodeAlias;
use crate::node_history::NodeHistory;
use crate::normative_statement::NormativeStatement;
use crate::omission::Omission;
use crate::record_front_matter::RecordFrontMatter;
use crate::record_relation::RecordRelation;
use crate::relation::Relation;
use crate::relation_type::RelationType;
use crate::source_block::SourceBlock;
use crate::source_document::SourceDocument;
use crate::source_heading::SourceHeading;
use crate::source_table_row::SourceTableRow;
use crate::submission::Submission;
use crate::submission_gap::SubmissionGap;
use crate::submission_value::SubmissionValue;
use crate::suite::Suite;

/// One row, carrying its own table name and only natural keys.
///
/// `uid` never appears. Two databases built from the same bundle assign different
/// surrogates and must still be the same corpus.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "table", content = "record")]
pub enum Record
{
    #[serde(rename = "submissions")]
    Submission(Submission),
    #[serde(rename = "submission_values")]
    SubmissionValue(SubmissionValue),
    #[serde(rename = "submission_gaps")]
    SubmissionGap(SubmissionGap),
    #[serde(rename = "blobs")]
    Blob(Blob),
    #[serde(rename = "source_documents")]
    SourceDocument(SourceDocument),
    #[serde(rename = "source_headings")]
    SourceHeading(SourceHeading),
    #[serde(rename = "source_blocks")]
    SourceBlock(SourceBlock),
    #[serde(rename = "source_table_rows")]
    SourceTableRow(SourceTableRow),
    #[serde(rename = "suites")]
    Suite(Suite),
    #[serde(rename = "nodes")]
    Node(Node),
    #[serde(rename = "node_aliases")]
    NodeAlias(NodeAlias),
    #[serde(rename = "node_history")]
    NodeHistory(NodeHistory),
    #[serde(rename = "relation_types")]
    RelationType(RelationType),
    #[serde(rename = "relations")]
    Relation(Relation),
    #[serde(rename = "normative_statements")]
    NormativeStatement(NormativeStatement),
    #[serde(rename = "lineage")]
    Lineage(Lineage),
    #[serde(rename = "omissions")]
    Omission(Omission),
    #[serde(rename = "record_front_matter")]
    RecordFrontMatter(RecordFrontMatter),
    #[serde(rename = "record_relations")]
    RecordRelation(RecordRelation),
}

impl Record
{
    #[must_use]
    pub const fn Table(&self) -> &'static str
    {
        return match self
        {
            Self::Submission(_) => "submissions",
            Self::SubmissionValue(_) => "submission_values",
            Self::SubmissionGap(_) => "submission_gaps",
            Self::Blob(_) => "blobs",
            Self::SourceDocument(_) => "source_documents",
            Self::SourceHeading(_) => "source_headings",
            Self::SourceBlock(_) => "source_blocks",
            Self::SourceTableRow(_) => "source_table_rows",
            Self::Suite(_) => "suites",
            Self::Node(_) => "nodes",
            Self::NodeAlias(_) => "node_aliases",
            Self::NodeHistory(_) => "node_history",
            Self::RelationType(_) => "relation_types",
            Self::Relation(_) => "relations",
            Self::NormativeStatement(_) => "normative_statements",
            Self::Lineage(_) => "lineage",
            Self::Omission(_) => "omissions",
            Self::RecordFrontMatter(_) => "record_front_matter",
            Self::RecordRelation(_) => "record_relations",
        };
    }
}
