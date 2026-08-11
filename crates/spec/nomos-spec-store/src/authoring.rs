//! The authoring round trip `D-129` decided: read a record out as markdown, write an edit
//! back as a transaction against the store.
//!
//! # Four steps, as types
//!
//! `D-129` says an author's edit is *claim, stage, preview, commit*, and that the preview
//! is mandatory. Four functions in a row would leave that a convention: a caller could stage
//! and commit, and the only thing standing between them and an unpreviewed write would be
//! remembering. So each step is a type that only the previous step can produce, and
//! [`SpecificationStore::Commit_Edit`] takes an [`EditPreview`] — a value nothing outside
//! this module can construct. The preview is not checked for; it is required to exist.
//!
//! That also settles staleness without a check. The preview owns the staged bytes rather
//! than pointing at them, so there is no window in which the thing committed is not the
//! thing previewed.
//!
//! # What the projection is made of
//!
//! [`SpecificationStore::Record_Markdown`] renders from the store's own rows — the declared
//! front matter, the blocks — and never from the ingested blob. Reading the blob back would
//! be an echo, and would prove nothing about whether the store holds enough to be the
//! substrate. The projection carries both content addresses so a caller can see that they
//! agree; `Test_Every_Governing_Record_Should_Project_To_Its_Own_Bytes` asserts they do,
//! for all of them.

use crate::read::DocumentSource;
use crate::record::{Disposition, Kind_Label, Kind_Of};
use crate::store::{
    Collected, EXTERNAL, Inverse_Of, NodeRow, SpecificationStore, StoreError, Write_Blob,
    Write_Node, Write_Relation, Write_Source_Blocks, Write_Source_Document,
};
use nomos_spec_model::{
    BlockKind, ContentHash, Normalize, Parse_Record, Record, RecordFrontMatter, RecordRelation,
    RenderError, Render_Record, Round_Trips, Segment, SourceBlock,
};
use rusqlite::{Connection, OptionalExtension, params};

/// What went into the store for one record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordWrite
{
    pub node_uid: i64,
    pub document_uid: i64,
    pub blocks: u32,
    pub headings: u32,
    pub relations: u32,
}

/// A record read back out as markdown, with both content addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordProjection
{
    pub node_id: String,
    pub path: String,
    pub revision: String,
    /// The markdown, rendered from the store's rows.
    pub markdown: String,
    /// What the ingested bytes hash to.
    pub source_hash: String,
    /// What this projection hashes to.
    pub projected_hash: String,
}

impl RecordProjection
{
    /// Whether the projection is the ingested bytes.
    ///
    /// Reported rather than asserted here. A document the store cannot reproduce is a real
    /// answer — a v14 record carrying a byte order mark is one — and a reader who asked for
    /// markdown should get it along with the fact, not an error instead of it.
    #[must_use]
    pub fn Matches_Source(&self) -> bool
    {
        return self.source_hash == self.projected_hash;
    }
}

/// Why an authoring step refused.
#[derive(Debug)]
pub enum EditError
{
    /// No node in the store carries this identifier.
    NoSuchRecord
    {
        node_id: String,
    },
    /// The node is known and no source document is recorded against it.
    NoContent
    {
        node_id: String,
    },
    /// More than one revision answers, so editing one of them would be a guess.
    Ambiguous
    {
        node_id: String,
        revisions: Vec<String>,
    },
    /// The document was ingested without a declared front matter row, so the store cannot
    /// say what its front matter said and cannot render it back.
    NotAuthored
    {
        path: String,
    },
    /// The staged text is not a readable record.
    Unreadable
    {
        cause: String,
    },
    /// The staged front matter names a different record.
    ///
    /// Refused rather than treated as a rename: `D-129` puts identity in the store, and a
    /// surface that let an author retype `id:` would make identity a property of the file
    /// again — the exact inversion the record is about.
    IdentityChanged
    {
        held: String,
        staged: String,
    },
    /// The staged text is not what the canonical layout would produce for it, so committing
    /// it would change bytes nobody asked to change.
    NotCanonical
    {
        cause: String,
    },
    /// The rename's destination already holds a document at this revision.
    PathTaken
    {
        path: String,
    },
    Store(StoreError),
}

impl core::fmt::Display for EditError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoSuchRecord { node_id } => write!(formatter, "no node identified {node_id} is in this store"),
            Self::NoContent { node_id } => write!(
                formatter,
                "{node_id} is in this store as an identity with no source document behind it, \
                 so there is nothing to read out or edit"
            ),
            Self::Ambiguous { node_id, revisions } => write!(
                formatter,
                "{node_id} is held at {} revisions ({}); name one with a revision, because \
                 editing whichever came back first is a guess",
                revisions.len(), revisions.join(", ")
            ),
            Self::NotAuthored { path } => write!(
                formatter,
                "{path} has no declared front matter in this store, so its markdown cannot be \
                 rendered from the store's own rows. Documents ingested from a corpus arrive \
                 as blocks and lineage; only a record written through this door declares one"
            ),
            Self::Unreadable { cause } => write!(formatter, "the staged text is not a record: {cause}"),
            Self::IdentityChanged { held, staged } => write!(
                formatter,
                "the staged front matter is identified {staged} and the record being edited is \
                 {held}. Identity lives in the store, so it is not editable through the file"
            ),
            Self::NotCanonical { cause } => write!(
                formatter,
                "the staged text is not what this surface would write for it, so committing it \
                 would change bytes the edit did not ask to change: {cause}"
            ),
            Self::PathTaken { path } => write!(
                formatter,
                "{path} already names a document at this revision, so the rename would merge \
                 two records into one address"
            ),
            Self::Store(error) => write!(formatter, "{error}"),
        };
    }
}

