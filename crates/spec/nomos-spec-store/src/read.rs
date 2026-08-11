//! Reading content back out, addressed the way a person addresses it.
//!
//! Nothing here writes. These are the queries a reader asks — *what did this record say*,
//! *what rows does that table hold* — and they live beside the schema rather than in the
//! caller, because a second place that knows the join from a node to the bytes it came
//! from is a second place that can be wrong about it.
//!
//! The three identifiers stay separate. A caller names a `node_id` or a document path;
//! `uid` is handed back only so a follow-up query can be scoped to the same document, and
//! never printed.

use crate::store::{Collected, SpecificationStore, StoreError};
use rusqlite::{OptionalExtension, params};

/// The stored bytes as text, or why this document cannot be read out as one.
fn Readable_Text(path: &str, bytes: Vec<u8>) -> Result<String, StoreError>
{
    return String::from_utf8(bytes).map_err(|error| {
        return StoreError::Record {
            path: path.to_owned(),
            cause: format!(
                "the stored bytes are not valid UTF-8 ({error}), so this document cannot be \
                 read out as text"
            ),
        };
    });
}

/// A node as the graph holds it, with no content behind it.
///
/// Separate from [`RecordSource`] because a node with no source document is a real and
/// common answer — the catalog mints thousands of them — and reporting that as "not
/// found" would tell a reader the identifier is unknown when the store knows it well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSummary
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
}

/// One source document, and the bytes it was ingested from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSource
{
    /// The join surrogate, so a caller can scope a second query to this document without
    /// resolving the path again. Never exported and never printed.
    pub uid: i64,
    pub path: String,
    pub revision: String,
    /// The document's content address.
    pub content_hash: String,
    /// The document exactly as it was ingested, byte for byte.
    pub text: String,
}

/// One line of a table, as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableLine
{
    /// Which block of the document carries it.
    pub block_ordinal: u32,
    /// Which table within that block. A block may carry more than one.
    pub table_ordinal: u32,
    /// Which line within the block, 1-based.
    pub row_ordinal: u32,
    /// `header`, `content` or `separator`.
    pub kind: String,
    pub cells: Vec<String>,
    /// The line as authored.
    pub text: String,
    pub content_hash: String,
}

/// How a document path was matched.
///
/// Reported rather than swallowed, so a caller who typed a fragment and got one document
/// can see it was a fragment that matched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathMatch
{
    /// The path was given in full.
    Exact,
    /// The last segment of the path was given.
    FileName,
    /// The text appears somewhere in the path.
    Fragment,
}

impl PathMatch
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Exact => "exact path",
            Self::FileName => "file name",
            Self::Fragment => "path fragment",
        };
    }
}

impl SpecificationStore
{
    /// The node behind an identifier, if the graph holds one.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Node_Summary(&self, node_id: &str) -> Result<Option<NodeSummary>, StoreError>
    {
        return Ok(self
            .Connection()
            .query_row(
                "SELECT node_id, kind, authority, representation, title
                 FROM nodes WHERE node_id = ?1 AND deleted_at IS NULL",
                params![node_id],
                |row| {
                    return Ok(NodeSummary {
                        node_id: row.get(0)?,
                        kind: row.get(1)?,
                        authority: row.get(2)?,
                        representation: row.get(3)?,
                        title: row.get(4)?,
                    });
                },
            )
            .optional()?);
    }

    /// Every source document whose content was disposed to a node, verbatim.
    ///
    /// A vector rather than an option, and unfiltered by default. One identifier can be
    /// carried by the same document at two revisions, and picking one of them silently
    /// would answer "what did this say" with "what one of these said" — which is the
    /// question the whole preservation ledger exists to keep answerable.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Record`] if a stored document is not valid UTF-8, and
    /// [`StoreError`] on any SQL failure.
    pub fn Documents_Behind(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<Vec<DocumentSource>, StoreError>
    {
        let uids = self.Documents_Disposed_To(node_id)?;
        let mut documents = Vec::new();

        for uid in uids
        {
            if let Some(document) = self.Document(uid)?
                && revision.is_none_or(|wanted| return document.revision == wanted)
            {
                documents.push(document);
            }
        }
        documents.sort_by(|left, right| {
            return (&left.revision, &left.path).cmp(&(&right.revision, &right.path));
        });

        return Ok(documents);
    }

    /// The surrogate of every document a block disposed to this node was cut from.
    fn Documents_Disposed_To(&self, node_id: &str) -> Result<Vec<i64>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT DISTINCT block.document_uid
             FROM lineage line
             JOIN source_blocks block ON block.uid = line.source_block_uid
             JOIN nodes node ON node.uid = line.target_node_uid
             WHERE node.node_id = ?1",
        )?;
        let found = statement.query_map(params![node_id], |row| return row.get::<usize, i64>(0))?;

