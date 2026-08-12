use crate::BundleError;
use crate::bundle::Bundle;
use crate::blob::Blob;
use crate::blob_encoding::BlobEncoding;
use crate::document_ref::DocumentRef;
use crate::ordinal_ref::OrdinalRef;
use crate::record::Record;
use crate::table_row_ref::TableRowRef;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use nomos_spec_model::ContentHash;
use nomos_spec_store::{SpecificationStore, Table};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportReport
{
    pub records: u32,
    pub counts: BTreeMap<String, u32>,
}

/// Places a bundle's content in a store, beside whatever that store already holds.
///
/// It is not a merge and must never become one by accident. What guarantees that used to be
/// that the store was empty, which is a guarantee no store this build assembles can offer:
/// `Assemble` seeds the governing records first and unconditionally, so demanding emptiness
/// put the durable text form `OD-SPEC-008` names beyond the reach of the only stores that
/// exist. The guarantee is now stated over content instead — the bundle and the store must
/// name nothing in common, and the bundle must resolve its own references — which is the
/// same promise on a store that has been seeded.
///
/// # Errors
///
/// Returns [`BundleError::Occupied`] if the store already holds something the bundle
/// carries, [`BundleError::Unresolved`] if a record names something the bundle does not
/// carry, and [`BundleError::Incomplete`] if the import does not place exactly what the
/// bundle declared.
pub fn Import(store: &mut SpecificationStore, bundle: &Bundle) -> Result<ImportReport, BundleError>
{
    bundle.Verify_Counts()?;
    Assert_Same_Schema(store, bundle)?;

    // Self-containment first, because it reads the bundle alone and so gives the same
    // answer whatever the store holds. Disjointness then asks the one question that does
    // depend on the store.
    Assert_Self_Contained(bundle)?;
    Assert_Disjoint(store, bundle)?;

    let before = Census(store)?;

    return store.In_Transaction(|transaction| return Insert_All(transaction, bundle, &before));
}

/// The bundle and the store are at the same schema version.
///
/// Refused in both directions. A bundle from a newer store carries columns this build has
/// nowhere to put; one from an older store would need migrating, and importing it unmigrated
/// would be a guess about what the missing columns meant.
fn Assert_Same_Schema(store: &SpecificationStore, bundle: &Bundle) -> Result<(), BundleError>
{
    let supported = store.Version();
    let declared = bundle.Header().schema_version;

    if declared > supported
    {
        return Err(BundleError::TooNew {
            found: declared,
            supported,
        });
    }
    if declared < supported
    {
        return Err(BundleError::Malformed(format!(
            "the bundle is at schema version {declared} and this store is at {supported}; \
             bundle migration is not implemented, so importing it would guess"
        )));
    }

    return Ok(());
}

/// Everything the bundle carries, in one transaction, and the guard that all of it landed.
fn Insert_All(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    before: &BTreeMap<&'static str, u32>,
) -> Result<ImportReport, BundleError>
{
    // The relation-type vocabulary is self-referential (`inverse_of` names another row in the
    // same table), so no insertion order satisfies it. Deferring moves every foreign-key check
    // to commit without weakening any of them.
    transaction.pragma_update(None, "defer_foreign_keys", "ON")?;
    Insert_Sources(transaction, bundle)?;
    Insert_Graph(transaction, bundle)?;
    Insert_Declarations(transaction, bundle)?;
    Insert_Submission_Rows(transaction, bundle)?;
    Assert_Landed(transaction, bundle, before)?;

    return Ok(ImportReport {
        records: bundle.Manifest().records,
        counts: bundle.Manifest().counts.clone(),
    });
}

/// The bytes, and the documents cut from them.
fn Insert_Sources(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Blobs(transaction, bundle)?;
    Insert_Source_Documents(transaction, bundle)?;
    Insert_Source_Headings(transaction, bundle)?;
    Insert_Source_Blocks(transaction, bundle)?;

    return Insert_Source_Table_Rows(transaction, bundle);
}

/// The identities those documents were read into, and the edges between them.
fn Insert_Graph(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Suites(transaction, bundle)?;
    Insert_Nodes(transaction, bundle)?;
    Insert_Node_Aliases(transaction, bundle)?;
    Insert_Node_History(transaction, bundle)?;
    Insert_Relation_Types(transaction, bundle)?;
    Insert_Relations(transaction, bundle)?;
    Insert_Normative_Statements(transaction, bundle)?;
    Insert_Lineage(transaction, bundle)?;

    return Insert_Omissions(transaction, bundle);
}

/// What a record declared about itself in its own front matter.
fn Insert_Declarations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Record_Front_Matter(transaction, bundle)?;

    return Insert_Record_Relations(transaction, bundle);
}

/// The submissions, the values attributed to them, and the gaps left open.
fn Insert_Submission_Rows(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    Insert_Submissions(transaction, bundle)?;
    Insert_Submission_Values(transaction, bundle)?;

    return Insert_Submission_Gaps(transaction, bundle);
}