impl std::error::Error for EditError
{}

impl From<StoreError> for EditError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl From<rusqlite::Error> for EditError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Store(StoreError::from(error));
    }
}

impl From<RenderError> for EditError
{
    fn from(error: RenderError) -> Self
    {
        return Self::NotCanonical {
            cause: error.to_string(),
        };
    }
}

/// What became of one block.
///
/// [`BlockChange::Reflowed`] is separate from [`BlockChange::Reworded`] on the normalizer's
/// authority: the normalized hashes are equal, so v14's own definition of same-content says
/// the wording did not change. Collapsing the two would make every reflowed paragraph
/// report as moved wording, and a preview that cries wolf is a preview people stop reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockChange
{
    Added
    {
        ordinal: u32,
        kind: String,
    },
    Removed
    {
        ordinal: u32,
        kind: String,
    },
    /// Same position, different wording.
    Reworded
    {
        ordinal: u32,
        before: String,
        after: String,
    },
    /// Same wording, different position.
    Moved
    {
        from: u32,
        to: u32,
    },
    /// Same wording, different whitespace.
    Reflowed
    {
        ordinal: u32,
    },
}

impl BlockChange
{
    /// Whether this change moves, rewrites or removes wording that was already there.
    #[must_use]
    pub const fn Disturbs_Wording(&self) -> bool
    {
        return matches!(
            self,
            Self::Removed { .. } | Self::Reworded { .. } | Self::Moved { .. }
        );
    }

    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Added { ordinal, kind } => format!("block {ordinal} added ({kind})"),
            Self::Removed { ordinal, kind } => format!("block {ordinal} removed ({kind})"),
            Self::Reworded {
                ordinal,
                before,
                after,
            } => format!("block {ordinal} reworded, {before} -> {after}"),
            Self::Moved { from, to } => format!("block {from} moved to {to}, wording unchanged"),
            Self::Reflowed { ordinal } =>
            {
                format!("block {ordinal} reflowed, wording unchanged under the normalizer")
            }
        };
    }
}

