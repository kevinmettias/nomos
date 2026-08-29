//! Band 1 — the portable specification bundle.
//!
//! The database is the working authority; this is the authority committed to git. Both
//! must say the same thing, which is why the round trip is a property and not a feature:
//! database to export to import to export yields byte-identical text.

#![forbid(unsafe_code)]

mod bundle;
mod bundle_error;
mod columns;
mod row;
mod determinism;
mod export;
mod import;

pub use bundle_error::BundleError;
pub use row::blob::Blob;
pub use row::blob::blob_encoding::BlobEncoding;
pub use bundle::Bundle;
pub use determinism::BundleSerialization;
pub use row::reference::document_ref::DocumentRef;
pub use export::Export;
pub use bundle::header::{FORMAT, Header};
pub use import::{Import_Bundle, ImportReport};
pub use bundle::lineage::Lineage;
pub use bundle::manifest::Manifest;
pub use row::node::Node;
pub use row::node::node_alias::NodeAlias;
pub use row::node::node_history::NodeHistory;
pub use row::normative_statement::NormativeStatement;
pub use bundle::omission::Omission;
pub use row::reference::ordinal_ref::OrdinalRef;
pub use row::record::Record;
pub use row::record::record_front_matter::RecordFrontMatter;
pub use row::record::record_relation::RecordRelation;
pub use row::relation::Relation;
pub use row::relation::relation_type::RelationType;
pub use row::source::source_block::SourceBlock;
pub use row::source::source_document::SourceDocument;
pub use row::source::source_heading::SourceHeading;
pub use row::source::source_table_row::SourceTableRow;
pub use row::suite::Suite;
pub use row::reference::table_row_ref::TableRowRef;