/// How many rows each table held before the import.
///
/// Taken rather than assumed zero. Once a store may already hold content, "the table is
/// empty afterwards" and "the import placed nothing" stopped being the same sentence, and
/// the completeness guard below is only a guard if it measures the difference.
fn Census(store: &SpecificationStore) -> Result<BTreeMap<&'static str, u32>, BundleError>
{
    let mut census: BTreeMap<&'static str, u32> = BTreeMap::new();
    for table in Table::All()
    {
        census.insert(table.Name(), store.Count(*table)?);
    }

    return Ok(census);
}

/// The store holds nothing this bundle also carries.
fn Assert_Disjoint(store: &SpecificationStore, bundle: &Bundle) -> Result<(), BundleError>
{
    let connection = store.Connection();

    for record in bundle.Records()
    {
        let Some(identity) = Collides(connection, record)?
        else
        {
            continue;
        };

        return Err(BundleError::Occupied {
            table: record.Table().to_owned(),
            identity,
        });
    }

    return Ok(());
}

/// What this record would arrive on top of, if the store already holds it.
fn Collides(connection: &Connection, record: &Record) -> Result<Option<String>, BundleError>
{
    let Some(stated) = Stated_Identity(record)
    else
    {
        return Ok(None);
    };
    let held = Already_Holds(connection, stated.sql, &stated.arguments)?;

    return Ok(held.then(|| return stated.identity));
}

/// How a record states its own identity: where a store would already hold it, what that
/// question binds, and how the identity reads in a refusal.
struct Stated<'a>
{
    sql: &'static str,
    arguments: Vec<&'a str>,
    identity: String,
}

/// The identity a record states in its own right, for the records that state one.
///
/// Only these tables are asked about. Every other table is reached through one of them — a
/// block through its document, a history entry through its node, a table row through its
/// block — so once these are disjoint a child row cannot collide either: the parent it hangs
/// from is one this import just inserted. The pass-through kinds are listed explicitly rather
/// than matched with `_`, so that a new record kind has to be thought about here instead of
/// defaulting to unchecked.
///
/// A submission's identity is the node it is, which `nodes` already collides on, so it is
/// deliberately not asked about a second time under its own table.
fn Stated_Identity(record: &Record) -> Option<Stated<'_>>
{
    return match record
    {
        Record::Blob(blob) => Some(By_One("SELECT 1 FROM blobs WHERE sha256 = ?1", &blob.sha256)),
        Record::SourceDocument(document) => Some(Stated {
            sql: "SELECT 1 FROM source_documents WHERE path = ?1 AND revision = ?2",
            arguments: vec![document.path.as_str(), document.revision.as_str()],
            identity: Document_Key_Of(&document.path, &document.revision),
        }),
        Record::Suite(suite) => Some(By_One("SELECT 1 FROM suites WHERE suite_id = ?1", &suite.suite_id)),
        Record::Node(node) => Some(By_One("SELECT 1 FROM nodes WHERE node_id = ?1", &node.node_id)),
        Record::NodeAlias(alias) => Some(By_One("SELECT 1 FROM node_aliases WHERE alias = ?1", &alias.alias)),
        Record::RelationType(relation_type) => Some(By_One(
            "SELECT 1 FROM relation_types WHERE name = ?1",
            &relation_type.name,
        )),
        Record::NormativeStatement(statement) => Some(By_One(
            "SELECT 1 FROM normative_statements WHERE statement_id = ?1",
            &statement.statement_id,
        )),
        Record::Submission(submission) => Some(By_One(
            "SELECT 1 FROM submissions s JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1",
            &submission.node_id,
        )),
        Record::SourceHeading(_) | Record::SourceBlock(_) | Record::SourceTableRow(_)
        | Record::NodeHistory(_) | Record::Relation(_) | Record::Lineage(_)
        | Record::Omission(_) | Record::RecordFrontMatter(_) | Record::RecordRelation(_)
        | Record::SubmissionValue(_) | Record::SubmissionGap(_) => None,
    };
}

/// A record whose identity is one column, and reads as that column's value.
fn By_One<'a>(sql: &'static str, key: &'a str) -> Stated<'a>
{
    return Stated {
        sql,
        arguments: vec![key],
        identity: key.to_owned(),
    };
}

/// `sql` is `&'static str` so that the statement cannot be built at runtime. Every collision
/// query is a literal written here, and the identity being tested is bound as an argument; the
/// type is what keeps it that way rather than a convention a later edit could quietly drop.
fn Already_Holds(
    connection: &Connection,
    sql: &'static str,
    arguments: &[&str],
) -> Result<bool, BundleError>
{
    let bound = rusqlite::params_from_iter(arguments);

    return Ok(connection
        .query_row(sql, bound, |row| return row.get::<_, i64>(0))
        .optional()?
        .is_some());
}

