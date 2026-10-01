//! Reading a record back out of the store's own rows.
//!
//! The write path is [`crate::authoring`]'s four steps. This is the other half: the rendered
//! markdown, the declared front matter, the declared relations, and what became of each
//! normative statement recorded against a record. Every one of them answers from the store's
//! own tables rather than from an ingested blob, which is what makes the projection a claim
//! about the store rather than an echo of what went into it.

use nomos_spec_model::{
    BlockKind, ContentHash, FrontMatter as RecordFrontMatter, RecordRelation, Render_Record,
    SourceBlock,
};
use rusqlite::{OptionalExtension, params};

use crate::DocumentSource;
use crate::EditError;
use crate::NormativeMovement;
use crate::RecordProjection;
use crate::SpecificationStore;
use crate::StoreError;
use crate::read::columns::Columns;
use crate::store::Collected_Rows;

impl SpecificationStore
{
    /// A record, rendered back out of the store's own rows as markdown.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] naming why no single record answered.
    pub fn Record_Markdown(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<RecordProjection, EditError>
    {
        let document = self.Sole_Document(node_id, revision)?;

        return self.Projection_Of(&document);
    }

    /// The blocks of one document, in the order they were authored.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Stored_Blocks(&self, document_uid: i64) -> Result<Vec<SourceBlock>, StoreError>
    {
        use crate::record::Kind_Of;

        let mut statement = self.Connection().prepare(
            "SELECT ordinal, kind, heading_path, text FROM source_blocks
             WHERE document_uid = ?1 ORDER BY ordinal",
        )?;
        let rows = statement.query_map(params![document_uid], |row| {
            let mut columns = Columns::Of(row);
            let ordinal = columns.Next()?;
            let kind: String = columns.Next()?;
            let path: String = columns.Next()?;

            return Ok(SourceBlock {
                ordinal,
                kind: Kind_Of(&kind).unwrap_or(BlockKind::Prose),
                heading_path: Heading_Path(&path),
                text: columns.Next()?,
            });
        })?;

        return Collected_Rows(rows);
    }

    /// The front matter a document declared, as it declared it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Declared_Front_Matter(
        &self,
        document_uid: i64,
    ) -> Result<Option<RecordFrontMatter>, StoreError>
    {
        let found = self
            .Connection()
            .query_row(
                "SELECT node.node_id, node.kind, node.title, node.authority,
                        front.status, front.version, front.tags_json
                 FROM record_front_matter front
                 JOIN nodes node ON node.uid = front.node_uid
                 WHERE front.document_uid = ?1",
                params![document_uid],
                Declared_Row,
            )
            .optional()?;
        let Some(declared) = found
        else
        {
            return Ok(None);
        };

        return Ok(Some(RecordFrontMatter {
            id: declared.id,
            kind: declared.kind,
            title: declared.title,
            status: declared.status,
            version: declared.version,
            authority: declared.authority,
            tags: serde_json::from_str(&declared.tags).unwrap_or_default(),
            relations: self.Declared_Relations(document_uid)?,
        }));
    }

