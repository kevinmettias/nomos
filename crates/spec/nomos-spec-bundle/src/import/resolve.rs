//! Refusing a bundle that names something it does not itself carry.

use std::collections::BTreeSet;

use crate::BundleError;
use crate::Bundle;
use crate::DocumentRef;
use crate::OrdinalRef;
use crate::Record;

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
            Record::Blob(blob) => Keep_Identity(&mut self.blobs, blob.sha256.clone()),
            Record::SourceDocument(document) =>
            {
                let key = Document_Key_Of(Path(&document.path), Revision(&document.revision));
                Keep_Identity(&mut self.documents, key);
            }
            Record::SourceHeading(heading) =>
            {
                let key = Ordinal_Key(&heading.document, heading.ordinal);
                Keep_Identity(&mut self.headings, key);
            }
            Record::SourceBlock(block) =>
            {
                let key = Ordinal_Key(&block.document, block.ordinal);
                Keep_Identity(&mut self.blocks, key);
            }
            Record::SourceTableRow(row) =>
            {
                let key = Row_Key(&row.block, row.ordinal);
                Keep_Identity(&mut self.rows, key);
            }
            Record::Suite(suite) => Keep_Identity(&mut self.suites, suite.suite_id.clone()),
            Record::Node(node) => Keep_Identity(&mut self.nodes, node.node_id.clone()),
            Record::RelationType(kind) => Keep_Identity(&mut self.relation_types, kind.name.clone()),
            Record::NormativeStatement(one) => Keep_Identity(&mut self.statements, one.statement_id.clone()),
            Record::Submission(submission) => Keep_Identity(&mut self.submissions, submission.node_id.clone()),
            Record::NodeAlias(_) | Record::NodeHistory(_) | Record::Relation(_)
            | Record::Lineage(_) | Record::Omission(_) | Record::RecordFrontMatter(_)
            | Record::RecordRelation(_) | Record::SubmissionValue(_)
            | Record::SubmissionGap(_) => {}
        }
    }
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
            Assert_Carried(&carried.blobs, Key(&document.blob_sha256), Kind("blob"))?;
        }
        Record::SourceHeading(heading) =>
        {
            let key = Document_Key(&heading.document);
            Assert_Carried(&carried.documents, Key(&key), Kind("source document"))?;
        }
        Record::SourceBlock(block) =>
        {
            let key = Document_Key(&block.document);
            Assert_Carried(&carried.documents, Key(&key), Kind("source document"))?;
        }
        Record::SourceTableRow(row) =>
        {
            let key = Ordinal_Key(&row.block.document, row.block.ordinal);
            Assert_Carried(&carried.blocks, Key(&key), Kind("source block"))?;
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
                Assert_Carried(&carried.suites, Key(suite_id), Kind("suite"))?;
            }
        }
        Record::NodeAlias(alias) => Assert_Carried(&carried.nodes, Key(&alias.node_id), Kind("node"))?,
        Record::NodeHistory(entry) => Assert_Carried(&carried.nodes, Key(&entry.node_id), Kind("node"))?,
        Record::RelationType(relation_type) =>
        {
            if let Some(inverse) = relation_type.inverse_of.as_deref()
            {
                Assert_Carried(&carried.relation_types, Key(inverse), Kind("relation type"))?;
            }
        }
        Record::Relation(relation) =>
        {
            Assert_Carried(&carried.nodes, Key(&relation.from_node_id), Kind("node"))?;
            Assert_Carried(&carried.nodes, Key(&relation.to_node_id), Kind("node"))?;
            Assert_Carried(&carried.relation_types, Key(&relation.relation_type), Kind("relation type"))?;
        }
        Record::NormativeStatement(statement) =>
        {
            Assert_Carried(&carried.nodes, Key(&statement.node_id), Kind("node"))?;
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
            Assert_Carried(&carried.documents, Key(&key), Kind("source document"))?;
            Assert_Carried(&carried.nodes, Key(&front_matter.node_id), Kind("node"))?;
        }
        Record::RecordRelation(relation) =>
        {
            let key = Document_Key(&relation.document);
            Assert_Carried(&carried.documents, Key(&key), Kind("source document"))?;
        }
        _ =>
        {}
    }

    return Ok(());
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
            Assert_Carried(&carried.nodes, Key(&submission.node_id), Kind("node"))?;
        }
        Record::SubmissionValue(value) =>
        {
            Assert_Carried(&carried.submissions, Key(&value.node_id), Kind("submission"))?;
        }
        Record::SubmissionGap(gap) =>
        {
            Assert_Carried(&carried.submissions, Key(&gap.node_id), Kind("submission"))?;
        }
        _ =>
        {}
    }

    return Ok(());
}

