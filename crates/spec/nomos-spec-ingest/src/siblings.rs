//! I6 and I8 — the sibling suites, and the game plans as lineage.
//!
//! A sibling's specification enters as a suite that is not the root. Without that
//! distinction a cross-suite relation looks exactly like an internal one and XVPE's D-085
//! reads as a decision this repository made. With it, "whose statement is this" is a
//! query, and the ecosystem contract's own boundary is expressible in the store rather
//! than only in prose.
//!
//! The game plans enter as commentary. D-120 says game-plan notes are lineage and never
//! sole authority, and [`Statements_Sourced_Only_From_Commentary`] is that sentence turned
//! into a query — a rule rather than a convention asking people to remember it.

use crate::archive::{Archive, ArchiveError};
use crate::phases::IngestError;
use nomos_spec_model::{Parse_Record, Segment};
use nomos_spec_store::{SpecificationStore, StoreError};
use serde::Deserialize;

/// The revision a game plan is ingested under.
///
/// Distinct from a corpus revision and from `authored`, so "what did the plan say" and
/// "what does the specification say" are never the same query.
pub const LINEAGE_NOTES: &str = "lineage-notes";

/// The authority every game-plan block carries, through the node it is disposed to.
pub const COMMENTARY: &str = "commentary";

/// The suite this repository's own specification belongs to.
pub const ROOT_SUITE: &str = "nomos";

/// A specification that is not this repository's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sibling
{
    Xvpe,
    Kwb,
    Ecosystem,
}

impl Sibling
{
    #[must_use]
    pub const fn Suite_Id(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "xvpe-spec-seed",
            Self::Kwb => "kwb-spec-seed",
            Self::Ecosystem => "ecosystem-contracts",
        };
    }

    #[must_use]
    pub const fn Title(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "XVPE specification seed",
            Self::Kwb => "KnowledgeWorkbench specification seed",
            Self::Ecosystem => "Ecosystem contracts",
        };
    }

    #[must_use]
    pub const fn Archive(self) -> &'static str
    {
        return match self
        {
            Self::Xvpe => "xvpe-spec-seed-v0.1.zip",
            Self::Kwb => "kwb-spec-seed-v0.1.zip",
            Self::Ecosystem => "ecosystem-contracts-v0.1.zip",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Xvpe, Self::Kwb, Self::Ecosystem];
    }
}

