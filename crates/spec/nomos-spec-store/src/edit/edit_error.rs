//! Why an authoring step refused.

use nomos_spec_model::RenderError;

use crate::StoreError;

/// Why an authoring step refused.
#[derive(Debug)]
pub enum EditError
{
    /// No node in the store carries this identifier.
    NoSuchRecord
    {
        node_id: String,
    },
    /// The node is known and no source document is recorded against it.
    NoContent
    {
        node_id: String,
    },
    /// More than one revision answers, so editing one of them would be a guess.
    Ambiguous
    {
        node_id: String,
        revisions: Vec<String>,
    },
    /// The document was ingested without a declared front matter row, so the store cannot
    /// say what its front matter said and cannot render it back.
    NotAuthored
    {
        path: String,
    },
    /// The staged text is not a readable record.
    Unreadable
    {
        cause: String,
    },
    /// The staged front matter names a different record.
    ///
    /// Refused rather than treated as a rename: `D-129` puts identity in the store, and a
    /// surface that let an author retype `id:` would make identity a property of the file
    /// again — the exact inversion the record is about.
    IdentityChanged
    {
        held: String,
        staged: String,
    },
    /// The staged text is not what the canonical layout would produce for it, so committing
    /// it would change bytes nobody asked to change.
    NotCanonical
    {
        cause: String,
    },
    /// The rename's destination already holds a document at this revision.
    PathTaken
    {
        path: String,
    },
    Store(StoreError),
}

impl core::fmt::Display for EditError
{
    // `fmt` is the fixed method name `std::fmt::Display` requires; it is not a style choice
    // and cannot be spelled out without ceasing to implement the trait.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoSuchRecord { node_id } => write!(formatter, "no node identified {node_id} is in this store"),
            Self::NoContent { node_id } => write!(
                formatter,
                "{node_id} is in this store as an identity with no source document behind it, \
                 so there is nothing to read out or edit"
            ),
            Self::Ambiguous { node_id, revisions } => write!(
                formatter,
                "{node_id} is held at {} revisions ({}); name one with a revision, because \
                 editing whichever came back first is a guess",
                revisions.len(), revisions.join(", ")
            ),
            Self::NotAuthored { path } => write!(
                formatter,
                "{path} has no declared front matter in this store, so its markdown cannot be \
                 rendered from the store's own rows. Documents ingested from a corpus arrive \
                 as blocks and lineage; only a record written through this door declares one"
            ),
            Self::Unreadable { cause } => write!(formatter, "the staged text is not a record: {cause}"),
            Self::IdentityChanged { held, staged } => write!(
                formatter,
                "the staged front matter is identified {staged} and the record being edited is \
                 {held}. Identity lives in the store, so it is not editable through the file"
            ),
            Self::NotCanonical { cause } => write!(
                formatter,
                "the staged text is not what this surface would write for it, so committing it \
                 would change bytes the edit did not ask to change: {cause}"
            ),
            Self::PathTaken { path } => write!(
                formatter,
                "{path} already names a document at this revision, so the rename would merge \
                 two records into one address"
            ),
            Self::Store(error) => write!(formatter, "{error}"),
        };
    }
}

impl std::error::Error for EditError
{}

impl From<StoreError> for EditError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl From<rusqlite::Error> for EditError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Store(StoreError::from(error));
    }
}

impl From<RenderError> for EditError
{
    fn from(error: RenderError) -> Self
    {
        return Self::NotCanonical {
            cause: error.to_string(),
        };
    }
}