fn Assert_Lineage_Resolves(
    lineage: &crate::bundle::lineage::Lineage,
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
        Assert_Carried(&carried.rows, Key(&key), Kind("source table row"))?;
    }
    if let Some(node_id) = lineage.target_node_id.as_deref()
    {
        Assert_Carried(&carried.nodes, Key(node_id), Kind("node"))?;
    }
    if let Some(statement_id) = lineage.target_statement_id.as_deref()
    {
        Assert_Carried(&carried.statements, Key(statement_id), Kind("normative statement"))?;
    }

    return Ok(());
}

fn Row_Key(block: &OrdinalRef, ordinal: i64) -> String
{
    return format!("{}.{ordinal}", Ordinal_Key(&block.document, block.ordinal));
}

/// Files one identity under the set that states it.
fn Keep_Identity(identities: &mut BTreeSet<String>, key: String)
{
    identities.insert(key);
}

/// A document's path, kept distinct from [`Revision`] so the two positions of
/// [`Document_Key_Of`] cannot be swapped at a call site.
pub(super) struct Path<'a>(pub(super) &'a str);

/// A document's revision, kept distinct from [`Path`] for the same reason.
pub(super) struct Revision<'a>(pub(super) &'a str);

/// The same key from the two parts a [`crate::Document`] carries loose rather than as a
/// [`DocumentRef`]. One spelling of the key, so the set and its lookups cannot drift.
pub(super) fn Document_Key_Of(path: Path<'_>, revision: Revision<'_>) -> String
{
    return format!("{}@{}", path.0, revision.0);
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
        Assert_Carried(&carried.blocks, Key(&key), Kind("source block"))?;
    }
    if let Some(heading) = heading
    {
        let key = Ordinal_Key(&heading.document, heading.ordinal);
        Assert_Carried(&carried.headings, Key(&key), Kind("source heading"))?;
    }

    return Ok(());
}

/// The identity a reference names, kept distinct from [`Kind`] so the two cannot be
/// swapped at a call site — both are plain strings and nothing else would tell them apart.
struct Key<'a>(&'a str);

/// What a reference is naming, reported in [`BundleError::Unresolved`] alongside the key
/// that did not resolve.
struct Kind<'a>(&'a str);

fn Assert_Carried(carried: &BTreeSet<String>, key: Key<'_>, kind: Kind<'_>) -> Result<(), BundleError>
{
    if carried.contains(key.0)
    {
        return Ok(());
    }

    return Err(BundleError::Unresolved {
        record: kind.0.to_owned(),
        reference: key.0.to_owned(),
    });
}

fn Document_Key(document: &DocumentRef) -> String
{
    return Document_Key_Of(Path(&document.path), Revision(&document.revision));
}

fn Ordinal_Key(document: &DocumentRef, ordinal: i64) -> String
{
    return format!("{}#{ordinal}", Document_Key(document));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Assert_Self_Contained_Should_Accept_A_Node_Naming_A_Suite_The_Bundle_Carries()
    {
        let bundle = Bundle::New(
            1,
            vec![
                Record::Suite(crate::Suite {
                    suite_id: "nomos".to_owned(),
                    title: "The Nomos Specification".to_owned(),
                    authority_root: true,
                }),
                Record::Node(crate::Node {
                    node_id: "N1".to_owned(),
                    kind: "requirement".to_owned(),
                    authority: "canonical".to_owned(),
                    representation: "record".to_owned(),
                    title: "Node One".to_owned(),
                    deleted_at: None,
                    suite_id: Some("nomos".to_owned()),
                }),
            ],
        )
        .expect("builds");

        assert!(Assert_Self_Contained(&bundle).is_ok());
    }

    #[test]
    fn Test_Assert_Self_Contained_Should_Refuse_A_Node_Naming_A_Suite_It_Does_Not_Carry()
    {
        let bundle = Bundle::New(
            1,
            vec![Record::Node(crate::Node {
                node_id: "N1".to_owned(),
                kind: "requirement".to_owned(),
                authority: "canonical".to_owned(),
                representation: "record".to_owned(),
                title: "Node One".to_owned(),
                deleted_at: None,
                suite_id: Some("ghost".to_owned()),
            })],
        )
        .expect("builds");

        let refusal =
            Assert_Self_Contained(&bundle).expect_err("a reference to an uncarried suite must be refused");
        assert!(
            matches!(refusal, BundleError::Unresolved { ref record, .. } if record == "suite"),
            "{refusal}"
        );
    }

    #[test]
    fn Test_Document_Key_Of_Should_Join_Path_And_Revision_With_An_At_Sign()
    {
        let key = Document_Key_Of(Path("doc.md"), Revision("v1"));

        assert_eq!(key, "doc.md@v1");
    }
}
