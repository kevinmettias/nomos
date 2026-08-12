//! Everything that stops the specification store answering.

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
}

impl core::fmt::Display for StoreError
{
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