/// Every reference the bundle makes is to something the bundle itself carries.
///
/// On an empty store this held for free: a reference to something absent found no row and
/// became [`BundleError::Unresolved`]. A store that already holds content withdraws that
/// for free — the same reference would find a row that was already there and silently bind
/// to it, and a bundle stitched onto rows it never mentioned is the merge this import
/// refuses to perform by accident. Answered from the bundle rather than the database, so
/// the answer does not depend on what the store happens to hold.
fn Assert_Self_Contained(bundle: &Bundle) -> Result<(), BundleError>
{
    let carried = Carried_By(bundle);

    for record in bundle.Records()
    {
        Assert_Resolves(record, &carried)?;
    }

    return Ok(());
}

/// The identities a bundle states in its own right, indexed for lookup.
#[derive(Default)]
struct Identities
{
    blobs: BTreeSet<String>,
    documents: BTreeSet<String>,
    headings: BTreeSet<String>,
    blocks: BTreeSet<String>,
    rows: BTreeSet<String>,
    suites: BTreeSet<String>,
    nodes: BTreeSet<String>,
    relation_types: BTreeSet<String>,
    statements: BTreeSet<String>,
    submissions: BTreeSet<String>,
}

impl Identities
{
    /// Files a record under the identity it states in its own right, and passes over the
    /// records that state none. The pass-through kinds are listed rather than matched with
    /// `_`, so a new record kind has to be thought about here.
    fn Note(&mut self, record: &Record)
    {
        match record
        {
            Record::Blob(blob) => Keep(&mut self.blobs, blob.sha256.clone()),
            Record::SourceDocument(document) =>
            {
                let key = Document_Key_Of(&document.path, &document.revision);
                Keep(&mut self.documents, key);
            }
            Record::SourceHeading(heading) =>
            {
                let key = Ordinal_Key(&heading.document, heading.ordinal);
                Keep(&mut self.headings, key);
            }
            Record::SourceBlock(block) =>
            {
                let key = Ordinal_Key(&block.document, block.ordinal);
                Keep(&mut self.blocks, key);
            }
            Record::SourceTableRow(row) =>
            {
                let key = Row_Key(&row.block, row.ordinal);
                Keep(&mut self.rows, key);
            }
            Record::Suite(suite) => Keep(&mut self.suites, suite.suite_id.clone()),
            Record::Node(node) => Keep(&mut self.nodes, node.node_id.clone()),
            Record::RelationType(kind) => Keep(&mut self.relation_types, kind.name.clone()),
            Record::NormativeStatement(one) => Keep(&mut self.statements, one.statement_id.clone()),
            Record::Submission(submission) => Keep(&mut self.submissions, submission.node_id.clone()),
            Record::NodeAlias(_) | Record::NodeHistory(_) | Record::Relation(_)
            | Record::Lineage(_) | Record::Omission(_) | Record::RecordFrontMatter(_)
            | Record::RecordRelation(_) | Record::SubmissionValue(_)
            | Record::SubmissionGap(_) => {}
        }
    }
}

/// Files one identity under the set that states it.
fn Keep(identities: &mut BTreeSet<String>, key: String)
{
    identities.insert(key);
}

fn Carried_By(bundle: &Bundle) -> Identities
{
    let mut carried = Identities::default();

    for record in bundle.Records()
    {
        carried.Note(record);
    }

    return carried;
}

/// The three submission records' references.
///
/// One of the families [`Assert_Resolves`] dispatches to. The rule is the same one every arm
/// there states: a record may only name what the bundle carries.
fn Assert_Submission_Resolves(record: &Record, carried: &Identities) -> Result<(), BundleError>
{
    match record
    {
        // A submission is a node, so the node is what it must resolve against.
        Record::Submission(submission) =>
        {
            Carried(&carried.nodes, &submission.node_id, "node")?;
        }
        Record::SubmissionValue(value) =>
        {
            Carried(&carried.submissions, &value.node_id, "submission")?;
        }
        Record::SubmissionGap(gap) =>
        {
            Carried(&carried.submissions, &gap.node_id, "submission")?;
        }
        _ =>
        {}
    }

    return Ok(());
}

/// One record's references, each against what the bundle carries.
///
/// Exhaustive here and grouped by what the reference is about, because the whole of it in one
/// body was longer than anyone reads. Each group below is a `_ => {}` match over its own
/// family; this is the one place that decides which family a record kind belongs to, so a new
/// kind cannot slip through by defaulting.
fn Assert_Resolves(record: &Record, carried: &Identities) -> Result<(), BundleError>
{
    match record
    {
        Record::SourceDocument(_)
        | Record::SourceHeading(_)
        | Record::SourceBlock(_)
        | Record::SourceTableRow(_) => Assert_Source_Record_Resolves(record, carried)?,
        Record::Node(_)
        | Record::NodeAlias(_)
        | Record::NodeHistory(_)
        | Record::RelationType(_)
        | Record::Relation(_)
        | Record::NormativeStatement(_) => Assert_Graph_Record_Resolves(record, carried)?,
        Record::RecordFrontMatter(_) | Record::RecordRelation(_) =>
        {
            Assert_Declared_Resolves(record, carried)?;
        }
        Record::Submission(_) | Record::SubmissionValue(_) | Record::SubmissionGap(_) =>
        {
            Assert_Submission_Resolves(record, carried)?;
        }
        Record::Lineage(lineage) => Assert_Lineage_Resolves(lineage, carried)?,
        Record::Omission(omission) => Assert_Source_Resolves(
            omission.source_block.as_ref(),
            omission.source_heading.as_ref(),
            carried,
        )?,
        Record::Blob(_) | Record::Suite(_) =>
        {}
    }

    return Ok(());
}