/// A front matter field the edit changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityChange
{
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Which relations the edit adds and which it removes.
///
/// Both halves are `Vec<RecordRelation>`, so returning them as a pair let a caller take them
/// in either order and still compile. Named, the mistake cannot be written.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RelationChanges
{
    added: Vec<RecordRelation>,
    removed: Vec<RecordRelation>,
}

/// What became of a normative statement recorded against this record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormativeMovement
{
    pub statement_id: String,
    pub canonical_hash: String,
    pub outcome: NormativeOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormativeOutcome
{
    /// Still in the block it was in.
    Held
    {
        block: u32,
    },
    /// Still present, in a different block.
    Moved
    {
        from: u32,
        to: u32,
    },
    /// Was in the record and is not in the staged text.
    Gone
    {
        from: u32,
    },
    /// Recorded against the record and not found in it before the edit either, so this edit
    /// cannot be what moved it. Reported rather than skipped: a statement the store cannot
    /// locate is a finding about the store.
    Unlocatable,
}

/// Step one: the record, read out and held for editing.
///
/// Carries the projection rather than pointing at the store, so the bytes an author was
/// shown are the bytes the preview compares against.
#[derive(Debug)]
pub struct ClaimedRecord
{
    projection: RecordProjection,
    front_matter: RecordFrontMatter,
    document_uid: i64,
}

impl ClaimedRecord
{
    /// The markdown to edit.
    #[must_use]
    pub fn Markdown(&self) -> &str
    {
        return &self.projection.markdown;
    }

    #[must_use]
    pub fn Node_Id(&self) -> &str
    {
        return &self.projection.node_id;
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.projection.path;
    }

    /// Step two: the edited markdown, and where it should live.
    ///
    /// A rename is `Some(path)` and nothing else. `D-129`'s first paragraph is what makes
    /// that cheap: a path is navigation, so moving one changes no identity and leaves the
    /// node, its blocks and their surrogates alone.
    ///
    /// # Errors
    ///
    /// Returns [`EditError::Unreadable`] if the text is not a record,
    /// [`EditError::IdentityChanged`] if it names a different one, and
    /// [`EditError::NotCanonical`] if this surface would not have written those bytes.
    pub fn Stage(self, markdown: &str, rename: Option<&str>) -> Result<StagedEdit, EditError>
    {
        let record = self.Accepted(markdown)?;
        let path = rename.unwrap_or(&self.projection.path).to_owned();

        return Ok(StagedEdit {
            claimed: self,
            path,
            markdown: markdown.to_owned(),
            record,
        });
    }

    /// The staged text as a record, or why this surface will not take those bytes.
    fn Accepted(&self, markdown: &str) -> Result<Record, EditError>
    {
        let record = Parse_Record(markdown)
            .map_err(|error| return EditError::Unreadable { cause: error.to_string() })?;

        if record.front_matter.id != self.projection.node_id
        {
            return Err(EditError::IdentityChanged {
                held: self.projection.node_id.clone(),
                staged: record.front_matter.id,
            });
        }
        if !Round_Trips(markdown)
        {
            return Err(EditError::NotCanonical {
                cause: Why_Not_Canonical(markdown, &record),
            });
        }

        return Ok(record);
    }
}

/// Step three: the edit, read and accepted, not yet inspected.
#[derive(Debug)]
pub struct StagedEdit
{
    claimed: ClaimedRecord,
    path: String,
    markdown: String,
    record: Record,
}

impl StagedEdit
{
    /// Step four: what committing this would change.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] if the store cannot be read.
    pub fn Preview(self, store: &SpecificationStore) -> Result<EditPreview, EditError>
    {
        let before = store.Stored_Blocks(self.claimed.document_uid)?;
        let after = Segment(&self.record.body);
        let blocks = Block_Changes(&before, &after);
        let identity = Identity_Changes(&self.claimed.front_matter, &self.record.front_matter);
        let relations = Relation_Changes(
            &self.claimed.front_matter.relations,
            &self.record.front_matter.relations,
        );
        let statements =
            store.Statement_Movements(&self.claimed.projection.node_id, &before, &after)?;

        return Ok(EditPreview {
            staged: self,
            blocks,
            identity,
            relations_added: relations.added,
            relations_removed: relations.removed,
            statements,
        });
    }
}

/// What committing an edit would change.
///
/// Only [`StagedEdit::Preview`] can build one, and [`SpecificationStore::Commit_Edit`] takes
/// one. That is the whole mechanism by which the mandatory preview is mandatory.
#[derive(Debug)]
pub struct EditPreview
{
    staged: StagedEdit,
    blocks: Vec<BlockChange>,
    identity: Vec<IdentityChange>,
    relations_added: Vec<RecordRelation>,
    relations_removed: Vec<RecordRelation>,
    statements: Vec<NormativeMovement>,
}

impl EditPreview
{
    #[must_use]
    pub fn Node_Id(&self) -> &str
    {
        return &self.staged.claimed.projection.node_id;
    }

    /// The paths before and after, when they differ.
    #[must_use]
    pub fn Rename(&self) -> Option<(&str, &str)>
    {
        let before = self.staged.claimed.projection.path.as_str();
        let after = self.staged.path.as_str();

        return (before != after).then_some((before, after));
    }

    #[must_use]
    pub fn Blocks(&self) -> &[BlockChange]
    {
        return &self.blocks;
    }

    #[must_use]
    pub fn Identity(&self) -> &[IdentityChange]
    {
        return &self.identity;
    }

    #[must_use]
    pub fn Relations_Added(&self) -> &[RecordRelation]
    {
        return &self.relations_added;
    }

    #[must_use]
    pub fn Relations_Removed(&self) -> &[RecordRelation]
    {
        return &self.relations_removed;
    }

    #[must_use]
    pub fn Statements(&self) -> &[NormativeMovement]
    {
        return &self.statements;
    }

    /// The staged bytes, so a caller can write them where the author expects them.
    #[must_use]
    pub fn Markdown(&self) -> &str
    {
        return &self.staged.markdown;
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.staged.path;
    }

    /// The question `D-129` calls mandatory: does this edit move wording that was there?
    ///
    /// Answered from two kinds of evidence, because they are not the same claim. A normative
    /// statement recorded against this record is the strong answer; the blocks are the answer
    /// available for every record, including the ones whose statements no corpus has been
    /// ingested for. A preview that reported only the strong answer would print nothing at
    /// all for this repository's own records, and nothing reads as *no*.
    #[must_use]
    pub fn Wording_Moved(&self) -> bool
    {
        return self.blocks.iter().any(BlockChange::Disturbs_Wording)
            || self.statements.iter().any(|movement| {
                return matches!(
                    movement.outcome,
                    NormativeOutcome::Moved { .. } | NormativeOutcome::Gone { .. }
                );
            });
    }

    /// Whether this edit changes anything at all.
    #[must_use]
    pub fn Changes_Nothing(&self) -> bool
    {
        return self.blocks.is_empty()
            && self.identity.is_empty()
            && self.relations_added.is_empty()
            && self.relations_removed.is_empty()
            && self.Rename().is_none();
    }

    /// The preview an author reads.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        let mut lines = vec![format!("{} at {}", self.Node_Id(), self.Path())];

        self.Describe_Changes(&mut lines);
        if self.Changes_Nothing()
        {
            lines.push("  nothing changes".to_owned());
        }
        lines.push(self.Describe_Normative());

        return lines.join("\n");
    }

    /// One line for each thing the edit changes, in the order an author reads them.
    fn Describe_Changes(&self, lines: &mut Vec<String>)
    {
        if let Some((before, after)) = self.Rename()
        {
            lines.push(format!("  renamed: {before} -> {after}"));
        }
        for change in &self.identity
        {
            lines.push(format!("  {}: {:?} -> {:?}", change.field, change.before, change.after));
        }
        for change in &self.blocks
        {
            lines.push(format!("  {}", change.Describe()));
        }
        for relation in &self.relations_added
        {
            lines.push(format!("  relation added: {} {}", relation.relation, relation.target));
        }
        for relation in &self.relations_removed
        {
            lines.push(format!("  relation removed: {} {}", relation.relation, relation.target));
        }
    }

    /// The mandatory sentence, and what it was derived from.
    fn Describe_Normative(&self) -> String
    {
        let verdict = self.Verdict();

        if self.statements.is_empty()
        {
            return format!(
                "  {verdict} - derived from the blocks, because no normative statement is \
                 recorded against {} in this store",
                self.Node_Id()
            );
        }

        return format!("  {verdict} - {}", self.Statement_Detail());
    }

    /// The sentence `D-129` calls mandatory, in the two words it can take.
    fn Verdict(&self) -> &'static str
    {
        if self.Wording_Moved()
        {
            return "normative wording moved";
        }

        return "no normative wording moved";
    }

    /// Each statement recorded against this record, and what became of it.
    fn Statement_Detail(&self) -> String
    {
        return self
            .statements
            .iter()
            .map(|movement| {
                return format!("{} {}", movement.statement_id, movement.outcome.Describe());
            })
            .collect::<Vec<String>>()
            .join(", ");
    }
}

impl NormativeOutcome
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Held { block } => format!("held in block {block}"),
            Self::Moved { from, to } => format!("moved from block {from} to {to}"),
            Self::Gone { from } => format!("gone from block {from}"),
            Self::Unlocatable => "not locatable in this record before the edit".to_owned(),
        };
    }
}

