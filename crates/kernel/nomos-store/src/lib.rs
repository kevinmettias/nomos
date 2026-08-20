#![forbid(unsafe_code)]

// A document's identity and its kind live beneath `document`, which is what they are
// parts of. Flat, this level was eleven files with that relationship spelled only in
// their name prefixes.
mod authority;
mod commit;
mod document;
mod index;
mod manifest;
mod recorded;
mod reference;
mod store;

pub use authority::Authority;
pub use document::{Document, DocumentId, DocumentKind};
pub use index::Index;
pub use commit::{Commit, COMMIT_SCHEMA};
pub use manifest::Manifest;
pub use recorded::Recorded;
pub use reference::Reference;
pub use store::DocumentStore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError
{
    NoSuchDocument
    {
        document: DocumentId,
    },
    WrongAuthority
    {
        store: Authority,
        kind: DocumentKind,
        document: Authority,
    },
    Vacuous
    {
        snapshot: String,
    },
    Malformed(String),
}

impl core::fmt::Display for StoreError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoSuchDocument { document } => write!(
                formatter,
                "no document {document}. Absent is not empty: nothing recorded this"
            ),
            Self::WrongAuthority {
                store,
                kind,
                document,
            } => write!(
                formatter,
                "a {} store cannot record a {} document, which is {}. Observed and authored \
                 content never share a store, or one authority's writes become the other's \
                 evidence",
                store.Label(),
                kind.Label(),
                document.Label()
            ),
            Self::Vacuous { snapshot } => write!(
                formatter,
                "a commit under snapshot {snapshot} records nothing. Refusing it, because a \
                 commit that recorded nothing is indistinguishable from a run that never \
                 happened"
            ),
            Self::Malformed(cause) => write!(formatter, "malformed document: {cause}"),
        };
    }
}

impl std::error::Error for StoreError
{}