        return Collected(found);
    }

    /// One source document by surrogate, with its bytes.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Record`] if the stored bytes are not valid UTF-8, and
    /// [`StoreError`] on any SQL failure.
    pub fn Document(&self, uid: i64) -> Result<Option<DocumentSource>, StoreError>
    {
        let found: Option<(String, String, String, Vec<u8>)> = self
            .Connection()
            .query_row(
                "SELECT document.path, document.revision, blob.sha256, blob.content
                 FROM source_documents document
                 JOIN blobs blob ON blob.uid = document.blob_uid
                 WHERE document.uid = ?1",
                params![uid],
                |row| return Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let Some((path, revision, content_hash, bytes)) = found
        else
        {
            return Ok(None);
        };
        let text = Readable_Text(&path, bytes)?;

        return Ok(Some(DocumentSource {
            uid,
            path,
            revision,
            content_hash,
            text,
        }));
    }

    /// The documents a path or fragment names, and how it named them.
    ///
    /// Tiered rather than a single `LIKE`. A caller who gives a whole path means that
    /// document, and returning it alongside three others that merely contain the text
    /// would make the precise form of the question the least useful one.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Documents_Named(
        &self,
        needle: &str,
        revision: Option<&str>,
    ) -> Result<(Vec<i64>, PathMatch), StoreError>
    {
        let candidates = self.Paths_At(revision)?;

        for tier in [PathMatch::Exact, PathMatch::FileName, PathMatch::Fragment]
        {
            let matched: Vec<i64> = candidates
                .iter()
                .filter(|(_, path)| return Matches(path, needle, tier))
                .map(|(uid, _)| return *uid)
                .collect();

            if !matched.is_empty()
            {
                return Ok((matched, tier));
            }
        }

        return Ok((Vec::new(), PathMatch::Exact));
    }

    /// Every document's surrogate and path, at one revision or across all of them.
    fn Paths_At(&self, revision: Option<&str>) -> Result<Vec<(i64, String)>, StoreError>
    {
        let mut statement = self
            .Connection()
            .prepare("SELECT uid, path, revision FROM source_documents ORDER BY revision, path")?;
        let rows = statement.query_map([], |row| {
            let uid: i64 = row.get(0)?;
            let path: String = row.get(1)?;
            let found: String = row.get(2)?;

            return Ok((uid, path, found));
        })?;
        let mut candidates = Vec::new();

        for row in rows
        {
            let (uid, path, found) = row?;
            if revision.is_none_or(|wanted| return found == wanted)
            {
                candidates.push((uid, path));
            }
        }

        return Ok(candidates);
    }

    /// Every table line in a document, in the order it was authored.
    ///
    /// `block` and `table` narrow it. A table is addressed by both, because `table_ordinal`
    /// is counted within its block and "the second table" means nothing on its own.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Table_Lines(
        &self,
        document_uid: i64,
        block: Option<u32>,
        table: Option<u32>,
    ) -> Result<Vec<TableLine>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT block.ordinal, line.table_ordinal, line.ordinal, line.kind,
                    line.cells_json, line.text, line.content_hash
             FROM source_table_rows line
             JOIN source_blocks block ON block.uid = line.source_block_uid
             WHERE block.document_uid = ?1
               AND (?2 IS NULL OR block.ordinal = ?2)
               AND (?3 IS NULL OR line.table_ordinal = ?3)
             ORDER BY block.ordinal, line.table_ordinal, line.ordinal",
        )?;
        let rows = statement.query_map(params![document_uid, block, table], |row| {
            let cells: String = row.get(4)?;

            return Ok(TableLine {
                block_ordinal: row.get(0)?,
                table_ordinal: row.get(1)?,
                row_ordinal: row.get(2)?,
                kind: row.get(3)?,
                cells: serde_json::from_str(&cells).unwrap_or_default(),
                text: row.get(5)?,
                content_hash: row.get(6)?,
            });
        })?;

        return Collected(rows);
    }
}

