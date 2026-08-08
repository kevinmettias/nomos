use serde::{Deserialize, Serialize};

/// A source document, named the way a person names one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRef
{
    pub path: String,
    pub revision: String,
}

/// A block or a heading, addressed by its position inside a document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinalRef
{
    pub document: DocumentRef,
    pub ordinal: i64,
}

/// How a blob's bytes are spelled in the bundle.
///
/// Chosen from the bytes alone, so a re-export picks the same arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobEncoding
{
    Utf8,
    Base64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob
{
    pub sha256: String,
    pub byte_length: i64,
    pub encoding: BlobEncoding,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDocument
{
    pub path: String,
    pub revision: String,
    pub blob_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHeading
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub depth: i64,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBlock
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub kind: String,
    pub heading_path: String,
    pub text: String,
    pub content_hash: String,
    pub normalized_hash: String,
}

/// A table row, addressed by the block that carries it and its position within.
///
/// Not an [`OrdinalRef`] with a different meaning: an `OrdinalRef` is a position inside a
/// document, and a row's position is inside a block. Reusing the type would make the two
/// interchangeable at the call site, and a lineage row pointing at block 7 when it meant
/// row 7 resolves to something rather than failing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRowRef
{
    pub block: OrdinalRef,
    pub ordinal: i64,
}

/// One pipe line of a table, addressed by the block that carries it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTableRow
{
    pub block: OrdinalRef,
    pub ordinal: i64,
    pub table_ordinal: i64,
    pub kind: String,
    pub cells: Vec<String>,
    pub text: String,
    pub content_hash: String,
    pub normalized_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
    pub deleted_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAlias
{
    pub alias: String,
    pub node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeHistory
{
    pub node_id: String,
    pub ordinal: i64,
    pub event: String,
    pub reason: String,
    pub previous_event_hash: Option<String>,
    pub event_hash: String,
    pub recorded_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationType
{
    pub name: String,
    pub tier: String,
    pub inverse_of: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation
{
    pub from_node_id: String,
    pub relation_type: String,
    pub to_node_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormativeStatement
{
    pub statement_id: String,
    pub node_id: String,
    pub kind: String,
    pub canonical_text: String,
    pub canonical_hash: String,
    pub supersedes_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage
{
    pub source_block: Option<OrdinalRef>,
    pub source_heading: Option<OrdinalRef>,
    pub source_table_row: Option<TableRowRef>,
    pub disposition: String,
    pub target_node_id: Option<String>,
    pub target_statement_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Omission
{
    pub source_block: Option<OrdinalRef>,
    pub source_heading: Option<OrdinalRef>,
    pub reason: String,
    pub justification: String,
    pub decision_record: String,
}

/// One row, carrying its own table name and only natural keys.
///
/// `uid` never appears. Two databases built from the same bundle assign different
/// surrogates and must still be the same corpus.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "table", content = "record")]
pub enum Record
{
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
}

impl Record
{
    #[must_use]
    pub const fn Table(&self) -> &'static str
    {
        return match self
        {
            Self::Blob(_) => "blobs",
            Self::SourceDocument(_) => "source_documents",
            Self::SourceHeading(_) => "source_headings",
            Self::SourceBlock(_) => "source_blocks",
            Self::SourceTableRow(_) => "source_table_rows",
            Self::Node(_) => "nodes",
            Self::NodeAlias(_) => "node_aliases",
            Self::NodeHistory(_) => "node_history",
            Self::RelationType(_) => "relation_types",
            Self::Relation(_) => "relations",
            Self::NormativeStatement(_) => "normative_statements",
            Self::Lineage(_) => "lineage",
            Self::Omission(_) => "omissions",
        };
    }
}