/// What a commit did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitReport
{
    pub node_id: String,
    pub path: String,
    pub blocks: usize,
    pub blocks_removed: usize,
    pub relations_added: usize,
    pub relations_removed: usize,
    pub renamed: bool,
}

impl SpecificationStore
{
    /// Writes an authored record: identity, bytes, blocks, dispositions and the front matter
    /// it declared.
    ///
    /// The one door. The seed uses it, a re-ingest uses it, and a commit uses the same
    /// writers underneath, so there is no second place that decides what a record is.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Record`] if the markdown is not a readable record, and
    /// [`StoreError`] on any SQL failure.
    pub fn Put_Record(
        &mut self,
        path: &str,
        revision: &str,
        markdown: &str,
    ) -> Result<RecordWrite, StoreError>
    {
        let record = Parse_Record(markdown).map_err(|error| {
            return StoreError::Record {
                path: path.to_owned(),
                cause: error.to_string(),
            };
        })?;

        return self.In_Transaction(|transaction| {
            return Write_Record(
                transaction,
                Authored {
                    path,
                    revision,
                    markdown,
                },
                &record,
            );
        });
    }

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

    /// Step one of the round trip: read the record out and hold it for editing.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] naming why no single record answered.
    pub fn Claim_For_Edit(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<ClaimedRecord, EditError>
    {
        let document = self.Sole_Document(node_id, revision)?;
        let projection = self.Projection_Of(&document)?;
        let front_matter = self
            .Declared_Front_Matter(document.uid)?
            .ok_or_else(|| {
                return EditError::NotAuthored {
                    path: document.path.clone(),
                };
            })?;

        return Ok(ClaimedRecord {
            projection,
            front_matter,
            document_uid: document.uid,
        });
    }

    /// Writes a previewed edit, atomically.
    ///
    /// Takes the preview rather than the staged text, so an unpreviewed commit is not a
    /// discipline but an unwritable program.
    ///
    /// # Errors
    ///
    /// Returns [`EditError::PathTaken`] if a rename would collide, and [`EditError::Store`]
    /// on any SQL failure. Nothing is written if anything fails.
    pub fn Commit_Edit(&mut self, preview: &EditPreview) -> Result<CommitReport, EditError>
    {
        return self.In_Transaction(|transaction| return Apply(transaction, preview));
    }

    /// The blocks of one document, in the order they were authored.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Stored_Blocks(&self, document_uid: i64) -> Result<Vec<SourceBlock>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT ordinal, kind, heading_path, text FROM source_blocks
             WHERE document_uid = ?1 ORDER BY ordinal",
        )?;
        let rows = statement.query_map(params![document_uid], |row| {
            let path: String = row.get(2)?;
            let kind: String = row.get(1)?;

            return Ok(SourceBlock {
                ordinal: row.get(0)?,
                kind: Kind_Of(&kind).unwrap_or(BlockKind::Prose),
                heading_path: Heading_Path(&path),
                text: row.get(3)?,
            });
        })?;

        return Collected(rows);
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

        return Collected(rows);
    }

    /// The one document behind an identifier, or why there is not exactly one.
    fn Sole_Document(
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
    fn Projection_Of(&self, document: &DocumentSource) -> Result<RecordProjection, EditError>
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
            projected_hash: ContentHash::Of(&markdown).As_Str().to_owned(),
            source_hash: document.content_hash.clone(),
            markdown,
        });
    }

    /// What became of each normative statement recorded against this record.
    fn Statement_Movements(
        &self,
        node_id: &str,
        before: &[SourceBlock],
        after: &[SourceBlock],
    ) -> Result<Vec<NormativeMovement>, StoreError>
    {
        let mut movements = Vec::new();
        for (statement_id, canonical_text, canonical_hash) in self.Recorded_Statements(node_id)?
        {
            movements.push(NormativeMovement {
                statement_id,
                canonical_hash,
                outcome: Located(&canonical_text, before, after),
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
            return Ok((row.get(0)?, row.get(1)?, row.get(2)?));
        })?;

        return Collected(rows);
    }
}

