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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::archive::tests::{FixturePrefix, Zip_Fixture};
    use crate::siblings::document::tests::Suite_In;
    use nomos_spec_store::SuiteAuthority;

    #[test]
    fn Test_Ingest_Prose_Should_Ingest_Every_Markdown_Entry_In_The_Suite()
    {
        let mut store = SpecificationStore::In_Memory()
            .expect("In_Memory migrates a fresh database, so no file or prior schema is involved");
        let suite_uid = store
            .Put_Suite(Sibling::Xvpe.Suite_Id(), Sibling::Xvpe.Title(), SuiteAuthority::Sibling)
            .expect("the fresh store holds no suite yet, so the xvpe suite is inserted");
        let suite = Suite { sibling: Sibling::Xvpe, uid: suite_uid };
        let mut archive =
            Zip_Fixture(FixturePrefix("nomos-spec-ingest-prose"), "ingest-prose", &[("suite/00-index.md", "# Index\n\nSome prose.\n")]);
        let mut report = SuiteReport::default();

        Ingest_Prose(&mut store, &mut archive, suite, &mut report)
            .expect("the fixture zip holds one markdown entry, so the walk reaches and ingests it");

        assert_eq!(report.documents, 1);
        assert!(report.blocks > 0);
        assert_eq!(report.records, vec!["xvpe-spec-seed:00-index.md".to_owned()]);
    }

    #[test]
    fn Test_Declared_By_Should_Qualify_A_Plain_Document_By_Its_Suite()
    {
        let declared = Declared_By(Sibling::Xvpe, EntryAt("suite/00-index.md"), EntryText("# Index\n\nText.\n"))
            .expect("the entry carries no front matter, so its heading is read as the declaration");

        assert_eq!(declared.id, "xvpe-spec-seed:00-index.md");
        assert_eq!(declared.kind, "document");
    }

    #[test]
    fn Test_Declared_By_Should_Read_A_Records_Own_Identifier_From_Its_Front_Matter()
    {
        let record_text = "---\nid: D-900\ntype: decision\ntitle: A title\nstatus: accepted\n\
                            version: 1\nauthority: canonical-normative-record\n---\n\n\
                            # A title\n\nBody.\n";

        let declared = Declared_By(Sibling::Xvpe, EntryAt("suite/records/d-900.md"), EntryText(record_text))
            .expect("record_text opens with front matter carrying an id, so the record declares itself");

        assert_eq!(declared.id, "D-900");
    }

    #[test]
    fn Test_Take_Node_Should_Report_A_Contested_Identifier_Rather_Than_A_Fresh_Claim()
    {
        let mut store = SpecificationStore::In_Memory()
            .expect("In_Memory migrates a fresh database, so no file or prior schema is involved");
        let first_suite = Suite_In(&mut store, Sibling::Xvpe);
        let second_suite = Suite_In(&mut store, Sibling::Kwb);

        let mut first_report = SuiteReport::default();
        Take_Shared_For_First(&mut store, first_suite, &mut first_report);

        let mut second_report = SuiteReport::default();
        Take_Shared_For_Second(&mut store, second_suite, &mut second_report);

        assert_eq!(first_report.records, vec!["shared-id".to_owned()]);
        assert_eq!(second_report.contested, vec!["shared-id".to_owned()]);
    }

    fn Take_Shared_For_First(
        mut store: &mut SpecificationStore,
        first_suite: Suite,
        mut first_report: &mut SuiteReport,
    )
    {
        Take_Node(
            &mut store,
            &Declared {
                id: "shared-id".to_owned(),
                kind: "document".to_owned(),
                authority: "canonical".to_owned(),
                title: "Shared".to_owned(),
            },
            first_suite,
            &mut first_report,
        )
        .expect("first_suite is the first suite to claim shared-id, so the node is minted rather than contested");
    }

    fn Take_Shared_For_Second(
        mut store: &mut SpecificationStore,
        second_suite: Suite,
        mut second_report: &mut SuiteReport,
    )
    {
        Take_Node(
            &mut store,
            &Declared {
                id: "shared-id".to_owned(),
                kind: "document".to_owned(),
                authority: "canonical".to_owned(),
                title: "Shared".to_owned(),
            },
            second_suite,
            &mut second_report,
        )
        .expect("a contested identifier is reported rather than refused, so the call still returns");
    }
}
