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
mod bundle_serialization;
mod export;
mod import;

pub use bundle_error::BundleError;
pub use row::blob::Blob;
pub use row::blob::encoding::Encoding;
pub use bundle::Bundle;
pub use bundle_serialization::BundleSerialization;
pub use row::reference::document_ref::DocumentRef;
pub use export::Export;
pub use bundle::header::{FORMAT, Header};
pub use import::{Import_Bundle, Report};
pub use bundle::lineage::Lineage;
pub use bundle::manifest::Manifest;
pub use row::node::Node;
pub use row::node::alias::Alias;
pub use row::node::history::History;
pub use row::normative_statement::NormativeStatement;
pub use bundle::omission::Omission;
pub use row::reference::ordinal_ref::OrdinalRef;
pub use row::record::Record;
pub use row::record::front_matter::FrontMatter;
// `Relation` would collide with `row::relation::Relation` (the typed-edge row) if flattened
// bare, so the declared-front-matter relation keeps its longer public name here.
pub use row::record::relation::Relation as RecordRelation;
pub use row::relation::Relation;
pub use row::relation::r#type::Type;
pub use row::source::block::Block;
pub use row::source::document::Document;
pub use row::source::heading::Heading;
pub use row::source::table_row::TableRow;
pub use row::suite::Suite;
pub use row::reference::table_row_ref::TableRowRef;
