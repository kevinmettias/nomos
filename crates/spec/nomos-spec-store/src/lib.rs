//! Band 1 — the specification store.
//!
//! Three identifiers, never conflated: `node_id` is identity and travels in a bundle,
//! `uid` is a join surrogate and is never exported, `content_hash` is a content address.
//! Paths are navigation.

#![forbid(unsafe_code)]

mod record;
mod schema;
mod store;

pub use record::{Disposition, Kind_Label};
pub use schema::{Latest_Version, MIGRATIONS, Migration};
pub use store::{SpecificationStore, StoreError, Table};
