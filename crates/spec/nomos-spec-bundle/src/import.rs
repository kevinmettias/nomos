use crate::BundleError;
use crate::bundle::Bundle;
use crate::model::{BlobEncoding, DocumentRef, OrdinalRef, Record, TableRowRef};
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

    // Self-containment first, because it reads the bundle alone and so gives the same
    // answer whatever the store holds. Disjointness then asks the one question that does
    // depend on the store.
    Assert_Self_Contained(bundle)?;
    Assert_Disjoint(store, bundle)?;

    let before = Census(store)?;

    return store.In_Transaction(|transaction| {
        // The relation-type vocabulary is self-referential (`inverse_of` names another
        // row in the same table), so no insertion order satisfies it. Deferring moves
        // every foreign-key check to commit without weakening any of them.
        transaction.pragma_update(None, "defer_foreign_keys", "ON")?;

        Insert_Blobs(transaction, bundle)?;
        Insert_Source_Documents(transaction, bundle)?;
        Insert_Source_Headings(transaction, bundle)?;
        Insert_Source_Blocks(transaction, bundle)?;
        Insert_Source_Table_Rows(transaction, bundle)?;
        Insert_Suites(transaction, bundle)?;
        Insert_Nodes(transaction, bundle)?;
        Insert_Node_Aliases(transaction, bundle)?;
        Insert_Node_History(transaction, bundle)?;
        Insert_Relation_Types(transaction, bundle)?;
        Insert_Relations(transaction, bundle)?;
        Insert_Normative_Statements(transaction, bundle)?;
        Insert_Lineage(transaction, bundle)?;
        Insert_Omissions(transaction, bundle)?;
        Insert_Record_Front_Matter(transaction, bundle)?;
        Insert_Record_Relations(transaction, bundle)?;

        Assert_Landed(transaction, bundle, &before)?;

        return Ok(ImportReport {
            records: bundle.Manifest().records,
            counts: bundle.Manifest().counts.clone(),
        });
    });
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
///
/// Only the tables whose identity a bundle states in its own right are asked about. Every
/// other table is reached through one of these — a block through its document, a history
/// entry through its node, a table row through its block — so once these are disjoint a
/// child row cannot collide either: the parent it hangs from is one this import just
/// inserted. Listing the pass-through kinds explicitly rather than matching `_` so that a
/// new record kind has to be thought about here instead of defaulting to unchecked.
fn Assert_Disjoint(store: &SpecificationStore, bundle: &Bundle) -> Result<(), BundleError>
{
    let connection = store.Connection();

    for record in bundle.Records()
    {
        let collision = match record
        {
            Record::Blob(blob) => Already_Holds(
                connection,
                "SELECT 1 FROM blobs WHERE sha256 = ?1",
                &[&blob.sha256],
            )?
            .then(|| return blob.sha256.clone()),
            Record::SourceDocument(document) => Already_Holds(
                connection,
                "SELECT 1 FROM source_documents WHERE path = ?1 AND revision = ?2",
                &[&document.path, &document.revision],
            )?
            .then(|| return format!("{}@{}", document.path, document.revision)),
            Record::Suite(suite) => Already_Holds(
                connection,
                "SELECT 1 FROM suites WHERE suite_id = ?1",
                &[&suite.suite_id],
            )?
            .then(|| return suite.suite_id.clone()),
            Record::Node(node) => Already_Holds(
                connection,
                "SELECT 1 FROM nodes WHERE node_id = ?1",
                &[&node.node_id],
            )?
            .then(|| return node.node_id.clone()),
            Record::NodeAlias(alias) => Already_Holds(
                connection,
                "SELECT 1 FROM node_aliases WHERE alias = ?1",
                &[&alias.alias],
            )?
            .then(|| return alias.alias.clone()),
            Record::RelationType(relation_type) => Already_Holds(
                connection,
                "SELECT 1 FROM relation_types WHERE name = ?1",
                &[&relation_type.name],
            )?
            .then(|| return relation_type.name.clone()),
            Record::NormativeStatement(statement) => Already_Holds(
                connection,
                "SELECT 1 FROM normative_statements WHERE statement_id = ?1",
                &[&statement.statement_id],
            )?
            .then(|| return statement.statement_id.clone()),
            Record::SourceHeading(_)
            | Record::SourceBlock(_)
            | Record::SourceTableRow(_)
            | Record::NodeHistory(_)
            | Record::Relation(_)
            | Record::Lineage(_)
            | Record::Omission(_)
            | Record::RecordFrontMatter(_)
            | Record::RecordRelation(_) => None,
        };

        if let Some(identity) = collision
        {
            return Err(BundleError::Occupied {
                table: record.Table().to_owned(),
                identity,
            });
        }
    }

    return Ok(());
}

