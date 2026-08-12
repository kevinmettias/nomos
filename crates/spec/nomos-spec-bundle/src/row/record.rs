//! One row, carrying its own table name and only natural keys.

pub(crate) mod front_matter;
pub(crate) mod relation;

use serde::{Deserialize, Serialize};

use crate::row::blob::Blob;
use crate::bundle::lineage::Lineage;
use crate::row::node::Node;
use crate::row::node::alias::NodeAlias;
use crate::row::node::history::NodeHistory;
use crate::row::normative_statement::NormativeStatement;
use crate::bundle::omission::Omission;
use crate::row::record::front_matter::RecordFrontMatter;
use crate::row::record::relation::RecordRelation;
use crate::row::relation::Relation;
use crate::row::relation::kind::RelationType;
use crate::row::source::block::SourceBlock;
use crate::row::source::document::SourceDocument;
use crate::row::source::heading::SourceHeading;
use crate::row::source::table_row::SourceTableRow;
use crate::row::submission::Submission;
use crate::row::submission::gap::SubmissionGap;
use crate::row::submission::value::SubmissionValue;
use crate::row::suite::Suite;

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