/// A document names the blob it was read from, and everything cut from a document names the
/// document it was cut from.
fn Assert_Source_Record_Resolves(record: &Record, carried: &Identities)
    -> Result<(), BundleError>
{
    match record
    {
        Record::SourceDocument(document) =>
        {
            Carried(&carried.blobs, &document.blob_sha256, "blob")?;
        }
        Record::SourceHeading(heading) =>
        {
            let key = Document_Key(&heading.document);
            Carried(&carried.documents, &key, "source document")?;
        }
        Record::SourceBlock(block) =>
        {
            let key = Document_Key(&block.document);
            Carried(&carried.documents, &key, "source document")?;
        }
        Record::SourceTableRow(row) =>
        {
            let key = Ordinal_Key(&row.block.document, row.block.ordinal);
            Carried(&carried.blocks, &key, "source block")?;
        }
        _ =>
        {}
    }

    return Ok(());
}

/// The graph's own references: a node's suite, an alias and a history entry's node, a
/// relation's two ends and its type, and the inverse a relation type names.
fn Assert_Graph_Record_Resolves(record: &Record, carried: &Identities)
    -> Result<(), BundleError>
{
    match record
    {
        Record::Node(node) =>
        {
            if let Some(suite_id) = node.suite_id.as_deref()
            {
                Carried(&carried.suites, suite_id, "suite")?;
            }
        }
        Record::NodeAlias(alias) => Carried(&carried.nodes, &alias.node_id, "node")?,
        Record::NodeHistory(entry) => Carried(&carried.nodes, &entry.node_id, "node")?,
        Record::RelationType(relation_type) =>
        {
            if let Some(inverse) = relation_type.inverse_of.as_deref()
            {
                Carried(&carried.relation_types, inverse, "relation type")?;
            }
        }
        Record::Relation(relation) =>
        {
            Carried(&carried.nodes, &relation.from_node_id, "node")?;
            Carried(&carried.nodes, &relation.to_node_id, "node")?;
            Carried(&carried.relation_types, &relation.relation_type, "relation type")?;
        }
        Record::NormativeStatement(statement) =>
        {
            Carried(&carried.nodes, &statement.node_id, "node")?;
        }
        _ =>
        {}
    }

    return Ok(());
}

/// What a record declared about itself names: the document it was declared in, and the node
/// that declaration is about.
fn Assert_Declared_Resolves(record: &Record, carried: &Identities) -> Result<(), BundleError>
{
    match record
    {
        Record::RecordFrontMatter(front_matter) =>
        {
            let key = Document_Key(&front_matter.document);
            Carried(&carried.documents, &key, "source document")?;
            Carried(&carried.nodes, &front_matter.node_id, "node")?;
        }
        Record::RecordRelation(relation) =>
        {
            let key = Document_Key(&relation.document);
            Carried(&carried.documents, &key, "source document")?;
        }
        _ =>
        {}
    }

    return Ok(());
}

fn Assert_Lineage_Resolves(
    lineage: &crate::lineage::Lineage,
    carried: &Identities,
) -> Result<(), BundleError>
{
    Assert_Source_Resolves(
        lineage.source_block.as_ref(),
        lineage.source_heading.as_ref(),
        carried,
    )?;

    if let Some(row) = lineage.source_table_row.as_ref()
    {
        let key = Row_Key(&row.block, row.ordinal);
        Carried(&carried.rows, &key, "source table row")?;
    }
    if let Some(node_id) = lineage.target_node_id.as_deref()
    {
        Carried(&carried.nodes, node_id, "node")?;
    }
    if let Some(statement_id) = lineage.target_statement_id.as_deref()
    {
        Carried(&carried.statements, statement_id, "normative statement")?;
    }

    return Ok(());
}

/// The block and heading a lineage row and an omission row point at, which they spell
/// the same way and which the schema lets either of them leave null.
fn Assert_Source_Resolves(
    block: Option<&OrdinalRef>,
    heading: Option<&OrdinalRef>,
    carried: &Identities,
) -> Result<(), BundleError>
{
    if let Some(block) = block
    {
        let key = Ordinal_Key(&block.document, block.ordinal);
        Carried(&carried.blocks, &key, "source block")?;
    }
    if let Some(heading) = heading
    {
        let key = Ordinal_Key(&heading.document, heading.ordinal);
        Carried(&carried.headings, &key, "source heading")?;
    }

    return Ok(());
}