/// What a schema file declares about itself.
#[derive(Debug, Deserialize)]
struct SchemaHeader
{
    #[serde(default, rename = "$id")]
    id: String,
    #[serde(default)]
    title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuiteReport
{
    pub suite: String,
    pub documents: u32,
    pub blocks: u32,
    /// Record identifiers, named rather than counted.
    pub records: Vec<String>,
    /// Schema node identifiers: the files declaring a shape.
    pub schemas: Vec<String>,
    /// The rest of the machine layer: instance documents, not schemas.
    pub machine_documents: Vec<String>,
    /// Identifiers another suite already owns, so this suite did not take them.
    ///
    /// Named rather than merged. Two suites declaring one identifier is a real ecosystem
    /// problem and reassigning the node would hide it by making the last writer right.
    pub contested: Vec<String>,
}

/// I6 — one sibling suite, as a non-root suite.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure and [`IngestError::Parse`] if a record or
/// schema in the archive does not read.
pub fn Ingest_Sibling_Suite(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    sibling: Sibling,
) -> Result<SuiteReport, IngestError>
{
    let suite = Suite {
        sibling,
        uid: store.Put_Suite(sibling.Suite_Id(), sibling.Title(), false)?,
    };
    let mut report = SuiteReport {
        suite: sibling.Suite_Id().to_owned(),
        ..SuiteReport::default()
    };

    Ingest_Prose(store, archive, suite, &mut report)?;
    Ingest_Machine(store, archive, suite, &mut report)?;

    return Ok(report);
}

/// The suite a document is being ingested into: which sibling, and the row it occupies.
///
/// The two are never useful apart — the row is where a claim is written and the sibling is
/// what a claim is checked against — and carrying them together is what keeps the ingest
/// verbs inside the argument budget.
#[derive(Clone, Copy)]
struct Suite
{
    sibling: Sibling,
    uid: i64,
}

/// A document as the archive holds it: where it sits, and what it says.
struct Sourced<'a>
{
    entry: &'a str,
    text: &'a str,
}

/// What a document declares itself to be.
struct Declared
{
    id: String,
    kind: String,
    authority: String,
    title: String,
}

/// Every markdown entry in the suite.
fn Ingest_Prose(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    for entry in archive.Ending_With(".md")
    {
        let text = Text(archive, &entry)?;
        let declared = Declared_By(suite.sibling, &entry, &text)?;
        let node = Take(store, &declared, suite, report)?;

        let sourced = Sourced {
            entry: &entry,
            text: &text,
        };
        let ingested = Ingest_Document(store, suite, &sourced, node)?;
        report.blocks = report.blocks.saturating_add(ingested);
        report.documents = report.documents.saturating_add(1);
    }

    return Ok(());
}

/// What a markdown entry declares itself to be.
///
/// A record keeps its own declared identifier — D-085 is D-085 in every suite that names
/// it, which is what makes a cross-suite relation an ordinary row. Anything else is
/// suite-qualified, because a filename is only unique inside its own seed.
fn Declared_By(sibling: Sibling, entry: &str, text: &str) -> Result<Declared, IngestError>
{
    if !entry.contains("/records/")
    {
        return Ok(Declared {
            id: Qualified(sibling, entry),
            kind: "document".to_owned(),
            authority: "canonical".to_owned(),
            title: Stem(entry).to_owned(),
        });
    }

    let record =
        Parse_Record(text).map_err(|error| IngestError::Parse(format!("{entry}: {error}")))?;

    return Ok(Declared {
        id: record.front_matter.id,
        kind: record.front_matter.kind,
        authority: record.front_matter.authority,
        title: record.front_matter.title,
    });
}

/// Mints the node, and records whether this suite got to keep the identifier.
fn Take(
    store: &mut SpecificationStore,
    declared: &Declared,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<i64, IngestError>
{
    let node = store.Upsert_Node(
        &declared.id,
        &declared.kind,
        &declared.authority,
        "document",
        &declared.title,
    )?;

    if Claim(store, &declared.id, node, suite)?
    {
        report.records.push(declared.id.clone());
    }
    else
    {
        report.contested.push(declared.id.clone());
    }

    return Ok(node);
}

/// Every JSON entry in the suite.
fn Ingest_Machine(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    for entry in archive.Ending_With(".json")
    {
        let text = Text(archive, &entry)?;
        let header: SchemaHeader = serde_json::from_str(&text)
            .map_err(|error| IngestError::Parse(format!("{entry}: {error}")))?;
        let declared = Machine_Declared(suite.sibling, &entry, &header);
        Record_Machine(store, declared, suite, report)?;
    }

    return Ok(());
}

/// Mints one machine node and files it under what it is, or under the conflict.
fn Record_Machine(
    store: &mut SpecificationStore,
    declared: Declared,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    let node = store.Upsert_Node(
        &declared.id,
        &declared.kind,
        &declared.authority,
        "record",
        &declared.title,
    )?;

    if !Claim(store, &declared.id, node, suite)?
    {
        report.contested.push(declared.id);

        return Ok(());
    }
    Note_Machine(declared, report);

    return Ok(());
}

/// What a machine file declares itself to be.
///
/// Suite-qualified, because `target-adapter.schema.json` is unique inside its own seed and
/// nowhere else. Not every file under `machine/` is a schema either — five of the thirteen
/// are instance documents, an ownership matrix, a dependency inventory, an evidence
/// exchange — and typing them all `schema` would make "how many schemas does the ecosystem
/// define" answer with the file count instead.
fn Machine_Declared(sibling: Sibling, entry: &str, header: &SchemaHeader) -> Declared
{
    let title = if header.title.is_empty() { &header.id } else { &header.title };
    let kind = if Is_Schema(entry) { "schema" } else { "machine_document" };

    return Declared {
        id: Qualified(sibling, entry),
        kind: kind.to_owned(),
        authority: "canonical".to_owned(),
        title: title.clone(),
    };
}

/// Files a machine node this suite kept under what it actually is.
fn Note_Machine(declared: Declared, report: &mut SuiteReport)
{
    if declared.kind == "schema"
    {
        report.schemas.push(declared.id);
    }
    else
    {
        report.machine_documents.push(declared.id);
    }
}

/// I8 — a game plan, as commentary.
///
/// Every block is disposed to a node whose authority is [`COMMENTARY`], which is what
/// makes D-120 checkable. Ingested rather than referenced: the plan is lineage, and
/// lineage nobody stored cannot be traced to.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Game_Plan(
    store: &mut SpecificationStore,
    suite_uid: i64,
    path: &str,
    text: &str,
) -> Result<String, IngestError>
{
    let node_id = format!("PLAN-{}", Slug(Stem(path)));
    let node = store.Upsert_Node(&node_id, "game_plan", COMMENTARY, "document", Stem(path))?;
    store.Assign_Suite(node, suite_uid)?;

    let document_uid = store.Put_Source_Document(path, LINEAGE_NOTES, text)?;
    let blocks = Segment(text);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    for block in &blocks
    {
        Dispose(store, document_uid, block.ordinal, node)?;
    }

    return Ok(node_id);
}

/// Statements whose only preserved source is commentary.
///
/// D-120 as a query. A statement resting on a game plan and nothing else is a statement
/// the specification never made — the note is the reasoning that led to it, not the
/// authority for it. Statements with no preserved lineage at all are not reported here:
/// that is NSV-PRESERVE-006's subject, and answering it twice in two ways is how one
/// finding becomes two.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Statements_Sourced_Only_From_Commentary(
    store: &SpecificationStore,
) -> Result<Vec<String>, IngestError>
{
    let mut statement = Sql(store.Connection().prepare(
        "SELECT s.statement_id FROM normative_statements s
         WHERE EXISTS (
             SELECT 1 FROM lineage l
             WHERE l.target_statement = s.uid AND l.source_block_uid IS NOT NULL
               AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
               AND l.source_block_uid IN (SELECT block FROM commentary_blocks)
         )
         AND NOT EXISTS (
             SELECT 1 FROM lineage l
             WHERE l.target_statement = s.uid AND l.source_block_uid IS NOT NULL
               AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
               AND l.source_block_uid NOT IN (SELECT block FROM commentary_blocks)
         )
         ORDER BY s.statement_id",
    ))?;

    let rows = statement.query_map(rusqlite::params![], |row| row.get(0));
    let found = Sql(rows)?;

    return Sql(found.collect());
}

