#![forbid(unsafe_code)]

// A document's identity and its kind live beneath `document`, which is what they are
// parts of. Flat, this level was eleven files with that relationship spelled only in
// their name prefixes.
mod authority;
mod commit;
mod document;
mod document_store;
mod index;
mod manifest;
mod recorded;
mod reference;
mod store_error;

pub use authority::Authority;
pub use document::{Document, DocumentId, DocumentKind};
pub use index::Index;
pub use commit::{Commit, COMMIT_SCHEMA};
pub use manifest::Manifest;
pub use recorded::Recorded;
pub use reference::Reference;
pub use document_store::DocumentStore;
pub use store_error::StoreError;
