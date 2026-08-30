//! Writing v15 documents and records into the store, tracking filler as it goes.

use crate::FillerBlock;
use crate::IngestError;
use crate::Overlaid;
use nomos_spec_model::{Parse_Record, Segment, SourceBlock};
use nomos_spec_store::{DocumentPath, NodeRow, SpecificationStore, StoreError};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report
{
    pub documents: u32,
    pub blocks: u32,
    /// Blocks judged filler, each with the pattern that judged it and the v14 heading it
    /// displaced. Named, never counted.
    pub filler: Vec<FillerBlock>,
    pub records: u32,
}

/// I4 — ingests one v15 document, recording filler as filler.
///
/// A filler block is stored, not discarded: a stub is evidence of what was lost, and
/// dropping it would leave the regression report with nothing to point at. What makes it
/// filler is the lineage row, which names the pattern that judged it and the v14 heading
/// it displaced.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Overlay_Document(
    store: &mut SpecificationStore,
    document: &Overlaid<'_>,
    v14_headings: &BTreeMap<String, i64>,
    report: &mut Report,
) -> Result<(), IngestError>
{
    let path = document.path;
    let markdown = document.markdown;
    let document_uid = store.Put_Source_Document(path, "v15.0", markdown)?;
    let blocks = Segment(markdown);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    let heading = blocks
        .first()
        .filter(|block| block.kind == nomos_spec_model::BlockKind::Heading)
        .map(|block| return block.text.clone());
    let displaced = heading.as_ref().filter(|text| v14_headings.contains_key(*text));

    for block in &blocks
    {
        report.blocks = report.blocks.saturating_add(1);
        let filler = Filler { document_uid, path, displaced: displaced.map(String::as_str) };
        Note_Filler(store, block, &filler, report)?;
    }

    report.documents = report.documents.saturating_add(1);

    return Ok(());
}

/// Where a block sits, for the two records a filler block produces.
///
/// The lineage row and the report entry need the same three facts about the document, and
/// carrying them as one value is what keeps the block walk from naming all of them twice.
struct Filler<'a>
{
    /// The document row the lineage entry hangs off.
    document_uid: i64,
    /// The path the report entry names.
    path: &'a str,
    /// The v14 heading this document displaced, if it displaced one.
    displaced: Option<&'a str>,
}

/// Records one block as filler, if it is filler.
fn Note_Filler(
    store: &mut SpecificationStore,
    block: &SourceBlock,
    filler: &Filler<'_>,
    report: &mut Report,
) -> Result<(), IngestError>
{
    let Some(pattern) = super::Get_Filler_Pattern(&block.text)
    else
    {
        return Ok(());
    };

    Record_Filler(store, filler.document_uid, block.ordinal, pattern)?;
    report.filler.push(FillerBlock {
        document: filler.path.to_owned(),
        ordinal: block.ordinal,
        pattern,
        displaced: filler.displaced.map(str::to_owned),
    });

    return Ok(());
}

fn Record_Filler(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    pattern: &str,
) -> Result<(), StoreError>
{
    store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition)
         SELECT uid, ?3 FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, format!("regression-filler: {pattern}")],
    )?;

    return Ok(());
}

/// The records that exist only in v15, ingested as authored nodes.
///
/// They have to be in the store before the validator they govern is trusted, which is
/// why they are ingested rather than referenced.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if a record does not read.
pub fn Ingest_V15_Record<'a>(
    store: &mut SpecificationStore,
    path: impl Into<DocumentPath<'a>>,
    markdown: &str,
) -> Result<String, IngestError>
{
    let path = path.into().0;
    let record = Parse_Record(markdown)
        .map_err(|error| IngestError::Parse(format!("{path}: {error}")))?;

    store.Upsert_Node(NodeRow {
        node_id: &record.front_matter.id,
        kind: &record.front_matter.kind,
        authority: &record.front_matter.authority,
        representation: "document",
        title: &record.front_matter.title,
    })?;

    let document_uid = store.Put_Source_Document(path, "v15.0", markdown)?;
    store.Put_Source_Blocks(document_uid, &Segment(&record.body))?;

    return Ok(record.front_matter.id);
}

#[cfg(test)]
mod tests
{
    use super::*;

    const RECORD: &str = "---\nid: D-900\ntype: decision\ntitle: A title\nstatus: accepted\n\
                          version: 1\nauthority: canonical-normative-record\n---\n\n\
                          # A title\n\nBody.\n";

    #[test]
    fn Test_Ingest_V15_Record_Should_Upsert_The_Declared_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let id = Ingest_V15_Record(&mut store, "docs/records/d-900.md", RECORD).expect("ingests");

        assert_eq!(id, "D-900");
    }

    #[test]
    fn Test_Ingest_Overlay_Document_Should_Count_Blocks_And_Judge_No_Filler_In_Real_Prose()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = Overlaid { path: "09-reference/a.md", markdown: "# A\n\nSome real prose.\n" };
        let mut report = Report::default();

        Ingest_Overlay_Document(&mut store, &document, &BTreeMap::new(), &mut report).expect("ingests");

        assert_eq!(report.documents, 1);
        assert!(report.blocks > 0);
        assert!(report.filler.is_empty());
    }
}