/// The blocks a commentary node owns, as a view the query above reads twice.
///
/// A view rather than a repeated subquery, so the two halves of "some commentary and no
/// other" cannot drift into asking different questions.
const COMMENTARY_BLOCKS: &str = "CREATE TEMP VIEW IF NOT EXISTS commentary_blocks AS
     SELECT DISTINCT l.source_block_uid AS block FROM lineage l
     JOIN nodes n ON n.uid = l.target_node_uid
     WHERE n.authority = 'commentary' AND l.source_block_uid IS NOT NULL";

/// Prepares the view the commentary query reads.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Prepare_Commentary_View(store: &SpecificationStore) -> Result<(), IngestError>
{
    Sql(store.Connection().execute_batch(COMMENTARY_BLOCKS))?;
    return Ok(());
}

/// Whether this suite took the identifier.
///
/// `false` where another suite already holds it. The node is left where it is: two suites
/// declaring one identifier is an ecosystem problem, and reassigning would make the last
/// ingest right rather than making the conflict visible.
fn Claim(
    store: &mut SpecificationStore,
    node_id: &str,
    node: i64,
    suite: Suite,
) -> Result<bool, IngestError>
{
    let held: Option<String> = store.Suite_Of(node_id)?.map(|(holder, _)| return holder);

    return match held.as_deref()
    {
        Some(holder) if holder != suite.sibling.Suite_Id() => Ok(false),
        Some(_) => Ok(true),
        None =>
        {
            store.Assign_Suite(node, suite.uid)?;
            Ok(true)
        }
    };
}