fn Already_Holds(
    connection: &Connection,
    sql: &str,
    arguments: &[&dyn rusqlite::ToSql],
) -> Result<bool, BundleError>
{
    return Ok(connection
        .query_row(sql, arguments, |row| return row.get::<_, i64>(0))
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
}

fn Carried_By(bundle: &Bundle) -> Identities
{
    let mut blobs: BTreeSet<String> = BTreeSet::new();
    let mut documents: BTreeSet<String> = BTreeSet::new();
    let mut headings: BTreeSet<String> = BTreeSet::new();
    let mut blocks: BTreeSet<String> = BTreeSet::new();
    let mut rows: BTreeSet<String> = BTreeSet::new();
    let mut suites: BTreeSet<String> = BTreeSet::new();
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    let mut relation_types: BTreeSet<String> = BTreeSet::new();
    let mut statements: BTreeSet<String> = BTreeSet::new();

    for record in bundle.Records()
    {
        match record
        {
            Record::Blob(blob) =>
            {
                blobs.insert(blob.sha256.clone());
            }
            Record::SourceDocument(document) =>
            {
                documents.insert(Document_Key_Of(&document.path, &document.revision));
            }
            Record::SourceHeading(heading) =>
            {
                headings.insert(Ordinal_Key(&heading.document, heading.ordinal));
            }
            Record::SourceBlock(block) =>
            {
                blocks.insert(Ordinal_Key(&block.document, block.ordinal));
            }
            Record::SourceTableRow(row) =>
            {
                rows.insert(Row_Key(&row.block, row.ordinal));
            }
            Record::Suite(suite) =>
            {
                suites.insert(suite.suite_id.clone());
            }
            Record::Node(node) =>
            {
                nodes.insert(node.node_id.clone());
            }
            Record::RelationType(relation_type) =>
            {
                relation_types.insert(relation_type.name.clone());
            }
            Record::NormativeStatement(statement) =>
            {
                statements.insert(statement.statement_id.clone());
            }
            Record::NodeAlias(_)
            | Record::NodeHistory(_)
            | Record::Relation(_)
            | Record::Lineage(_)
            | Record::Omission(_)
            | Record::RecordFrontMatter(_)
            | Record::RecordRelation(_) =>
            {}
        }
    }

    return Identities {
        blobs,
        documents,
        headings,
        blocks,
        rows,
        suites,
        nodes,
        relation_types,
        statements,
    };
}

/// One record's references, each against what the bundle carries.
fn Assert_Resolves(record: &Record, carried: &Identities) -> Result<(), BundleError>
{
    match record
    {
        Record::SourceDocument(document) =>
        {
            Carried(&carried.blobs, &document.blob_sha256, "blob")?;
        }
        Record::SourceHeading(heading) =>
        {
            Carried(
                &carried.documents,
                &Document_Key(&heading.document),
                "source document",
            )?;
        }
        Record::SourceBlock(block) =>
        {
            Carried(
                &carried.documents,
                &Document_Key(&block.document),
                "source document",
            )?;
        }
        Record::SourceTableRow(row) =>
        {
            Carried(
                &carried.blocks,
                &Ordinal_Key(&row.block.document, row.block.ordinal),
                "source block",
            )?;
        }
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
            Carried(
                &carried.relation_types,
                &relation.relation_type,
                "relation type",
            )?;
        }
        Record::NormativeStatement(statement) =>
        {
            Carried(&carried.nodes, &statement.node_id, "node")?;
        }
        Record::Lineage(lineage) => Assert_Lineage_Resolves(lineage, carried)?,
        Record::Omission(omission) =>
        {
            Assert_Source_Resolves(
                omission.source_block.as_ref(),
                omission.source_heading.as_ref(),
                carried,
            )?;
        }
        Record::RecordFrontMatter(front_matter) =>
        {
            Carried(
                &carried.documents,
                &Document_Key(&front_matter.document),
                "source document",
            )?;
            Carried(&carried.nodes, &front_matter.node_id, "node")?;
        }
        Record::RecordRelation(relation) =>
        {
            Carried(
                &carried.documents,
                &Document_Key(&relation.document),
                "source document",
            )?;
        }
        Record::Blob(_) | Record::Suite(_) =>
        {}
    }

    return Ok(());
}

