//! Everything that stops the specification store answering.
//!
//! Named `StoreError` rather than `Error` because this crate publishes nine subsystems'
//! vocabulary at one flat root, and two of those subsystems declare an error -- this one and
//! `edit`'s. The published name carries its subsystem, so the two sit side by side.

#[derive(Debug)]
pub enum StoreError
{
    Sql(String),
    Migration
    {
        from: u32,
        cause: String,
    },
    /// The database was written by a newer build than this one.
    TooNew
    {
        found: u32,
        supported: u32,
    },
    /// A document does not read: an authored record this build embeds that will not
    /// parse, or stored bytes that are not text. Both name the document and say what
    /// went wrong with it, because in either case the caller's next question is which
    /// file.
    Record
    {
        path: String,
        cause: String,
    },
    /// A block carries something shaped like a table that cannot be read as one.
    Table
    {
        document_uid: i64,
        ordinal: u32,
        cause: String,
    },
    /// A relation type was registered with no domain, no range or no cardinality.
    ///
    /// `OD-SPEC-012`: a relation type that admits everything is not a constraint, so
    /// registering one with nothing declared is refused rather than left permissive.
    UnconstrainedRelationType
    {
        name: String,
    },
    /// An edge whose endpoint kind the relation type does not admit at that end.
    RelationEndpoint
    {
        relation_type: String,
        role: &'static str,
        node_id: String,
        kind: String,
        admits: Vec<String>,
    },
    /// An edge that would exceed a relation type's declared cap on how many of it one node
    /// may carry.
    RelationCardinality
    {
        relation_type: String,
        node_id: String,
        max_per_node: u32,
    },
}

impl core::fmt::Display for StoreError
{
    // `fmt` is the fixed method name `std::fmt::Display` requires; it is not a style choice
    // and cannot be spelled out without ceasing to implement the trait.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Sql(cause) => write!(formatter, "store error: {cause}"),
            Self::Migration { from, cause } => {
                write!(formatter, "migration from version {from} failed: {cause}")
            }
            Self::TooNew { found, supported } => write!(
                formatter,
                "the store is at schema version {found}; this build understands {supported}. \
                 Refusing to open it rather than reading tables whose meaning may have changed"
            ),
            Self::Record { path, cause } => write!(formatter, "{path}: {cause}"),
            Self::Table {
                document_uid,
                ordinal,
                cause,
            } => write!(formatter, "document {document_uid} block {ordinal}: {cause}"),
            Self::UnconstrainedRelationType { name } => write!(
                formatter,
                "relation type `{name}` declares no domain, range or cardinality. Every \
                 relation type must say which node kinds it may join at each end and how \
                 many edges of it one node may carry; a type that admits everything is not \
                 a constraint"
            ),
            Self::RelationEndpoint { relation_type, role, node_id, kind, admits } => write!(
                formatter,
                "`{relation_type}` does not admit `{node_id}` as its {role}: {node_id} is a \
                 {kind}, and `{relation_type}` admits {} there",
                admits.join(", ")
            ),
            Self::RelationCardinality { relation_type, node_id, max_per_node } => write!(
                formatter,
                "`{node_id}` already carries {max_per_node} `{relation_type}` edge(s), which \
                 is the most `{relation_type}` declares a node may carry; remove one first or \
                 use a different node"
            ),
        };
    }
}

impl std::error::Error for StoreError
{}

impl From<rusqlite::Error> for StoreError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Sql(error.to_string());
    }
}
