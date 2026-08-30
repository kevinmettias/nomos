//! The tables a caller may count, and the statement each one carries.

// Counting a table's rows, each number its own measurement.
pub(crate) mod rows;

pub(crate) mod row_census;
pub(crate) mod row_scope;

pub(crate) mod suite_authority;
pub(crate) mod line;

/// The counting statement each table carries.
///
/// One constant per table rather than eighteen match arms each holding their own string:
/// the arms then say only which statement belongs to which variant, and the statements read
/// as the table of literals they are. Written out rather than interpolated from
/// [`Table::Name`] for the reason [`Table::Tally_Sql`] gives.
const TALLY_BLOBS: &str = "SELECT 'blobs' AS which, count(*) AS tally FROM blobs";
const TALLY_SOURCE_DOCUMENTS: &str =
    "SELECT 'source_documents' AS which, count(*) AS tally FROM source_documents";
const TALLY_SOURCE_HEADINGS: &str =
    "SELECT 'source_headings' AS which, count(*) AS tally FROM source_headings";
const TALLY_SOURCE_BLOCKS: &str =
    "SELECT 'source_blocks' AS which, count(*) AS tally FROM source_blocks";
const TALLY_SOURCE_TABLE_ROWS: &str =
    "SELECT 'source_table_rows' AS which, count(*) AS tally FROM source_table_rows";
const TALLY_SUITES: &str = "SELECT 'suites' AS which, count(*) AS tally FROM suites";
const TALLY_NODES: &str = "SELECT 'nodes' AS which, count(*) AS tally FROM nodes";
const TALLY_NODE_ALIASES: &str =
    "SELECT 'node_aliases' AS which, count(*) AS tally FROM node_aliases";
const TALLY_NODE_HISTORY: &str =
    "SELECT 'node_history' AS which, count(*) AS tally FROM node_history";
const TALLY_RELATIONS: &str = "SELECT 'relations' AS which, count(*) AS tally FROM relations";
const TALLY_RELATION_TYPES: &str =
    "SELECT 'relation_types' AS which, count(*) AS tally FROM relation_types";
const TALLY_NORMATIVE_STATEMENTS: &str =
    "SELECT 'normative_statements' AS which, count(*) AS tally FROM normative_statements";
const TALLY_LINEAGE: &str = "SELECT 'lineage' AS which, count(*) AS tally FROM lineage";
const TALLY_OMISSIONS: &str = "SELECT 'omissions' AS which, count(*) AS tally FROM omissions";
const TALLY_RECORD_FRONT_MATTER: &str =
    "SELECT 'record_front_matter' AS which, count(*) AS tally FROM record_front_matter";
const TALLY_RECORD_RELATIONS: &str =
    "SELECT 'record_relations' AS which, count(*) AS tally FROM record_relations";
const TALLY_SUBMISSIONS: &str = "SELECT 'submissions' AS which, count(*) AS tally FROM submissions";
const TALLY_SUBMISSION_VALUES: &str =
    "SELECT 'submission_values' AS which, count(*) AS tally FROM submission_values";
const TALLY_SUBMISSION_GAPS: &str =
    "SELECT 'submission_gaps' AS which, count(*) AS tally FROM submission_gaps";

/// The tables a caller may count.
///
/// An enum rather than a string, so `Count` cannot become a hole through which
/// arbitrary SQL reaches the database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table
{
    Blobs,
    SourceDocuments,
    SourceHeadings,
    SourceBlocks,
    SourceTableRows,
    Suites,
    Nodes,
    NodeAliases,
    NodeHistory,
    Relations,
    RelationTypes,
    NormativeStatements,
    Lineage,
    Omissions,
    RecordFrontMatter,
    RecordRelations,
    Submissions,
    SubmissionValues,
    SubmissionGaps,
}

impl Table
{
    #[must_use]
    pub const fn Name(self) -> &'static str
    {
        return match self
        {
            Self::Blobs => "blobs",
            Self::SourceDocuments => "source_documents",
            Self::SourceHeadings => "source_headings",
            Self::SourceBlocks => "source_blocks",
            Self::SourceTableRows => "source_table_rows",
            Self::Suites => "suites",
            Self::Nodes => "nodes",
            Self::NodeAliases => "node_aliases",
            Self::NodeHistory => "node_history",
            Self::Relations => "relations",
            Self::RelationTypes => "relation_types",
            Self::NormativeStatements => "normative_statements",
            Self::Lineage => "lineage",
            Self::Omissions => "omissions",
            Self::RecordFrontMatter => "record_front_matter",
            Self::RecordRelations => "record_relations",
            Self::Submissions => "submissions",
            Self::SubmissionValues => "submission_values",
            Self::SubmissionGaps => "submission_gaps",
        };
    }

    /// The statement counting this table's rows, labelled with the table's own name.
    ///
    /// Written out per variant rather than interpolated from [`Table::Name`] at the call
    /// site. A table identifier occupies no value position, so no driver can bind one and
    /// there is no parameterized form to prefer; the only way to keep the statement out of
    /// runtime string building is for each variant to carry its own. The label is selected
    /// rather than assumed from row order, because a compound `SELECT` without `ORDER BY`
    /// is not promised to come back in the order its arms were written.
    #[must_use]
    pub const fn Tally_Sql(self) -> &'static str
    {
        return match self
        {
            Self::Blobs => TALLY_BLOBS,
            Self::SourceDocuments => TALLY_SOURCE_DOCUMENTS,
            Self::SourceHeadings => TALLY_SOURCE_HEADINGS,
            Self::SourceBlocks => TALLY_SOURCE_BLOCKS,
            Self::SourceTableRows => TALLY_SOURCE_TABLE_ROWS,
            Self::Suites => TALLY_SUITES,
            Self::Nodes => TALLY_NODES,
            Self::NodeAliases => TALLY_NODE_ALIASES,
            Self::NodeHistory => TALLY_NODE_HISTORY,
            Self::Relations => TALLY_RELATIONS,
            Self::RelationTypes => TALLY_RELATION_TYPES,
            Self::NormativeStatements => TALLY_NORMATIVE_STATEMENTS,
            Self::Lineage => TALLY_LINEAGE,
            Self::Omissions => TALLY_OMISSIONS,
            Self::RecordFrontMatter => TALLY_RECORD_FRONT_MATTER,
            Self::RecordRelations => TALLY_RECORD_RELATIONS,
            Self::Submissions => TALLY_SUBMISSIONS,
            Self::SubmissionValues => TALLY_SUBMISSION_VALUES,
            Self::SubmissionGaps => TALLY_SUBMISSION_GAPS,
        };
    }

    /// Every table in the schema — a list kept beside the enum, which the compiler does
    /// not check against the schema it claims to enumerate.
    ///
    /// Mirrored by `Test_Every_Table_In_The_Schema_Should_Be_Declared`.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Blobs,
            Self::SourceDocuments,
            Self::SourceHeadings,
            Self::SourceBlocks,
            Self::SourceTableRows,
            Self::Suites,
            Self::Nodes,
            Self::NodeAliases,
            Self::NodeHistory,
            Self::Relations,
            Self::RelationTypes,
            Self::NormativeStatements,
            Self::Lineage,
            Self::Omissions,
            Self::RecordFrontMatter,
            Self::RecordRelations,
            Self::Submissions,
            Self::SubmissionValues,
            Self::SubmissionGaps,
        ];
    }
}
