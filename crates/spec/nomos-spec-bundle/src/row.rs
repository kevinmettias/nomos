//! One type per row of the bundle, and the tables they belong to.
//!
//! The flat level this replaces was twenty-nine files, of which twenty-two were a row
//! type and seven were the bundle itself and how it moves. `columns.rs` names every
//! table and column, and `export.rs` refuses a table it did not emit, so the set of
//! row types is a contract -- but the set being fixed is not an argument for it being
//! flat, and node, record, relation, source, submission and the three reference
//! shapes each had a group already spelled in their name prefixes.
//!
//! Declared `pub(crate) mod` rather than re-exported name by name, so a reader of
//! `crate::row::source::heading::Heading` sees which table it is a row of.

pub(crate) mod blob;
pub(crate) mod node;
pub(crate) mod normative_statement;
pub(crate) mod record;
pub(crate) mod reference;
pub(crate) mod relation;
pub(crate) mod source;
pub(crate) mod submission;
pub(crate) mod suite;
