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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::archive::tests::Zip_Fixture;
    use crate::siblings::document::tests::Suite_In;
    use nomos_spec_store::SuiteAuthority;

    #[test]
    fn Test_Ingest_Machine_Should_Record_A_Schema_File_As_A_Schema_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let suite_uid =
            store.Put_Suite(Sibling::Xvpe.Suite_Id(), Sibling::Xvpe.Title(), SuiteAuthority::Sibling).expect("puts suite");
        let suite = Suite { sibling: Sibling::Xvpe, uid: suite_uid };
        let mut archive = Zip_Fixture(
            "nomos-spec-ingest-machine",
            "ingest-machine",
            &[("suite/machine/target-adapter.schema.json", r#"{"$id":"target-adapter","title":"Target adapter"}"#)],
        );
        let mut report = SuiteReport::default();

        Ingest_Machine(&mut store, &mut archive, suite, &mut report).expect("ingests");

        assert_eq!(report.schemas, vec!["xvpe-spec-seed:target-adapter.schema.json".to_owned()]);
        assert!(report.machine_documents.is_empty());
    }

    #[test]
    fn Test_Record_Machine_Should_File_A_Node_It_Cannot_Claim_As_Contested()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let xvpe = Suite_In(&mut store, Sibling::Xvpe);
        let kwb = Suite_In(&mut store, Sibling::Kwb);

        let mut first_report = SuiteReport::default();
        Record_Shared_For_Xvpe(&mut store, xvpe, &mut first_report);

        let mut second_report = SuiteReport::default();
        Record_Shared_For_Kwb(&mut store, kwb, &mut second_report);

        assert!(first_report.contested.is_empty());
        assert_eq!(second_report.contested, vec!["shared-id".to_owned()]);
    }

    fn Record_Shared_For_Xvpe(
        mut store: &mut SpecificationStore,
        xvpe: Suite,
        mut first_report: &mut SuiteReport,
    )
    {
        Record_Machine(
            &mut store,
            Declared {
                id: "shared-id".to_owned(),
                kind: "schema".to_owned(),
                authority: "canonical".to_owned(),
                title: "Shared".to_owned(),
            },
            xvpe,
            &mut first_report,
        )
        .expect("records");
    }

    fn Record_Shared_For_Kwb(
        mut store: &mut SpecificationStore,
        kwb: Suite,
        mut second_report: &mut SuiteReport,
    )
    {
        Record_Machine(
            &mut store,
            Declared {
                id: "shared-id".to_owned(),
                kind: "schema".to_owned(),
                authority: "canonical".to_owned(),
                title: "Shared".to_owned(),
            },
            kwb,
            &mut second_report,
        )
        .expect("records");
    }

    #[test]
    fn Test_Machine_Declared_Should_Type_A_File_By_The_Schema_Json_Naming_Convention()
    {
        let schema = SchemaHeader { id: "target-adapter".to_owned(), title: String::new() };
        let declared = Machine_Declared(Sibling::Xvpe, "suite/machine/target-adapter.schema.json", &schema);

        assert_eq!(declared.kind, "schema");
        assert_eq!(declared.id, "xvpe-spec-seed:target-adapter.schema.json");
        assert_eq!(declared.title, "target-adapter");

        let instance = SchemaHeader { id: "ownership".to_owned(), title: "Ownership matrix".to_owned() };
        let declared_instance = Machine_Declared(Sibling::Xvpe, "suite/machine/ownership.json", &instance);

        assert_eq!(declared_instance.kind, "machine_document");
        assert_eq!(declared_instance.title, "Ownership matrix");
    }

    #[test]
    fn Test_Note_Machine_Should_File_A_Node_By_Its_Kind_Into_The_Report()
    {
        let mut report = SuiteReport::default();
        Note_Machine(
            Declared { id: "a".to_owned(), kind: "schema".to_owned(), authority: "canonical".to_owned(), title: "A".to_owned() },
            &mut report,
        );
        Note_Machine(
            Declared {
                id: "b".to_owned(),
                kind: "machine_document".to_owned(),
                authority: "canonical".to_owned(),
                title: "B".to_owned(),
            },
            &mut report,
        );

        assert_eq!(report.schemas, vec!["a".to_owned()]);
        assert_eq!(report.machine_documents, vec!["b".to_owned()]);
    }

    #[test]
    fn Test_Is_Schema_Should_Recognize_The_Schema_Json_Suffix()
    {
        assert!(Is_Schema("target-adapter.schema.json"));
        assert!(!Is_Schema("ownership.json"));
    }
}
