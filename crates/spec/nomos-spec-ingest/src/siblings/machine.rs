//! Ingesting a sibling suite's machine-readable schemas.

use super::{Deserialize, NodeRow, SpecificationStore, Archive, Suite, SuiteReport, IngestError, Read_Text, Declared, Claim_Node_Id, Sibling, Qualified_Node_Id};

/// What a schema file declares about itself.
#[derive(Debug, Deserialize)]
pub(super) struct SchemaHeader
{
    #[serde(default, rename = "$id")]
    id: String,
    #[serde(default)]
    title: String,
}

/// Every JSON entry in the suite.
pub(super) fn Ingest_Machine(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    for entry in archive.Listing().Ending_With(".json")
    {
        let text = Read_Text(archive, &entry)?;
        let header: SchemaHeader = serde_json::from_str(&text)
            .map_err(|error| IngestError::Parse(format!("{entry}: {error}")))?;
        let declared = Machine_Declared(suite.sibling, &entry, &header);
        Record_Machine(store, declared, suite, report)?;
    }

    return Ok(());
}

/// Mints one machine node and files it under what it is, or under the conflict.
pub(super) fn Record_Machine(
    store: &mut SpecificationStore,
    declared: Declared,
    suite: Suite,
    report: &mut SuiteReport,
) -> Result<(), IngestError>
{
    let node = store.Upsert_Node(NodeRow {
        node_id: &declared.id,
        kind: &declared.kind,
        authority: &declared.authority,
        representation: "record",
        title: &declared.title,
    })?;

    if !Claim_Node_Id(store, &declared.id, node, suite)?
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
pub(super) fn Machine_Declared(sibling: Sibling, entry: &str, header: &SchemaHeader) -> Declared
{
    let title = if header.title.is_empty() { &header.id } else { &header.title };
    let kind = if Is_Schema(entry) { "schema" } else { "machine_document" };

    return Declared {
        id: Qualified_Node_Id(sibling, entry),
        kind: kind.to_owned(),
        authority: "canonical".to_owned(),
        title: title.clone(),
    };
}

/// Files a machine node this suite kept under what it actually is.
pub(super) fn Note_Machine(declared: Declared, report: &mut SuiteReport)
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

/// A file that declares a shape, by the naming convention the seeds use.
pub(super) fn Is_Schema(entry: &str) -> bool
{
    return entry.ends_with(".schema.json");
}