fn Carried(carried: &BTreeSet<String>, key: &str, kind: &str) -> Result<(), BundleError>
{
    if carried.contains(key)
    {
        return Ok(());
    }

    return Err(BundleError::Unresolved {
        record: kind.to_owned(),
        reference: key.to_owned(),
    });
}

fn Document_Key(document: &DocumentRef) -> String
{
    return Document_Key_Of(&document.path, &document.revision);
}

/// The same key from the two parts a [`SourceDocument`] carries loose rather than as a
/// [`DocumentRef`]. One spelling of the key, so the set and its lookups cannot drift.
fn Document_Key_Of(path: &str, revision: &str) -> String
{
    return format!("{path}@{revision}");
}

fn Ordinal_Key(document: &DocumentRef, ordinal: i64) -> String
{
    return format!("{}#{ordinal}", Document_Key(document));
}

fn Row_Key(block: &OrdinalRef, ordinal: i64) -> String
{
    return format!("{}.{ordinal}", Ordinal_Key(&block.document, block.ordinal));
}

/// The import placed exactly what the bundle said it would.
///
/// The mirror of the exporter's completeness guard: an insert that collapsed rows, or a
/// record kind nothing inserts, fails the import instead of producing a store that is
/// quietly smaller than its own bundle. Measured as a difference rather than a total,
/// because the rows that were already there are not this bundle's to account for.
fn Assert_Landed(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    before: &BTreeMap<&'static str, u32>,
) -> Result<(), BundleError>
{
    let after = Landed_Counts(transaction)?;

    for table in Table::All()
    {
        let landed = Added(&after, before, table.Name());
        let declared = bundle.Manifest().counts.get(table.Name()).copied().unwrap_or(0);

        if landed != declared
        {
            return Err(BundleError::Incomplete {
                table: table.Name().to_owned(),
                in_store: landed,
                exported: declared,
            });
        }
    }

    return Ok(());
}

/// How many rows this import added to one table.
fn Added(
    after: &BTreeMap<&'static str, u32>,
    before: &BTreeMap<&'static str, u32>,
    table: &'static str,
) -> u32
{
    let now = after.get(table).copied().unwrap_or(0);

    return now.saturating_sub(before.get(table).copied().unwrap_or(0));
}

/// Every table's tally in one statement.
///
/// Every arm is a `&'static str` the table itself carries, joined rather than assembled: no
/// value is woven into the statement text at any point, so there is nothing here for a caller
/// to reach.
fn Tally_Statement() -> String
{
    return Table::All()
        .iter()
        .map(|table| return table.Tally_Sql())
        .collect::<Vec<&'static str>>()
        .join(" UNION ALL ");
}

/// Every table's row count, in one crossing rather than one per table.
///
/// The counts are compared against the manifest afterwards, in memory. Asking each table
/// separately made the completeness guard cost one round trip per table for an answer that
/// one statement returns whole.
fn Landed_Counts(transaction: &Transaction<'_>) -> Result<BTreeMap<&'static str, u32>, BundleError>
{
    let tally = Tally_Statement();
    let mut statement = transaction.prepare(&tally)?;
    let counted = statement.query_map([], |row| {
        return Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?));
    })?;

    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for row in counted
    {
        let (which, tally) = row?;
        if let Some(table) = Table::All().iter().find(|table| return table.Name() == which)
        {
            counts.insert(table.Name(), tally);
        }
    }

    return Ok(counts);
}

/// Prepares one insert and offers every record the bundle carries to it.
///
/// Each importer below was the same frame — prepare, walk the records, skip the ones this
/// table is not about, resolve the parent row, execute — around the two lines that say
/// which variant and which columns. The frame is what they share and the reader is what
/// they do not, so the frame lives here and the reader stays at the call.
///
/// The reader returns `Ok(())` for a record of another kind, which is the skip.
fn Insert_Each<Bind>(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    sql: &str,
    mut bind: Bind,
) -> Result<(), BundleError>
where
    Bind: FnMut(&mut rusqlite::Statement<'_>, &Record) -> Result<(), BundleError>,
{
    let mut insert = transaction.prepare(sql)?;

    for record in bundle.Records()
    {
        bind(&mut insert, record)?;
    }

    return Ok(());
}

fn Insert_Blobs(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Blob(blob) = record
            else
            {
                return Ok(());
            };
            let bytes = Decoded(blob)?;
            Assert_Declared(blob, &bytes)?;
            insert.execute(params![blob.sha256, blob.byte_length, bytes])?;

            return Ok(());
        },
    );
}

/// A blob's bytes, in whichever form the bundle carried them.
fn Decoded(blob: &Blob) -> Result<Vec<u8>, BundleError>
{
    return match blob.encoding
    {
        BlobEncoding::Utf8 => Ok(blob.content.clone().into_bytes()),
        BlobEncoding::Base64 => STANDARD.decode(&blob.content).map_err(|error| {
            return BundleError::Malformed(format!("blob {}: {error}", blob.sha256));
        }),
    };
}

