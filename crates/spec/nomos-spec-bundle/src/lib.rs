//! Band 1 — the portable specification bundle.
//!
//! The database is the working authority; this is the authority committed to git. Both
//! must say the same thing, which is why the round trip is a property and not a feature:
//! database to export to import to export yields byte-identical text.

#![forbid(unsafe_code)]

mod bundle;
mod columns;
mod determinism;
mod export;
mod import;
mod model;

pub use bundle::{Bundle, FORMAT, Header, Manifest};
pub use determinism::BundleSerialization;
pub use export::Export;
pub use import::{Import, ImportReport};
pub use model::{
    Blob, BlobEncoding, DocumentRef, Lineage, Node, NodeAlias, NodeHistory, NormativeStatement,
    Omission, OrdinalRef, Record, RecordFrontMatter, RecordRelation, Relation, RelationType,
    SourceBlock, SourceDocument, SourceHeading, SourceTableRow, Suite, TableRowRef,
};

/// Everything that stops a bundle from being written, read or trusted.
#[derive(Debug)]
pub enum BundleError
{
    Store(nomos_spec_store::StoreError),
    Sql(String),
    Json(String),
    /// A table holds rows the exporter did not emit.
    ///
    /// The guard that makes silent loss structurally impossible: adding a table to the
    /// schema without teaching the exporter about it fails the export instead of
    /// producing a bundle that is quietly missing content.
    Incomplete
    {
        table: String,
        in_store: u32,
        exported: u32,
    },
    /// A column the exporter does not carry.
    ///
    /// The loss the row count cannot see. Every row exported, one field short, and both
    /// sides of the round trip equally blind to the difference.
    UncoveredColumn
    {
        table: String,
        column: String,
    },
    /// A declared column the schema does not have.
    PhantomColumn
    {
        table: String,
        column: String,
    },
    /// A declared column names a record field that does not exist.
    Uncarried
    {
        table: String,
        column: String,
        field: String,
    },
    /// The digest in the manifest does not match the content it covers.
    Tampered
    {
        declared: String,
        computed: String,
    },
    /// The manifest's record counts disagree with the records present.
    Miscounted
    {
        table: String,
        declared: u32,
        present: u32,
    },
    /// Well-formed JSON, but not the canonical serialization of itself.
    NotCanonical
    {
        line: usize,
    },
    Malformed(String),
    /// A newer format or schema than this build understands.
    TooNew
    {
        found: u32,
        supported: u32,
    },
    /// A reference names something the bundle does not contain.
    Unresolved
    {
        record: String,
        reference: String,
    },
    /// The store already holds something the bundle carries.
    ///
    /// Import places a bundle beside what a store holds and never merges into it; merging
    /// is a different operation with different conflict semantics, and doing it by
    /// accident is how one authority becomes two. An empty store was the old way of
    /// guaranteeing that, and it is too strong: every store this build assembles is seeded
    /// with the governing records first, so requiring emptiness put the durable form
    /// `OD-SPEC-008` names out of reach of the only stores that exist. Disjointness is the
    /// same guarantee stated over the content rather than over the table.
    Occupied
    {
        table: String,
        identity: String,
    },
}

impl core::fmt::Display for BundleError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Sql(cause) => write!(formatter, "bundle sql error: {cause}"),
            Self::Json(cause) => write!(formatter, "bundle json error: {cause}"),
            Self::Incomplete {
                table,
                in_store,
                exported,
            } => write!(
                formatter,
                "{table} holds {in_store} row(s) but the export emitted {exported}. \
                 Refusing to write a bundle that is missing content"
            ),
            Self::UncoveredColumn { table, column } => write!(
                formatter,
                "{table}.{column} reaches no bundle record. Refusing to write a bundle whose \
                 rows all count and are each missing a field"
            ),
            Self::PhantomColumn { table, column } => write!(
                formatter,
                "the exporter claims to carry {table}.{column}, which the schema does not have"
            ),
            Self::Uncarried {
                table,
                column,
                field,
            } => write!(
                formatter,
                "{table}.{column} claims to travel as `{field}`, which is not a field of a \
                 {table} record"
            ),
            Self::Tampered { declared, computed } => write!(
                formatter,
                "the manifest declares {declared} but the content hashes to {computed}"
            ),
            Self::Miscounted {
                table,
                declared,
                present,
            } => write!(
                formatter,
                "the manifest declares {declared} {table} record(s); {present} are present"
            ),
            Self::NotCanonical { line } => write!(
                formatter,
                "line {line} is not the canonical serialization of its own content. A bundle \
                 that cannot be regenerated to a fixpoint is a gate that can never go green"
            ),
            Self::Malformed(cause) => write!(formatter, "malformed bundle: {cause}"),
            Self::TooNew { found, supported } => write!(
                formatter,
                "the bundle is format {found}; this build understands {supported}"
            ),
            Self::Unresolved { record, reference } => write!(
                formatter,
                "{record} names {reference}, which the bundle does not contain"
            ),
            Self::Occupied { table, identity } => write!(
                formatter,
                "the store already holds {table} {identity}, which this bundle also carries. \
                 Import places a bundle beside what a store holds rather than merging into \
                 it, so it refuses rather than deciding which of the two is right"
            ),
        };
    }
}

impl std::error::Error for BundleError {}

impl From<nomos_spec_store::StoreError> for BundleError
{
    fn from(error: nomos_spec_store::StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl From<rusqlite::Error> for BundleError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Sql(error.to_string());
    }
}

impl From<serde_json::Error> for BundleError
{
    fn from(error: serde_json::Error) -> Self
    {
        return Self::Json(error.to_string());
    }
}
