//! Ingesting a sibling suite's prose documents.

use super::{NodeRow, SpecificationStore, Archive, Suite, SuiteReport, IngestError, Read_Text, Ingest_Document, Sibling, Qualified_Node_Id, Path_Stem, Parse_Record, Claim_Node_Id};

/// A markdown entry's location in the archive, as opposed to the [`Sibling`] it came from
/// or the text it holds — the two `&str`-shaped things `Declared_By` sits next to.
pub(super) struct EntryAt<'a>(&'a str);

/// The bytes a markdown entry holds, distinct from where it sits ([`EntryAt`]).
pub(super) struct EntryText<'a>(&'a str);

/// A document as the archive holds it: where it sits, and what it says.
pub(super) struct Sourced<'a>
{
    pub(super) entry: &'a str,
    pub(super) text: &'a str,
}

/// What a document declares itself to be.
pub(super) struct Declared
{
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) authority: String,
    pub(super) title: String,
}

/// Every markdown entry in the suite.
pub(super) fn Ingest_Prose(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    for entry in archive.Listing().Ending_With(".md")
    {
        let text = Read_Text(archive, &entry)?;
        let declared = Declared_By(suite.sibling, EntryAt(&entry), EntryText(&text))?;
        let node = Take_Node(store, &declared, suite, report)?;

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
pub(super) fn Declared_By(sibling: Sibling, entry: EntryAt<'_>, text: EntryText<'_>) -> Result<Declared, IngestError>
{
    let entry = entry.0;
    let text = text.0;

    if !entry.contains("/records/")
    {
        return Ok(Declared {
            id: Qualified_Node_Id(sibling, entry),
            kind: "document".to_owned(),
            authority: "canonical".to_owned(),
            title: Path_Stem(entry).to_owned(),
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
pub(super) fn Take_Node(
    store: &mut SpecificationStore,
    declared: &Declared,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<i64, IngestError>
{
    let node = store.Upsert_Node(NodeRow {
        node_id: &declared.id,
        kind: &declared.kind,
        authority: &declared.authority,
        representation: "document",
        title: &declared.title,
    })?;

    if Claim_Node_Id(store, &declared.id, node, suite)?
    {
        report.records.push(declared.id.clone());
    }
    else
    {
        report.contested.push(declared.id.clone());
    }

    return Ok(node);
}