/// A normative statement as the store keeps it: identifier, canonical text, canonical hash.
type Recorded = (String, String, String);

/// The authored document a record arrived as: where it lives, at which revision, and the
/// bytes themselves.
#[derive(Clone, Copy)]
pub(crate) struct Authored<'a>
{
    pub(crate) path: &'a str,
    pub(crate) revision: &'a str,
    pub(crate) markdown: &'a str,
}

/// Writes one record through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Record(
    connection: &Connection,
    authored: Authored<'_>,
    record: &Record,
) -> Result<RecordWrite, StoreError>
{
    let Authored { path, revision, markdown } = authored;
    let front_matter = &record.front_matter;
    let node_uid = Write_Node(connection, NodeRow {
        node_id: &front_matter.id,
        kind: &front_matter.kind,
        authority: &front_matter.authority,
        representation: "document",
        title: &front_matter.title,
    })?;
    let document_uid = Write_Source_Document(connection, path, revision, markdown)?;
    let blocks = Segment(&record.body);
    Write_Source_Blocks(connection, document_uid, &blocks)?;
    let headings = Write_Headings(connection, document_uid, node_uid, &blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, front_matter)?;
    let relations = Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;

    return Ok(RecordWrite {
        node_uid,
        document_uid,
        blocks: u32::try_from(blocks.len()).unwrap_or(u32::MAX),
        headings,
        relations,
    });
}