fn Assert_Lineage_Resolves(
    lineage: &crate::model::Lineage,
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
        Carried(
            &carried.rows,
            &Row_Key(&row.block, row.ordinal),
            "source table row",
        )?;
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
        Carried(
            &carried.blocks,
            &Ordinal_Key(&block.document, block.ordinal),
            "source block",
        )?;
    }
    if let Some(heading) = heading
    {
        Carried(
            &carried.headings,
            &Ordinal_Key(&heading.document, heading.ordinal),
            "source heading",
        )?;
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
    for table in Table::All()
    {
        let sql = format!("SELECT count(*) FROM {}", table.Name());
        let after: u32 = transaction.query_row(&sql, [], |row| row.get(0))?;
        let landed = after.saturating_sub(before.get(table.Name()).copied().unwrap_or(0));
        let declared = bundle
            .Manifest()
            .counts
            .get(table.Name())
            .copied()
            .unwrap_or(0);

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

fn Insert_Blobs(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Blob(blob) = record
        else
        {
            continue;
        };

        let bytes = match blob.encoding
        {
            BlobEncoding::Utf8 => blob.content.clone().into_bytes(),
            BlobEncoding::Base64 => STANDARD
                .decode(&blob.content)
                .map_err(|error| BundleError::Malformed(format!("blob {}: {error}", blob.sha256)))?,
        };

        let digest = ContentHash::Of_Bytes(&bytes);
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

        transaction.execute(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
            params![blob.sha256, blob.byte_length, bytes],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Documents(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceDocument(document) = record
        else
        {
            continue;
        };

        let blob_uid = Blob_Uid(transaction, &document.blob_sha256)?;
        transaction.execute(
            "INSERT INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
            params![document.path, document.revision, blob_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Headings(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceHeading(heading) = record
        else
        {
            continue;
        };

        let document_uid = Document_Uid(transaction, &heading.document)?;
        transaction.execute(
            "INSERT INTO source_headings (document_uid, ordinal, depth, title)
             VALUES (?1, ?2, ?3, ?4)",
            params![document_uid, heading.ordinal, heading.depth, heading.title],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Blocks(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceBlock(block) = record
        else
        {
            continue;
        };

        let document_uid = Document_Uid(transaction, &block.document)?;
        transaction.execute(
            "INSERT INTO source_blocks
             (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                document_uid,
                block.ordinal,
                block.kind,
                block.heading_path,
                block.text,
                block.content_hash,
                block.normalized_hash
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Table_Rows(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceTableRow(row) = record
        else
        {
            continue;
        };

        let block_uid = Block_Uid(transaction, &row.block)?;
        let cells = serde_json::to_string(&row.cells)
            .map_err(|error| BundleError::Sql(error.to_string()))?;
        transaction.execute(
            "INSERT INTO source_table_rows
             (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
              content_hash, normalized_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                block_uid,
                row.ordinal,
                row.table_ordinal,
                row.kind,
                cells,
                row.text,
                row.content_hash,
                row.normalized_hash
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Suites(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Suite(suite) = record
        else
        {
            continue;
        };

        transaction.execute(
            "INSERT INTO suites (suite_id, title, authority_root) VALUES (?1, ?2, ?3)",
            params![suite.suite_id, suite.title, i64::from(suite.authority_root)],
        )?;
    }

    return Ok(());
}

fn Insert_Nodes(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Node(node) = record
        else
        {
            continue;
        };

        let suite_uid = Optional_Suite_Uid(transaction, node.suite_id.as_deref())?;
        transaction.execute(
            "INSERT INTO nodes
             (node_id, kind, authority, representation, title, deleted_at, suite_uid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                node.node_id,
                node.kind,
                node.authority,
                node.representation,
                node.title,
                node.deleted_at,
                suite_uid
            ],
        )?;
    }

    return Ok(());
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
                "suite",
                id.to_owned(),
            );
        })
        .transpose();
}

fn Insert_Node_Aliases(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NodeAlias(alias) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &alias.node_id)?;
        transaction.execute(
            "INSERT INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
            params![alias.alias, node_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Node_History(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NodeHistory(entry) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &entry.node_id)?;
        transaction.execute(
            "INSERT INTO node_history
             (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                node_uid,
                entry.ordinal,
                entry.event,
                entry.reason,
                entry.previous_event_hash,
                entry.event_hash,
                entry.recorded_at
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Relation_Types(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::RelationType(relation_type) = record
        else
        {
            continue;
        };

        transaction.execute(
            "INSERT INTO relation_types (name, tier, inverse_of) VALUES (?1, ?2, ?3)",
            params![
                relation_type.name,
                relation_type.tier,
                relation_type.inverse_of
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Relations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Relation(relation) = record
        else
        {
            continue;
        };

        let from_uid = Node_Uid(transaction, &relation.from_node_id)?;
        let to_uid = Node_Uid(transaction, &relation.to_node_id)?;
        transaction.execute(
            "INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             VALUES (?1, ?2, ?3)",
            params![from_uid, relation.relation_type, to_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Normative_Statements(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NormativeStatement(statement) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &statement.node_id)?;
        transaction.execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                node_uid,
                statement.statement_id,
                statement.kind,
                statement.canonical_text,
                statement.canonical_hash,
                statement.supersedes_hash
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Lineage(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Lineage(lineage) = record
        else
        {
            continue;
        };

        let block_uid = Optional_Block_Uid(transaction, lineage.source_block.as_ref())?;
        let heading_uid = Optional_Heading_Uid(transaction, lineage.source_heading.as_ref())?;
        let row_uid = Optional_Table_Row_Uid(transaction, lineage.source_table_row.as_ref())?;
        let node_uid = Optional_Node_Uid(transaction, lineage.target_node_id.as_deref())?;
        let statement_uid =
            Optional_Statement_Uid(transaction, lineage.target_statement_id.as_deref())?;

        transaction.execute(
            "INSERT INTO lineage
             (source_block_uid, source_heading_uid, source_table_row_uid, disposition,
              target_node_uid, target_statement)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                block_uid,
                heading_uid,
                row_uid,
                lineage.disposition,
                node_uid,
                statement_uid
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Omissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Omission(omission) = record
        else
        {
            continue;
        };

        let block_uid = Optional_Block_Uid(transaction, omission.source_block.as_ref())?;
        let heading_uid = Optional_Heading_Uid(transaction, omission.source_heading.as_ref())?;

        transaction.execute(
            "INSERT INTO omissions
             (source_block_uid, source_heading_uid, reason, justification, decision_record)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block_uid,
                heading_uid,
                omission.reason,
                omission.justification,
                omission.decision_record
            ],
        )?;
    }

    return Ok(());
}

/// A reference that does not resolve is an error, never a NULL. The distinction is the
/// whole point: a NULL here would leave a lineage row that counts as a row and points at
/// nothing.
fn Resolve(
    transaction: &Transaction<'_>,
    sql: &str,
    arguments: &[&dyn rusqlite::ToSql],
    record: &str,
    reference: String,
) -> Result<i64, BundleError>
{
    return transaction
        .query_row(sql, arguments, |row| row.get(0))
        .map_err(|error| match error
        {
            rusqlite::Error::QueryReturnedNoRows => BundleError::Unresolved {
                record: record.to_owned(),
                reference,
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
        "blob",
        sha256.to_owned(),
    );
}

fn Document_Uid(transaction: &Transaction<'_>, document: &DocumentRef) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        &[&document.path, &document.revision],
        "source document",
        format!("{}@{}", document.path, document.revision),
    );
}

fn Node_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM nodes WHERE node_id = ?1",
        &[&node_id],
        "node",
        node_id.to_owned(),
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
                "normative statement",
                id.to_owned(),
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
        "source block",
        format!(
            "{}@{}#{}",
            reference.document.path, reference.document.revision, reference.ordinal
        ),
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
                "source table row",
                format!(
                    "{}@{}#{}.{}",
                    reference.block.document.path,
                    reference.block.document.revision,
                    reference.block.ordinal,
                    reference.ordinal
                ),
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
                "source heading",
                format!(
                    "{}@{}#{}",
                    reference.document.path, reference.document.revision, reference.ordinal
                ),
            );
        })
        .transpose();
}

fn Insert_Record_Front_Matter(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::RecordFrontMatter(front_matter) = record
        else
        {
            continue;
        };

        let tags = serde_json::to_string(&front_matter.tags)
            .map_err(|error| BundleError::Sql(error.to_string()))?;
        transaction.execute(
            "INSERT INTO record_front_matter
             (document_uid, node_uid, status, version, tags_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Document_Uid(transaction, &front_matter.document)?,
                Node_Uid(transaction, &front_matter.node_id)?,
                front_matter.status,
                front_matter.version,
                tags
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Record_Relations(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::RecordRelation(relation) = record
        else
        {
            continue;
        };

        transaction.execute(
            "INSERT INTO record_relations (document_uid, ordinal, target, relation)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                Document_Uid(transaction, &relation.document)?,
                relation.ordinal,
                relation.target,
                relation.relation
            ],
        )?;
    }

    return Ok(());
}