fn Ingest_Document(
    store: &mut SpecificationStore,
    suite: Suite,
    document: &Sourced<'_>,
    node_uid: i64,
) -> Result<u32, IngestError>
{
    let suite_id = suite.sibling.Suite_Id();
    let document_uid = store.Put_Source_Document(document.entry, suite_id, document.text)?;
    let blocks = Segment(document.text);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    for block in &blocks
    {
        Dispose(store, document_uid, block.ordinal, node_uid)?;
    }

    return Ok(u32::try_from(blocks.len()).unwrap_or(u32::MAX));
}

fn Dispose(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let disposed = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, node_uid],
    );
    Sql(disposed)?;

    return Ok(());
}

fn Text(archive: &mut Archive, entry: &str) -> Result<String, IngestError>
{
    return archive
        .Read_Text(entry)
        .map_err(|error: ArchiveError| return IngestError::Parse(error.to_string()));
}

/// A file that declares a shape, by the naming convention the seeds use.
fn Is_Schema(entry: &str) -> bool
{
    return entry.ends_with(".schema.json");
}

/// `xvpe-spec-seed:target-adapter.schema.json` — the suite, then the name inside it.
fn Qualified(sibling: Sibling, entry: &str) -> String
{
    return format!("{}:{}", sibling.Suite_Id(), Stem(entry));
}

/// The entry's file name, without the archive's top-level directory.
fn Stem(entry: &str) -> &str
{
    return entry.rsplit('/').next().unwrap_or(entry);
}

fn Slug(name: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in name.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_uppercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}

fn Sql<T>(result: rusqlite::Result<T>) -> Result<T, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
mod tests
{
    use super::*;

    const PLAN: &str = "4. How Nomos relates to KnowledgeWorkbench\n\n\
                        Nomos deterministically enforces the standards.\n";