/// Writes each heading and points it at the record it belongs to.
///
/// The disposition is written against the heading's own identity — its title and depth —
/// rather than against the ordinal it happened to have. An edit that moves a section changes
/// every following ordinal, and a lineage row that only matches the old number is a lineage
/// row an edit silently drops.
fn Write_Headings(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<u32, StoreError>
{
    let mut insert_heading = connection.prepare(
        "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut dispose_heading = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition, target_node_uid)
         SELECT uid, ?4, ?5 FROM source_headings
         WHERE document_uid = ?1 AND title = ?2 AND depth = ?3",
    )?;
    let headings: Vec<Heading<'_>> = blocks
        .iter()
        .filter(|block| return block.kind == BlockKind::Heading)
        .map(Heading_Of)
        .collect();
    for heading in &headings
    {
        insert_heading
            .execute(params![document_uid, heading.ordinal, heading.depth, heading.title])?;
        dispose_heading.execute(params![
            document_uid,
            heading.title,
            heading.depth,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(u32::try_from(headings.len()).unwrap_or(u32::MAX));
}

/// A heading as the schema files it: where it sat, and the identity the lineage row matches
/// on.
struct Heading<'a>
{
    ordinal: u32,
    depth: i64,
    title: &'a str,
}

/// Reads a heading's depth from its markers and its title from what follows them.
fn Heading_Of(block: &SourceBlock) -> Heading<'_>
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();

    return Heading {
        ordinal: block.ordinal,
        depth: i64::try_from(depth).unwrap_or(0),
        title: block.text.trim_start_matches('#').trim(),
    };
}

/// Points every block at the record it belongs to.
fn Dispose_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut dispose = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, ?3, ?4 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
    )?;

    for block in blocks
    {
        dispose.execute(params![
            document_uid,
            block.ordinal,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(());
}

/// Writes the three front matter fields nothing else keeps.
fn Write_Front_Matter(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    front_matter: &RecordFrontMatter,
) -> Result<(), StoreError>
{
    let tags = serde_json::to_string(&front_matter.tags)
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    connection.execute(
        "INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(document_uid) DO UPDATE SET
             node_uid = excluded.node_uid,
             status = excluded.status,
             version = excluded.version,
             tags_json = excluded.tags_json",
        params![
            document_uid,
            node_uid,
            front_matter.status,
            front_matter.version,
            tags
        ],
    )?;

    return Ok(());
}

/// Replaces the declared relation list, in order.
///
/// Replaced rather than merged: the list is a document's own sentence about itself, and
/// merging would make a removed relation unremovable through the surface that wrote it.
fn Write_Declared_Relations(
    connection: &Connection,
    document_uid: i64,
    relations: &[RecordRelation],
) -> Result<u32, StoreError>
{
    connection.execute(
        "DELETE FROM record_relations WHERE document_uid = ?1",
        params![document_uid],
    )?;

    let mut insert = connection.prepare(
        "INSERT INTO record_relations (document_uid, ordinal, target, relation)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    let mut ordinal = 0_u32;
    for relation in relations
    {
        ordinal = ordinal.saturating_add(1);
        insert.execute(params![document_uid, ordinal, relation.target, relation.relation])?;
    }

    return Ok(ordinal);
}

/// Writes a previewed edit through a caller's transaction.
fn Apply(connection: &Connection, preview: &EditPreview) -> Result<CommitReport, EditError>
{
    let claimed = &preview.staged.claimed;
    let document_uid = claimed.document_uid;
    let front_matter = &preview.staged.record.front_matter;

    if let Some((_, after)) = preview.Rename()
    {
        Rename_Document(connection, document_uid, after, &claimed.projection.revision)?;
    }
    Rewrite_Bytes(connection, document_uid, &preview.staged.markdown)?;
    let node_uid = Restate_Node(connection, front_matter)?;
    let blocks = Segment(&preview.staged.record.body);
    let blocks_removed = Rewrite_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, front_matter)?;
    Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;
    Update_Graph(connection, &front_matter.id, preview)?;

    return Ok(Reported(preview, blocks.len(), blocks_removed));
}

/// Replaces the document's stored bytes with the ones the author staged.
fn Rewrite_Bytes(connection: &Connection, document_uid: i64, markdown: &str)
    -> Result<(), StoreError>
{
    let blob_uid = Write_Blob(connection, markdown.as_bytes())?;

    connection.execute(
        "UPDATE source_documents SET blob_uid = ?2 WHERE uid = ?1",
        params![document_uid, blob_uid],
    )?;

    return Ok(());
}

/// Moves the title and the kind with the author's own edit.
///
/// `Write_Node` refuses to overwrite a real node deliberately — the first writer of a record
/// is its author and a later ingest pass must not restate it — and this is that author,
/// arriving through the door the record decided they would use.
fn Restate_Node(connection: &Connection, front_matter: &RecordFrontMatter)
    -> Result<i64, StoreError>
{
    let node_uid = Node_Uid_Of(connection, &front_matter.id)?;

    connection.execute(
        "UPDATE nodes SET kind = ?2, authority = ?3, title = ?4 WHERE uid = ?1",
        params![
            node_uid,
            front_matter.kind,
            front_matter.authority,
            front_matter.title
        ],
    )?;

    return Ok(node_uid);
}

/// Replaces the document's blocks with the staged ones, and reports how many the edit
/// shortened it past.
fn Rewrite_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, EditError>
{
    Write_Source_Blocks(connection, document_uid, blocks)?;
    Write_Headings(connection, document_uid, node_uid, blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, blocks)?;

    return Prune_Blocks_Beyond(connection, document_uid, blocks.len());
}

/// What the commit did, as the caller reads it back.
fn Reported(preview: &EditPreview, blocks: usize, blocks_removed: usize) -> CommitReport
{
    return CommitReport {
        node_id: preview.staged.record.front_matter.id.clone(),
        path: preview.staged.path.clone(),
        blocks,
        blocks_removed,
        relations_added: preview.relations_added.len(),
        relations_removed: preview.relations_removed.len(),
        renamed: preview.Rename().is_some(),
    };
}

/// Moves a document to a new path, keeping its surrogate.
///
/// `UPDATE` rather than insert-and-delete, because `uid` is what every block, lineage and
/// omission row hangs from. A rename that minted a new document row would be a rename that
/// silently dropped the record's history — which is what makes a rename a migration
/// elsewhere and an ordinary edit here.
fn Rename_Document(
    connection: &Connection,
    document_uid: i64,
    path: &str,
    revision: &str,
) -> Result<(), EditError>
{
    let holder = Document_At(connection, path, revision)?;

    if holder.is_some_and(|found| return found != document_uid)
    {
        return Err(EditError::PathTaken {
            path: path.to_owned(),
        });
    }

    connection
        .execute(
            "UPDATE source_documents SET path = ?2 WHERE uid = ?1",
            params![document_uid, path],
        )
        .map_err(StoreError::from)?;

    return Ok(());
}

/// Which document, if any, already lives at this address.
fn Document_At(connection: &Connection, path: &str, revision: &str)
    -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            params![path, revision],
            |row| row.get(0),
        )
        .optional()?);
}

/// Removes the blocks an edit shortened the document past.
///
/// Refuses rather than deletes where a block carries a justified omission. An omission row
/// is the only sanctioned way for content to leave this store, carrying its reason and the
/// decision that allowed it; deleting the block underneath it would destroy the justification
/// and leave the content gone — the exact pair of events this schema exists to prevent.
fn Prune_Blocks_Beyond(
    connection: &Connection,
    document_uid: i64,
    keep: usize,
) -> Result<usize, EditError>
{
    let keep = i64::try_from(keep).unwrap_or(i64::MAX);
    let justified = Justified_Omissions_Beyond(connection, document_uid, keep)?;

    if justified > 0
    {
        return Err(EditError::Store(StoreError::Sql(format!(
            "{justified} block(s) this edit removes carry a justified omission; deleting them \
             would destroy the justification and the content together"
        ))));
    }

    Detach_Blocks_Beyond(connection, document_uid, keep)?;

    let removed = connection
        .execute(
            "DELETE FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2",
            params![document_uid, keep],
        )
        .map_err(StoreError::from)?;

    return Ok(removed);
}

/// How many blocks past `keep` carry a justified omission.
fn Justified_Omissions_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<u32, StoreError>
{
    return Ok(connection.query_row(
        "SELECT count(*) FROM omissions o
         JOIN source_blocks b ON b.uid = o.source_block_uid
         WHERE b.document_uid = ?1 AND b.ordinal > ?2",
        params![document_uid, keep],
        |row| row.get(0),
    )?);
}

/// Removes everything hanging from the blocks past `keep`, so the blocks themselves can go.
///
/// Written out rather than iterated over a list. These are three different statements run
/// once each, in an order the foreign keys require — a row's lineage before the row, and both
/// before the block they hang from — and a loop said "repeat this" about three steps that are
/// not repetitions of one another.
fn Detach_Blocks_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM lineage WHERE source_table_row_uid IN (
             SELECT r.uid FROM source_table_rows r
             JOIN source_blocks b ON b.uid = r.source_block_uid
             WHERE b.document_uid = ?1 AND b.ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM lineage WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM source_table_rows WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;

    return Ok(());
}

/// Brings the graph into line with what the record now declares.
fn Update_Graph(
    connection: &Connection,
    node_id: &str,
    preview: &EditPreview,
) -> Result<(), StoreError>
{
    for relation in &preview.relations_added
    {
        Add_Relation(connection, node_id, relation)?;
    }
    for relation in &preview.relations_removed
    {
        Remove_Relation(connection, node_id, relation)?;
    }

    return Ok(());
}

/// Declares a relation, minting the target as an external node when nothing holds it yet.
fn Add_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    if Optional_Node_Uid(connection, &relation.target)?.is_none()
    {
        Write_Node(connection, NodeRow {
            node_id: &relation.target,
            kind: "unknown",
            authority: EXTERNAL,
            representation: "record",
            title: &relation.target,
        })?;
    }

    return Write_Relation(connection, node_id, &relation.relation, &relation.target);
}

