//! The specification store: one `SQLite` database, opened, migrated and queried.

// Seeding this build's own governing records into a store. It sits here rather than under
// `registration` because the build script reaches registration.rs by #[path] and compiles
// it with no crate around it, so a child of registration cannot name `crate::`.
mod seed_report;

pub use seed_report::{GOVERNING_RECORD_IDS, Seed_Governing_Records, SeedReport};

pub(crate) mod store_error;

pub(crate) mod relation;
mod specification_store;
mod write;

pub(crate) use write::{
    Collected_Rows, Inverse_Of, Write_Blob, Write_Node, Write_Relation, Write_Source_Blocks,
    Write_Source_Document,
};

pub use specification_store::SpecificationStore;

/// The revision label for content this repository authors itself.
///
/// Distinct from an ingested corpus revision like `v14.36`, so a query can tell what
/// Nomos said about itself from what it read out of an archive.
pub const AUTHORED: &str = "authored";

/// The authority of a node that exists only because something points at it.
pub const EXTERNAL: &str = "external";
