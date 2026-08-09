#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod document;
mod index;
mod snapshot;
mod store;

pub use document::{Authority, Document, DocumentId, DocumentKind};
pub use index::Index;
pub use snapshot::{Manifest, Recorded, Reference, Snapshot, SNAPSHOT_SCHEMA};
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
                "snapshot {snapshot} records nothing. Refusing to commit it, because a \
                 snapshot that recorded nothing is indistinguishable from a run that never \
                 happened"
            ),
            Self::Malformed(cause) => write!(formatter, "malformed document: {cause}"),
        };
    }
}

impl std::error::Error for StoreError {}
