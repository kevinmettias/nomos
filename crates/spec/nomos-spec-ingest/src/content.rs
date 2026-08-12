//! What the ingest recovered out of the archive, by the shape it was recovered as.
//!
//! A block, a section and a statement each carry their own lineage, their own recorded
//! form and their own mismatches, and flat at the crate root those eleven files said so
//! only in their name prefixes.

pub(crate) mod block;
pub(crate) mod section;
pub(crate) mod statement;
