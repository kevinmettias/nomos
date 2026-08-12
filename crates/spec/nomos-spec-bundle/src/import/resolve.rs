//! Refusing a bundle that names something it does not itself carry.

use std::collections::BTreeSet;


use crate::BundleError;
use crate::bundle::Bundle;
use crate::document_ref::DocumentRef;
use crate::ordinal_ref::OrdinalRef;
use crate::record::Record;


/// Every reference the bundle makes is to something the bundle itself carries.
///
/// On an empty store this held for free: a reference to something absent found no row and
/// became [`BundleError::Unresolved`]. A store that already holds content withdraws that
/// for free — the same reference would find a row that was already there and silently bind
/// to it, and a bundle stitched onto rows it never mentioned is the merge this import
/// refuses to perform by accident. Answered from the bundle rather than the database, so
/// the answer does not depend on what the store happens to hold.
pub(super) fn Assert_Self_Contained(bundle: &Bundle) -> Result<(), BundleError>
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
pub(super) fn Document_Key_Of(path: &str, revision: &str) -> String
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