fn Matches(path: &str, needle: &str, tier: PathMatch) -> bool
{
    return match tier
    {
        PathMatch::Exact => path == needle,
        PathMatch::FileName => path.rsplit('/').next() == Some(needle),
        PathMatch::Fragment => path.contains(needle),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::governing::Seed_Governing_Records;

    fn Seeded() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Seed_Governing_Records(&mut store).expect("seeds");
        return store;
    }

    #[test]
    fn Test_A_Record_Should_Come_Back_As_The_Bytes_It_Went_In_As()
    {
        let store = Seeded();

        let documents = store.Documents_Behind("D-132", None).expect("queries");

        let first = documents.first().expect("D-132 is seeded from a document");
        assert_eq!(documents.len(), 1);
        assert!(first.path.ends_with("D-132-the-plan-is-a-game-plan.md"));
        assert_eq!(first.revision, crate::store::AUTHORED);
        assert!(
            first.text.starts_with("---\nid: D-132\n"),
            "the front matter did not survive: {:?}",
            first.text.get(..40)
        );
    }

    /// The catalog mints nodes with no document behind them, and so does a placeholder.
    /// An empty vector here means "no content recorded", which is not the same as an
    /// unknown identifier and must not print like one.
    #[test]
    fn Test_A_Node_With_No_Source_Should_Be_Told_From_An_Unknown_One()
    {
        let store = Seeded();

        assert!(store.Node_Summary("ADR-DOC-001").expect("queries").is_some());
        assert!(store.Documents_Behind("ADR-DOC-001", None).expect("queries").is_empty());
        assert!(store.Node_Summary("D-9999").expect("queries").is_none());
    }

    #[test]
    fn Test_A_Revision_Filter_Should_Narrow_A_Record()
    {
        let store = Seeded();

        assert_eq!(
            store
                .Documents_Behind("D-132", Some(crate::store::AUTHORED))
                .expect("queries")
                .len(),
            1
        );
        assert!(
            store
                .Documents_Behind("D-132", Some("v14.36"))
                .expect("queries")
                .is_empty()
        );
    }

    /// A whole path beats a fragment. Otherwise the most precise way to ask is the one
    /// that drags in every neighbouring document.
    #[test]
    fn Test_A_Whole_Path_Should_Beat_A_Fragment()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        store.Put_Source_Document("a/model.md", "v1", "# one\n").expect("writes");
        store.Put_Source_Document("b/model.md.old", "v1", "# two\n").expect("writes");

        let (exact, tier) = store.Documents_Named("a/model.md", None).expect("queries");
        assert_eq!(exact.len(), 1);
        assert_eq!(tier, PathMatch::Exact);

        let (named, tier) = store.Documents_Named("model.md", None).expect("queries");
        assert_eq!(named.len(), 1, "the file name should not have matched the .old file");
        assert_eq!(tier, PathMatch::FileName);

        let (fragment, tier) = store.Documents_Named("model", None).expect("queries");
        assert_eq!(fragment.len(), 2);
        assert_eq!(tier, PathMatch::Fragment);
    }

    #[test]
    fn Test_A_Name_Nothing_Holds_Should_Match_Nothing()
    {
        let store = Seeded();

        let (found, _) = store.Documents_Named("no-such-document.md", None).expect("queries");

        assert!(found.is_empty());
    }

    #[test]
    fn Test_Table_Lines_Should_Come_Back_Typed_And_In_Order()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let markdown = "# Volume\n\n| Concept | Meaning |\n| --- | --- |\n| Ledger | a claim |\n| Gate | a stop |\n";
        let uid = store.Put_Source_Document("volume.md", "v14.36", markdown).expect("writes");
        store
            .Put_Source_Blocks(uid, &nomos_spec_model::Segment(markdown))
            .expect("writes blocks");

        let lines = store.Table_Lines(uid, None, None).expect("queries");

        assert_eq!(
            lines.iter().map(|line| return line.kind.as_str()).collect::<Vec<&str>>(),
            vec!["header", "separator", "content", "content"]
        );
        assert_eq!(
            lines.get(2).map(|line| return line.cells.clone()),
            Some(vec!["Ledger".to_owned(), "a claim".to_owned()])
        );
        assert!(
            lines.iter().all(|line| return line.content_hash.starts_with("sha256:")),
            "every line carries its own content address"
        );
    }

    #[test]
    fn Test_A_Document_With_No_Table_Should_Return_No_Lines()
    {
        let store = Seeded();

        let (found, _) = store
            .Documents_Named("D-130-no-xvpe-dependency-before-phase-5.md", None)
            .expect("queries");
        let uid = *found.first().expect("the record is seeded");

        assert!(store.Table_Lines(uid, None, None).expect("queries").is_empty());
    }
}