/// Withdraws a relation, and the inverse the schema keeps beside it.
fn Remove_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    Delete_Relation(connection, node_id, &relation.relation, &relation.target)?;

    if let Some(inverse) = Inverse_Of(connection, &relation.relation)?
    {
        Delete_Relation(connection, &relation.target, &inverse, node_id)?;
    }

    return Ok(());
}

fn Delete_Relation(
    connection: &Connection,
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM relations
         WHERE from_node_uid = (SELECT uid FROM nodes WHERE node_id = ?1)
           AND relation_type = ?2
           AND to_node_uid = (SELECT uid FROM nodes WHERE node_id = ?3)",
        params![from_node_id, relation_type, to_node_id],
    )?;

    return Ok(());
}

fn Node_Uid_Of(connection: &Connection, node_id: &str) -> Result<i64, StoreError>
{
    return Optional_Node_Uid(connection, node_id)?.ok_or_else(|| {
        return StoreError::Sql(format!(
            "{node_id} was read out of this store and is no longer in it"
        ));
    });
}

fn Optional_Node_Uid(connection: &Connection, node_id: &str) -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| row.get(0),
        )
        .optional()?);
}

/// What became of each block, before and after.
fn Block_Changes(before: &[SourceBlock], after: &[SourceBlock]) -> Vec<BlockChange>
{
    let mut taken = vec![false; before.len()];
    let mut changes = Vec::new();
    let mut added: Vec<&SourceBlock> = Vec::new();

    for block in after
    {
        match Claimed(before, &mut taken, block)
        {
            Match::Held(change) => changes.extend(change),
            Match::New => added.push(block),
        }
    }

    let removed = Untaken(before, &taken);
    let paired = Paired(&added, &removed);

    changes.extend(paired);
    changes.sort_by_key(Position_Of);

    return changes;
}

/// What a staged block turned out to be against the blocks that were already there.
enum Match
{
    /// One of them is this block, and this is what became of it — nothing worth reporting,
    /// where it neither moved nor was reflowed.
    Held(Option<BlockChange>),
    /// Nothing before this edit carries this wording.
    New,
}

/// Claims the block this staged one is — verbatim first, then under the normalizer — so that
/// no earlier block is matched twice, and says what became of it.
fn Claimed(before: &[SourceBlock], taken: &mut [bool], block: &SourceBlock) -> Match
{
    if let Some(index) = Unmatched(before, taken, |candidate| return candidate.text == block.text)
    {
        Take(taken, index);
        let found = before.get(index);
        let moved = Displaced(found, block.ordinal);

        return Match::Held(moved);
    }
    if let Some(index) = Unmatched(before, taken, |candidate| {
        return Normalize(&candidate.text) == Normalize(&block.text);
    })
    {
        Take(taken, index);

        return Match::Held(Some(BlockChange::Reflowed {
            ordinal: block.ordinal,
        }));
    }

    return Match::New;
}

/// The same wording at a different ordinal moved. At the same ordinal, nothing happened to
/// it, and reporting that would be noise in a preview an author has to read every time.
fn Displaced(found: Option<&SourceBlock>, to: u32) -> Option<BlockChange>
{
    return found
        .filter(|block| return block.ordinal != to)
        .map(|block| {
            return BlockChange::Moved {
                from: block.ordinal,
                to,
            };
        });
}

/// The blocks no staged block claimed, which are the ones the edit removed.
fn Untaken<'a>(before: &'a [SourceBlock], taken: &[bool]) -> Vec<&'a SourceBlock>
{
    return before
        .iter()
        .enumerate()
        .filter(|(index, _)| return Is_Free(taken, *index))
        .map(|(_, block)| return block)
        .collect();
}

/// An addition and a removal at one ordinal are a rewording, and saying so is more use to a
/// reader than two lines that do not mention each other.
fn Paired(added: &[&SourceBlock], removed: &[&SourceBlock]) -> Vec<BlockChange>
{
    let mut changes = Vec::new();

    for block in added
    {
        let change = Added_Or_Reworded(block, removed);
        changes.push(change);
    }
    for gone in removed
    {
        if !added.iter().any(|block| return block.ordinal == gone.ordinal)
        {
            changes.push(BlockChange::Removed {
                ordinal: gone.ordinal,
                kind: Kind_Label(gone.kind).to_owned(),
            });
        }
    }

    return changes;
}

