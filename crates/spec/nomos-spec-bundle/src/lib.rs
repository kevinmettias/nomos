//! Band 1 — the portable specification bundle.
//!
//! The database is the working authority; this is the authority committed to git. Both
//! must say the same thing, which is why the round trip is a property and not a feature:
//! database to export to import to export yields byte-identical text.

#![forbid(unsafe_code)]

mod bundle;
mod columns;
mod row;
mod determinism;
mod export;
mod import;

pub use row::blob::Blob;
pub use row::blob::encoding::BlobEncoding;
pub use bundle::Bundle;
pub use determinism::BundleSerialization;
pub use row::reference::document::DocumentRef;
pub use export::Export;
pub use bundle::header::{FORMAT, Header};
pub use import::{Import, ImportReport};
pub use bundle::lineage::Lineage;
pub use bundle::manifest::Manifest;
pub use row::node::Node;
pub use row::node::alias::NodeAlias;
pub use row::node::history::NodeHistory;
pub use row::normative_statement::NormativeStatement;
pub use bundle::omission::Omission;
pub use row::reference::ordinal::OrdinalRef;
pub use row::record::Record;
pub use row::record::front_matter::RecordFrontMatter;
pub use row::record::relation::RecordRelation;
pub use row::relation::Relation;
pub use row::relation::kind::RelationType;
pub use row::source::block::SourceBlock;
pub use row::source::document::SourceDocument;
pub use row::source::heading::SourceHeading;
pub use row::source::table_row::SourceTableRow;
pub use row::suite::Suite;
pub use row::reference::table_row::TableRowRef;

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
            Self::Incomplete { table, in_store, exported } => Say_Incomplete(formatter, table, *in_store, *exported),
            Self::UncoveredColumn { table, column } => Say_Uncovered_Column(formatter, table, column),
            Self::PhantomColumn { table, column } => write!(
                formatter,
                "the exporter claims to carry {table}.{column}, which the schema does not have"
            ),
            Self::Uncarried { table, column, field } => Say_Uncarried(formatter, table, column, field),
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
            Self::NotCanonical { line } => Say_Not_Canonical(formatter, *line),
            Self::Malformed(cause) => write!(formatter, "malformed bundle: {cause}"),
            Self::TooNew { found, supported } => write!(
                formatter,
                "the bundle is format {found}; this build understands {supported}"
            ),
            Self::Unresolved { record, reference } => write!(
                formatter,
                "{record} names {reference}, which the bundle does not contain"
            ),
            Self::Occupied { table, identity } => Say_Occupied(formatter, table, identity),
        };
    }
}

/// The row guard: a table holds rows the export did not emit.
fn Say_Incomplete(
    formatter: &mut core::fmt::Formatter<'_>,
    table: &str,
    in_store: u32,
    exported: u32,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{table} holds {in_store} row(s) but the export emitted {exported}. \
         Refusing to write a bundle that is missing content"
    );
}

/// The column guard: every row counts and each is missing a field.
fn Say_Uncovered_Column(
    formatter: &mut core::fmt::Formatter<'_>,
    table: &str,
    column: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{table}.{column} reaches no bundle record. Refusing to write a bundle whose \
         rows all count and are each missing a field"
    );
}

/// A declaration naming a field the record does not have.
fn Say_Uncarried(
    formatter: &mut core::fmt::Formatter<'_>,
    table: &str,
    column: &str,
    field: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{table}.{column} claims to travel as `{field}`, which is not a field of a \
         {table} record"
    );
}

/// A bundle no regeneration would reproduce.
fn Say_Not_Canonical(formatter: &mut core::fmt::Formatter<'_>, line: usize) -> core::fmt::Result
{
    return write!(
        formatter,
        "line {line} is not the canonical serialization of its own content. A bundle \
         that cannot be regenerated to a fixpoint is a gate that can never go green"
    );
}

/// Import places a bundle beside what a store holds rather than merging into it.
fn Say_Occupied(
    formatter: &mut core::fmt::Formatter<'_>,
    table: &str,
    identity: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "the store already holds {table} {identity}, which this bundle also carries. \
         Import places a bundle beside what a store holds rather than merging into \
         it, so it refuses rather than deciding which of the two is right"
    );
}

impl std::error::Error for BundleError
{}

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