    /// The relations a document declared, in the order it declared them.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Declared_Relations(
        &self,
        document_uid: i64,
    ) -> Result<Vec<RecordRelation>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT target, relation FROM record_relations
             WHERE document_uid = ?1 ORDER BY ordinal",
        )?;
        let rows = statement.query_map(params![document_uid], |row| {
            return Ok(RecordRelation {
                target: row.get(0)?,
                relation: row.get(1)?,
            });
        })?;

        return Collected_Rows(rows);
    }

    /// What became of each normative statement recorded against this record.
    pub(crate) fn Statement_Movements(
        &self,
        node_id: &str,
        before: &[SourceBlock],
        after: &[SourceBlock],
    ) -> Result<Vec<NormativeMovement>, StoreError>
    {
        use crate::authoring::difference::Located_Statement;

        let mut movements = Vec::new();
        for (statement_id, canonical_text, canonical_hash) in self.Recorded_Statements(node_id)?
        {
            movements.push(NormativeMovement {
                statement_id,
                canonical_hash,
                outcome: Located_Statement(&canonical_text, before, after),
            });
        }

        return Ok(movements);
    }

    /// Every normative statement recorded against a record: its identifier, its canonical
    /// text, and what that text hashes to.
    fn Recorded_Statements(&self, node_id: &str) -> Result<Vec<Recorded>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT s.statement_id, s.canonical_text, s.canonical_hash
             FROM normative_statements s
             JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1
             ORDER BY s.statement_id",
        )?;
        let rows = statement.query_map(params![node_id], |row| {
            let mut columns = Columns::Of(row);
            return Ok((columns.Next()?, columns.Next()?, columns.Next()?));
        })?;

        return Collected_Rows(rows);
    }

    /// The one document behind an identifier, or why there is not exactly one.
    ///
    /// `pub(super)` because [`SpecificationStore::Claim_For_Edit`] in the parent module asks
    /// the same question and must get the same answer; a second walk over `Documents_Behind`
    /// would be a second place that could disagree about what "one document" means.
    pub(super) fn Sole_Document(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<DocumentSource, EditError>
    {
        let documents = self.Documents_Behind(node_id, revision)?;

        if let [only] = documents.as_slice()
        {
            return Ok(only.clone());
        }
        if documents.len() > 1
        {
            return Err(Too_Many_Revisions(node_id, &documents));
        }

        return Err(self.Nothing_To_Read(node_id)?);
    }

    /// Which refusal a record with no document behind it deserves: an identity this store
    /// holds and has no content for, or no identity at all.
    fn Nothing_To_Read(&self, node_id: &str) -> Result<EditError, StoreError>
    {
        if self.Node_Summary(node_id)?.is_some()
        {
            return Ok(EditError::NoContent {
                node_id: node_id.to_owned(),
            });
        }

        return Ok(EditError::NoSuchRecord {
            node_id: node_id.to_owned(),
        });
    }

    /// The markdown for one document, rendered from its rows.
    pub(super) fn Projection_Of(
        &self,
        document: &DocumentSource,
    ) -> Result<RecordProjection, EditError>
    {
        let front_matter = self
            .Declared_Front_Matter(document.uid)?
            .ok_or_else(|| {
                return EditError::NotAuthored {
                    path: document.path.clone(),
                };
            })?;
        let blocks = self.Stored_Blocks(document.uid)?;
        let markdown = Render_Record(&front_matter, &blocks)?;

        return Ok(RecordProjection {
            node_id: front_matter.id,
            path: document.path.clone(),
            revision: document.revision.clone(),
            projected_hash: ContentHash::Of(&markdown).As_String_Slice().to_owned(),
            source_hash: document.content_hash.clone(),
            markdown,
        });
    }
}

/// A normative statement as the store keeps it: identifier, canonical text, canonical hash.
type Recorded = (String, String, String);

/// More than one revision answered, so editing whichever came back first would be a guess.
fn Too_Many_Revisions(node_id: &str, documents: &[DocumentSource]) -> EditError
{
    return EditError::Ambiguous {
        node_id: node_id.to_owned(),
        revisions: documents
            .iter()
            .map(|document| return document.revision.clone())
            .collect(),
    };
}

/// The row a document's declared front matter joins to.
struct DeclaredRow
{
    id: String,
    kind: String,
    title: String,
    authority: String,
    status: String,
    version: u32,
    tags: String,
}

fn Declared_Row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeclaredRow>
{
    let mut columns = Columns::Of(row);

    return Ok(DeclaredRow {
        id: columns.Next()?,
        kind: columns.Next()?,
        title: columns.Next()?,
        authority: columns.Next()?,
        status: columns.Next()?,
        version: columns.Next()?,
        tags: columns.Next()?,
    });
}