/// An addition at an ordinal something was removed from is that block, reworded.
fn Added_Or_Reworded(block: &SourceBlock, removed: &[&SourceBlock]) -> BlockChange
{
    let Some(gone) = removed.iter().find(|gone| return gone.ordinal == block.ordinal)
    else
    {
        return BlockChange::Added {
            ordinal: block.ordinal,
            kind: Kind_Label(block.kind).to_owned(),
        };
    };

    return BlockChange::Reworded {
        ordinal: block.ordinal,
        before: ContentHash::Of(&gone.text).As_Str().to_owned(),
        after: ContentHash::Of(&block.text).As_Str().to_owned(),
    };
}

const fn Position_Of(change: &BlockChange) -> u32
{
    return match change
    {
        BlockChange::Added { ordinal, .. }
        | BlockChange::Removed { ordinal, .. }
        | BlockChange::Reworded { ordinal, .. }
        | BlockChange::Reflowed { ordinal } => *ordinal,
        BlockChange::Moved { to, .. } => *to,
    };
}

fn Unmatched(
    before: &[SourceBlock],
    taken: &[bool],
    predicate: impl Fn(&SourceBlock) -> bool,
) -> Option<usize>
{
    for (index, block) in before.iter().enumerate()
    {
        if Is_Free(taken, index) && predicate(block)
        {
            return Some(index);
        }
    }

    return None;
}

/// Whether the block at this index is still unclaimed.
///
/// An index past the end reads as taken, so a walk that runs off the slice matches nothing
/// rather than pairing a change with a block that is not there.
fn Is_Free(taken: &[bool], index: usize) -> bool
{
    return !taken.get(index).copied().unwrap_or(true);
}

fn Take(taken: &mut [bool], index: usize)
{
    if let Some(slot) = taken.get_mut(index)
    {
        *slot = true;
    }
}

/// Which front matter fields the edit changes.
fn Identity_Changes(before: &RecordFrontMatter, after: &RecordFrontMatter) -> Vec<IdentityChange>
{
    let mut changes = Vec::new();

    for (field, was, now) in [
        ("type", before.kind.clone(), after.kind.clone()),
        ("title", before.title.clone(), after.title.clone()),
        ("status", before.status.clone(), after.status.clone()),
        ("authority", before.authority.clone(), after.authority.clone()),
        ("version", before.version.to_string(), after.version.to_string()),
        ("tags", before.tags.join(", "), after.tags.join(", ")),
    ]
    {
        if was != now
        {
            changes.push(IdentityChange {
                field: field.to_owned(),
                before: was,
                after: now,
            });
        }
    }

    return changes;
}

fn Relation_Changes(before: &[RecordRelation], after: &[RecordRelation]) -> RelationChanges
{
    let added = after
        .iter()
        .filter(|relation| return !before.contains(relation))
        .cloned()
        .collect();
    let removed = before
        .iter()
        .filter(|relation| return !after.contains(relation))
        .cloned()
        .collect();

    return RelationChanges { added, removed };
}

/// Where a statement's canonical text sits before and after the edit.
///
/// Compared under the normalizer, because that is what decides whether two spellings are one
/// statement everywhere else in this system. A statement whose block was merely reflowed is
/// held, not moved.
fn Located(canonical_text: &str, before: &[SourceBlock], after: &[SourceBlock]) -> NormativeOutcome
{
    let needle = Normalize(canonical_text);
    let carrying = |blocks: &[SourceBlock]| {
        return blocks
            .iter()
            .find(|block| return Normalize(&block.text).contains(&needle))
            .map(|block| return block.ordinal);
    };

    return match (carrying(before), carrying(after))
    {
        (None, _) => NormativeOutcome::Unlocatable,
        (Some(from), None) => NormativeOutcome::Gone { from },
        (Some(from), Some(to)) if from == to => NormativeOutcome::Held { block: from },
        (Some(from), Some(to)) => NormativeOutcome::Moved { from, to },
    };
}

/// Why the staged text is not what this surface would have written.
fn Why_Not_Canonical(markdown: &str, record: &Record) -> String
{
    return match Render_Record(&record.front_matter, &Segment(&record.body))
    {
        Err(error) => error.to_string(),
        Ok(rendered) => First_Difference(&rendered, markdown),
    };
}

fn First_Difference(expected: &str, found: &str) -> String
{
    for (index, (left, right)) in expected.lines().zip(found.lines()).enumerate()
    {
        if left != right
        {
            return format!(
                "line {}: this surface writes {left:?} and the staged text has {right:?}",
                index.saturating_add(1)
            );
        }
    }

    let (written, staged) = (expected.lines().count(), found.lines().count());
    if written != staged
    {
        return format!("this surface writes {written} line(s) and the staged text has {staged}");
    }

    return "every line matches, so the difference is in the leading or trailing whitespace"
        .to_owned();
}

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
    return Ok(DeclaredRow {
        id: row.get(0)?,
        kind: row.get(1)?,
        title: row.get(2)?,
        authority: row.get(3)?,
        status: row.get(4)?,
        version: row.get(5)?,
        tags: row.get(6)?,
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