    fn Rooted() -> (SpecificationStore, i64)
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let root = store
            .Put_Suite(ROOT_SUITE, "The Nomos specification", true)
            .expect("records the root suite");
        return (store, root);
    }

    #[test]
    fn Test_A_Game_Plan_Should_Enter_As_Commentary()
    {
        let (mut store, root) = Rooted();

        let node = Ingest_Game_Plan(&mut store, root, "nomos full game plan.txt", PLAN)
            .expect("ingests");

        assert_eq!(node, "PLAN-NOMOS-FULL-GAME-PLAN-TXT");
        let authority: String = store
            .Connection()
            .query_row(
                "SELECT authority FROM nodes WHERE node_id = ?1",
                rusqlite::params![node],
                |row| row.get(0),
            )
            .expect("queries");
        assert_eq!(authority, COMMENTARY);
    }

    /// Every block of a game plan is disposed, or NSV-PRESERVE-002 reports the plan as
    /// silently dropped content the moment it is ingested.
    #[test]
    fn Test_Every_Game_Plan_Block_Should_Be_Disposed_To_The_Commentary_Node()
    {
        let (mut store, root) = Rooted();
        Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");

        let undisposed: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM source_blocks b
                 WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
                [],
                |row| row.get(0),
            )
            .expect("queries");

        assert_eq!(undisposed, 0);
        assert!(store.Count(nomos_spec_store::Table::SourceBlocks).expect("counts") > 0);
    }

    /// D-120, mechanized. A statement resting on the plan and nothing else is reported.
    #[test]
    fn Test_A_Statement_Sourced_Only_From_A_Game_Plan_Should_Be_Reported()
    {
        let (mut store, root) = Rooted();
        Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
        Statement(&mut store, "AGT-001");
        Trace_To_Plan(&store, "AGT-001");
        Prepare_Commentary_View(&store).expect("prepares");

        assert_eq!(
            Statements_Sourced_Only_From_Commentary(&store).expect("queries"),
            vec!["AGT-001".to_owned()]
        );
    }

    /// The other half. A statement that also rests on real source is not reported, or the
    /// rule would forbid citing the plan at all rather than forbidding relying on it.
    #[test]
    fn Test_A_Statement_With_A_Real_Source_Too_Should_Not_Be_Reported()
    {
        let (mut store, root) = Rooted();
        Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
        crate::phases::Ingest_Source_Document(&mut store, "v.md", "v14.36", "# T\n\nReal.\n")
            .expect("ingests");
        Statement(&mut store, "AGT-001");
        Trace_To_Plan(&store, "AGT-001");

        store
            .Connection()
            .execute(
                "INSERT INTO lineage (source_block_uid, disposition, target_statement)
                 SELECT b.uid, 'preserved-verbatim', s.uid
                 FROM source_blocks b, source_documents d, normative_statements s
                 WHERE d.path = 'v.md' AND b.document_uid = d.uid AND b.ordinal = 2
                   AND s.statement_id = 'AGT-001'",
                [],
            )
            .expect("links the real source");
        Prepare_Commentary_View(&store).expect("prepares");

        assert!(
            Statements_Sourced_Only_From_Commentary(&store)
                .expect("queries")
                .is_empty()
        );
    }

    /// A statement with no lineage at all belongs to NSV-PRESERVE-006, not here.
    #[test]
    fn Test_A_Statement_With_No_Lineage_Should_Not_Be_Reported_Here()
    {
        let (mut store, root) = Rooted();
        Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
        Statement(&mut store, "AGT-002");
        Prepare_Commentary_View(&store).expect("prepares");

        assert!(
            Statements_Sourced_Only_From_Commentary(&store)
                .expect("queries")
                .is_empty()
        );
    }

    fn Statement(store: &mut SpecificationStore, id: &str)
    {
        let node = store
            .Upsert_Node(id, "requirement", "canonical", "record", id)
            .expect("mints");
        store
            .Connection()
            .execute(
                "INSERT INTO normative_statements
                 (node_uid, statement_id, kind, canonical_text, canonical_hash)
                 VALUES (?1, ?2, 'Requirement', 'Nomos shall.', 'sha256:aa')",
                rusqlite::params![node, id],
            )
            .expect("inserts");
    }

    fn Trace_To_Plan(store: &SpecificationStore, id: &str)
    {
        store
            .Connection()
            .execute(
                "INSERT INTO lineage (source_block_uid, disposition, target_statement)
                 SELECT b.uid, 'preserved-verbatim', s.uid
                 FROM source_blocks b, source_documents d, normative_statements s
                 WHERE d.revision = ?1 AND b.document_uid = d.uid AND b.ordinal = 1
                   AND s.statement_id = ?2",
                rusqlite::params![LINEAGE_NOTES, id],
            )
            .expect("links the plan");
    }

    #[test]
    fn Test_Suite_Identifiers_Should_Be_Distinct()
    {
        let mut seen = std::collections::BTreeSet::new();
        for sibling in Sibling::All()
        {
            assert!(seen.insert(sibling.Suite_Id()), "{} is declared twice", sibling.Suite_Id());
            assert_ne!(sibling.Suite_Id(), ROOT_SUITE, "a sibling claimed the root suite");
        }
    }

    #[test]
    fn Test_A_Qualified_Identifier_Should_Name_Its_Suite()
    {
        assert_eq!(
            Qualified(Sibling::Xvpe, "xvpe-spec-seed-v0.1/machine/target-adapter.schema.json"),
            "xvpe-spec-seed:target-adapter.schema.json"
        );
    }
}