/// The bytes are what the blob said they were.
///
/// Both the digest and the length are checked, because they fail differently: a digest that
/// disagrees is content that changed on the way here, and a length that disagrees is a
/// bundle that miscounted what it was carrying.
fn Assert_Declared(blob: &Blob, bytes: &[u8]) -> Result<(), BundleError>
{
    let digest = ContentHash::Of_Bytes(bytes);

    if digest.As_Str() != blob.sha256
    {
        return Err(BundleError::Tampered {
            declared: blob.sha256.clone(),
            computed: digest.As_Str().to_owned(),
        });
    }
    if i64::try_from(bytes.len()).unwrap_or(i64::MAX) != blob.byte_length
    {
        return Err(BundleError::Malformed(format!(
            "blob {} declares {} byte(s) and carries {}",
            blob.sha256,
            blob.byte_length,
            bytes.len()
        )));
    }

    return Ok(());
}

fn Insert_Source_Documents(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::SourceDocument(document) = record
            else
            {
                return Ok(());
            };

            let blob_uid = Blob_Uid(transaction, &document.blob_sha256)?;
            insert.execute(params![document.path, document.revision, blob_uid])?;

            return Ok(());
        },
    );
}

fn Insert_Source_Headings(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
        |insert, record| {
            let Record::SourceHeading(heading) = record
            else
            {
                return Ok(());
            };

            let document_uid = Document_Uid(transaction, &heading.document)?;
            insert.execute(params![
                document_uid,
                heading.ordinal,
                heading.depth,
                heading.title
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Source_Blocks(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_blocks
         (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::SourceBlock(block) = record
            else
            {
                return Ok(());
            };

            let document_uid = Document_Uid(transaction, &block.document)?;
            insert.execute(params![
                document_uid,
                block.ordinal,
                block.kind,
                block.heading_path,
                block.text,
                block.content_hash,
                block.normalized_hash
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Source_Table_Rows(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_table_rows
         (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
          content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |insert, record| {
            let Record::SourceTableRow(row) = record
            else
            {
                return Ok(());
            };

            let block_uid = Block_Uid(transaction, &row.block)?;
            let cells = serde_json::to_string(&row.cells)
                .map_err(|error| BundleError::Sql(error.to_string()))?;
            insert.execute(params![
                block_uid,
                row.ordinal,
                row.table_ordinal,
                row.kind,
                cells,
                row.text,
                row.content_hash,
                row.normalized_hash
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Suites(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO suites (suite_id, title, authority_root) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Suite(suite) = record
            else
            {
                return Ok(());
            };

            insert.execute(params![
                suite.suite_id,
                suite.title,
                i64::from(suite.authority_root)
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Nodes(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO nodes
         (node_id, kind, authority, representation, title, deleted_at, suite_uid)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::Node(node) = record
            else
            {
                return Ok(());
            };

            let suite_uid = Optional_Suite_Uid(transaction, node.suite_id.as_deref())?;
            insert.execute(params![
                node.node_id,
                node.kind,
                node.authority,
                node.representation,
                node.title,
                node.deleted_at,
                suite_uid
            ])?;

            return Ok(());
        },
    );
}

fn Optional_Suite_Uid(
    transaction: &Transaction<'_>,
    suite_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return suite_id
        .map(|id| {
            return Resolve(
                transaction,
                "SELECT uid FROM suites WHERE suite_id = ?1",
                &[&id],
                Referenced {
                    record: "suite",
                    reference: id.to_owned(),
                },
            );
        })
        .transpose();
}

fn Insert_Node_Aliases(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    let mut insert =
        transaction.prepare("INSERT INTO node_aliases (alias, node_uid) VALUES (?1, ?2)")?;

    for record in bundle.Records()
    {
        let Record::NodeAlias(alias) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &alias.node_id)?;
        insert.execute(params![alias.alias, node_uid])?;
    }

    return Ok(());
}

fn Insert_Node_History(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO node_history
         (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::NodeHistory(entry) = record
            else
            {
                return Ok(());
            };

            let node_uid = Node_Uid(transaction, &entry.node_id)?;
            insert.execute(params![
                node_uid,
                entry.ordinal,
                entry.event,
                entry.reason,
                entry.previous_event_hash,
                entry.event_hash,
                entry.recorded_at
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Relation_Types(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    let mut insert = transaction
        .prepare("INSERT INTO relation_types (name, tier, inverse_of) VALUES (?1, ?2, ?3)")?;

    for record in bundle.Records()
    {
        let Record::RelationType(relation_type) = record
        else
        {
            continue;
        };

        insert.execute(params![
            relation_type.name,
            relation_type.tier,
            relation_type.inverse_of
        ])?;
    }

    return Ok(());
}

fn Insert_Relations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
         VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Relation(relation) = record
            else
            {
                return Ok(());
            };

            let from_uid = Node_Uid(transaction, &relation.from_node_id)?;
            let to_uid = Node_Uid(transaction, &relation.to_node_id)?;
            insert.execute(params![from_uid, relation.relation_type, to_uid])?;

            return Ok(());
        },
    );
}

fn Insert_Normative_Statements(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO normative_statements
         (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::NormativeStatement(statement) = record
            else
            {
                return Ok(());
            };

            let node_uid = Node_Uid(transaction, &statement.node_id)?;
            insert.execute(params![
                node_uid,
                statement.statement_id,
                statement.kind,
                statement.canonical_text,
                statement.canonical_hash,
                statement.supersedes_hash
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Lineage(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO lineage
         (source_block_uid, source_heading_uid, source_table_row_uid, disposition,
          target_node_uid, target_statement)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::Lineage(lineage) = record
            else
            {
                return Ok(());
            };
            let block_uid = Optional_Block_Uid(transaction, lineage.source_block.as_ref())?;
            let heading_uid = Optional_Heading_Uid(transaction, lineage.source_heading.as_ref())?;
            let row_uid = Optional_Table_Row_Uid(transaction, lineage.source_table_row.as_ref())?;
            let node_uid = Optional_Node_Uid(transaction, lineage.target_node_id.as_deref())?;
            let statement_uid =
                Optional_Statement_Uid(transaction, lineage.target_statement_id.as_deref())?;
            insert.execute(params![
                block_uid,
                heading_uid,
                row_uid,
                lineage.disposition,
                node_uid,
                statement_uid
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Omissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO omissions
         (source_block_uid, source_heading_uid, reason, justification, decision_record)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        |insert, record| {
            let Record::Omission(omission) = record
            else
            {
                return Ok(());
            };
            let block_uid = Optional_Block_Uid(transaction, omission.source_block.as_ref())?;
            let heading_uid = Optional_Heading_Uid(transaction, omission.source_heading.as_ref())?;
            insert.execute(params![
                block_uid,
                heading_uid,
                omission.reason,
                omission.justification,
                omission.decision_record
            ])?;

            return Ok(());
        },
    );
}

/// What a failed lookup names in its error: the kind of record that carried the reference,
/// and the reference exactly as it was written.
struct Referenced<'a>
{
    record: &'a str,
    reference: String,
}

/// A reference that does not resolve is an error, never a NULL. The distinction is the
/// whole point: a NULL here would leave a lineage row that counts as a row and points at
/// nothing.
/// `sql` is `&'static str` so that the statement cannot be built at runtime, and
/// `prepare_cached` so that resolving N references parses and plans the lookup once rather
/// than once per reference — these resolvers are called from inside the insert loops.
fn Resolve(
    transaction: &Transaction<'_>,
    sql: &'static str,
    arguments: &[&dyn rusqlite::ToSql],
    referenced: Referenced<'_>,
) -> Result<i64, BundleError>
{
    return transaction
        .prepare_cached(sql)
        .map_err(|error| return BundleError::Sql(error.to_string()))?
        .query_row(arguments, |row| row.get(0))
        .map_err(|error| match error
        {
            rusqlite::Error::QueryReturnedNoRows => BundleError::Unresolved {
                record: referenced.record.to_owned(),
                reference: referenced.reference,
            },
            other => BundleError::Sql(other.to_string()),
        });
}

fn Blob_Uid(transaction: &Transaction<'_>, sha256: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM blobs WHERE sha256 = ?1",
        &[&sha256],
        Referenced {
            record: "blob",
            reference: sha256.to_owned(),
        },
    );
}

fn Document_Uid(transaction: &Transaction<'_>, document: &DocumentRef) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        &[&document.path, &document.revision],
        Referenced {
            record: "source document",
            reference: format!("{}@{}", document.path, document.revision),
        },
    );
}

fn Node_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM nodes WHERE node_id = ?1",
        &[&node_id],
        Referenced {
            record: "node",
            reference: node_id.to_owned(),
        },
    );
}

fn Optional_Node_Uid(
    transaction: &Transaction<'_>,
    node_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return node_id.map(|id| Node_Uid(transaction, id)).transpose();
}

fn Optional_Statement_Uid(
    transaction: &Transaction<'_>,
    statement_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return statement_id
        .map(|id| {
            return Resolve(
                transaction,
                "SELECT uid FROM normative_statements WHERE statement_id = ?1",
                &[&id],
                Referenced {
                    record: "normative statement",
                    reference: id.to_owned(),
                },
            );
        })
        .transpose();
}

fn Block_Uid(transaction: &Transaction<'_>, reference: &OrdinalRef) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT b.uid FROM source_blocks b
         JOIN source_documents d ON d.uid = b.document_uid
         WHERE d.path = ?1 AND d.revision = ?2 AND b.ordinal = ?3",
        &[
            &reference.document.path,
            &reference.document.revision,
            &reference.ordinal,
        ],
        Referenced {
            record: "source block",
            reference: format!(
                "{}@{}#{}",
                reference.document.path, reference.document.revision, reference.ordinal
            ),
        },
    );
}

fn Optional_Table_Row_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&TableRowRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| {
            return Resolve(
                transaction,
                "SELECT r.uid FROM source_table_rows r
                 JOIN source_blocks b ON b.uid = r.source_block_uid
                 JOIN source_documents d ON d.uid = b.document_uid
                 WHERE d.path = ?1 AND d.revision = ?2 AND b.ordinal = ?3 AND r.ordinal = ?4",
                &[
                    &reference.block.document.path,
                    &reference.block.document.revision,
                    &reference.block.ordinal,
                    &reference.ordinal,
                ],
                Referenced {
                    record: "source table row",
                    reference: format!(
                        "{}@{}#{}.{}",
                        reference.block.document.path,
                        reference.block.document.revision,
                        reference.block.ordinal,
                        reference.ordinal
                    ),
                },
            );
        })
        .transpose();
}

fn Optional_Block_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&OrdinalRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| return Block_Uid(transaction, reference))
        .transpose();
}

fn Optional_Heading_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&OrdinalRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| {
            return Resolve(
                transaction,
                "SELECT h.uid FROM source_headings h
                 JOIN source_documents d ON d.uid = h.document_uid
                 WHERE d.path = ?1 AND d.revision = ?2 AND h.ordinal = ?3",
                &[
                    &reference.document.path,
                    &reference.document.revision,
                    &reference.ordinal,
                ],
                Referenced {
                    record: "source heading",
                    reference: format!(
                        "{}@{}#{}",
                        reference.document.path, reference.document.revision, reference.ordinal
                    ),
                },
            );
        })
        .transpose();
}

fn Insert_Record_Front_Matter(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO record_front_matter
         (document_uid, node_uid, status, version, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        |insert, record| {
            let Record::RecordFrontMatter(front_matter) = record
            else
            {
                return Ok(());
            };

            let tags = serde_json::to_string(&front_matter.tags)
                .map_err(|error| BundleError::Sql(error.to_string()))?;
            let document_uid = Document_Uid(transaction, &front_matter.document)?;
            let node_uid = Node_Uid(transaction, &front_matter.node_id)?;
            insert.execute(params![
                document_uid,
                node_uid,
                front_matter.status,
                front_matter.version,
                tags
            ])?;

            return Ok(());
        },
    );
}

fn Insert_Record_Relations(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO record_relations (document_uid, ordinal, target, relation)
         VALUES (?1, ?2, ?3, ?4)",
        |insert, record| {
            let Record::RecordRelation(relation) = record
            else
            {
                return Ok(());
            };
            let document_uid = Document_Uid(transaction, &relation.document)?;
            insert.execute(params![
                document_uid,
                relation.ordinal,
                relation.target,
                relation.relation
            ])?;

            return Ok(());
        },
    );
}

/// The submission rows, each onto the node it is.
fn Insert_Submissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submissions
             (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
         VALUES ((SELECT uid FROM nodes WHERE node_id = ?1), ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::Submission(submission) = record
            else
            {
                return Ok(());
            };
            insert.execute(params![
                submission.node_id,
                submission.kind,
                submission.form_contract_version,
                submission.state,
                submission.submitted_by,
                submission.submitted_through
            ])?;

            return Ok(());
        },
    );
}

/// Every attributed value, keeping the ordinal that decides which reading is current.
fn Insert_Submission_Values(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submission_values
             (submission_uid, field, ordinal, origin, value, value_hash, supersedes_hash,
              recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |insert, record| {
            let Record::SubmissionValue(value) = record
            else
            {
                return Ok(());
            };
            let submission_uid = Submission_Uid(transaction, &value.node_id)?;
            insert.execute(params![
                submission_uid,
                value.field,
                value.ordinal,
                value.origin,
                value.value,
                value.value_hash,
                value.supersedes_hash,
                value.recorded_at
            ])?;

            return Ok(());
        },
    );
}

/// Every gap, open and closed alike.
fn Insert_Submission_Gaps(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO submission_gaps
             (submission_uid, ordinal, question, blocks, severity, closed_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::SubmissionGap(gap) = record
            else
            {
                return Ok(());
            };
            let submission_uid = Submission_Uid(transaction, &gap.node_id)?;
            insert.execute(params![
                submission_uid,
                gap.ordinal,
                gap.question,
                gap.blocks,
                gap.severity,
                gap.closed_by
            ])?;

            return Ok(());
        },
    );
}

/// The surrogate of the submission filed under `node_id`.
fn Submission_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Ok(transaction.query_row(
        "SELECT s.uid FROM submissions s JOIN nodes n ON n.uid = s.node_uid
         WHERE n.node_id = ?1",
        params![node_id],
        |row| return row.get(0),
    )?);
}