fn Heading_Path(stored: &str) -> Vec<String>
{
    if stored.is_empty()
    {
        return Vec::new();
    }

    return stored.split(" / ").map(str::to_owned).collect();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Seed_Governing_Records;
    use crate::store::AUTHORED;

    /// A window over the block list wide enough to hold two blocks, which is the smallest
    /// window an ordering claim can be made about.
    const ADJACENT_BLOCKS: usize = 2;

    #[test]
    fn Test_Record_Markdown_Should_Render_The_Same_Bytes_Back_Out()
    {
        let store = With_Synthetic();

        let projection = store
            .Record_Markdown("D-900", None)
            .expect("Put_Record gave D-900 a declared front matter and blocks");

        assert_eq!(projection.markdown, CANONICAL);
        assert!(projection.Is_Matching_Source());
    }

    #[test]
    fn Test_Stored_Blocks_Should_Return_Blocks_In_Authored_Order()
    {
        let store = With_Synthetic();
        let document = store
            .Documents_Behind("D-900", None)
            .expect("Put_Record wrote D-900 as one document in this store")
            .into_iter()
            .next()
            .expect("Documents_Behind returned the one document Put_Record wrote");

        let blocks = store
            .Stored_Blocks(document.uid)
            .expect("Put_Record wrote this document's blocks into the store");

        assert!(!blocks.is_empty());
        assert!(blocks.windows(ADJACENT_BLOCKS).all(|pair| {
            return pair.first().expect("windows(2) yields two-element slices").ordinal
                < pair.get(1).expect("windows(2) yields two-element slices").ordinal;
        }));
    }

    #[test]
    fn Test_Declared_Front_Matter_Should_Return_What_The_Record_Declared()
    {
        let store = With_Synthetic();
        let document = store
            .Documents_Behind("D-900", None)
            .expect("Put_Record wrote D-900 as one document in this store")
            .into_iter()
            .next()
            .expect("Documents_Behind returned the one document Put_Record wrote");

        let front_matter = store
            .Declared_Front_Matter(document.uid)
            .expect("the document Put_Record wrote has a declared front matter row")
            .expect("an authored document declares front matter");

        assert_eq!(front_matter.id, "D-900");
        assert_eq!(front_matter.status, "accepted");
    }

    #[test]
    fn Test_Declared_Relations_Should_Return_Relations_In_Authored_Order()
    {
        let store = With_Synthetic();
        let document = store
            .Documents_Behind("D-900", None)
            .expect("Put_Record wrote D-900 as one document in this store")
            .into_iter()
            .next()
            .expect("Documents_Behind returned the one document Put_Record wrote");

        let relations = store
            .Declared_Relations(document.uid)
            .expect("Put_Record wrote the relation D-900 declared into the store");

        assert_eq!(relations.len(), 1);
        assert_eq!(
            relations.first().expect("asserted above to contain exactly one relation").target,
            "D-129"
        );
    }

    /// Nothing in this crate writes `normative_statements` yet, so a real store's answer is
    /// the empty one asserted here — not a stand-in for a fixture this crate cannot build.
    #[test]
    fn Test_Statement_Movements_Should_Report_None_When_Nothing_Was_Recorded()
    {
        let store = With_Synthetic();

        let movements = store
            .Statement_Movements("D-900", &[], &[])
            .expect("the statement query runs against this store's own tables");

        assert!(movements.is_empty());
    }

    const CANONICAL: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                             status: accepted\nversion: 1\n\
                             authority: canonical-normative-record\ntags:\n  - testing\n\
                             relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                             # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                             ## Rationale\n\nSecond paragraph.\n";
    const PATH: &str = "docs/records/D-900-a-synthetic-record.md";

    fn Seeded() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory()
            .expect("In_Memory builds its own schema, so opening touches no file");
        Seed_Governing_Records(&mut store)
            .expect("the governing record set is compiled in, so seeding reads no file");
        return store;
    }

    fn With_Synthetic() -> SpecificationStore
    {
        let mut store = Seeded();
        store.Put_Record(PATH, AUTHORED, CANONICAL).expect("writes the synthetic record");
        return store;
    }
}
