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

mod prose;
mod machine;
mod commentary;
mod document;
mod sibling;
#[cfg(test)]
mod tests;

use prose::{Declared, Sourced, Ingest_Prose};
pub use commentary::{Prepare_Commentary_View, Statements_Sourced_Only_From_Commentary};
use document::{Suite, Read_Text, Ingest_Document, Qualified_Node_Id, Path_Stem, Claim_Node_Id, Wrap_Sql_Result, Slug_Of_Name, Dispose_Block};
pub use sibling::Sibling;

use crate::SuiteReport;
use crate::Archive;
use crate::ArchiveError;
use crate::IngestError;
use nomos_spec_model::{Parse_Record, Segment};
use nomos_spec_store::{DocumentPath, NodeRow, SpecificationStore, StoreError, SuiteAuthority};
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
    use machine::Ingest_Machine;

    let suite = Suite {
        sibling,
        uid: store.Put_Suite(sibling.Suite_Id(), sibling.Title(), SuiteAuthority::Sibling)?,
    };
    let mut report = SuiteReport {
        suite: sibling.Suite_Id().to_owned(),
        ..SuiteReport::default()
    };

    Ingest_Prose(store, archive, suite, &mut report)?;
    Ingest_Machine(store, archive, suite, &mut report)?;

    return Ok(report);
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
pub fn Ingest_Game_Plan<'a>(
    store: &mut SpecificationStore,
    suite_uid: i64,
    path: impl Into<DocumentPath<'a>>,
    text: &str,
) -> Result<String, IngestError>
{
    let path = path.into().0;
    let node_id = format!("PLAN-{}", Slug_Of_Name(Path_Stem(path)));
    let node = store.Upsert_Node(NodeRow {
        node_id: &node_id,
        kind: "game_plan",
        authority: COMMENTARY,
        representation: "document",
        title: Path_Stem(path),
    })?;
    store.Assign_Suite(node, suite_uid)?;

    let document_uid = store.Put_Source_Document(path, LINEAGE_NOTES, text)?;
    let blocks = Segment(text);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    for block in &blocks
    {
        Dispose_Block(store, document_uid, block.ordinal, node)?;
    }

    return Ok(node_id);
}
