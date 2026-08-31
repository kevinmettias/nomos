//! Putting one document of one suite into the store.

use super::{Sibling, SpecificationStore, IngestError, Sourced, Segment, Archive, ArchiveError, StoreError};

/// The suite a document is being ingested into: which sibling, and the row it occupies.
///
/// The two are never useful apart — the row is where a claim is written and the sibling is
/// what a claim is checked against — and carrying them together is what keeps the ingest
/// verbs inside the argument budget.
#[derive(Clone, Copy)]
pub(super) struct Suite
{
    pub(super) sibling: Sibling,
    pub(super) uid: i64,
}

/// Whether this suite took the identifier.
///
/// `false` where another suite already holds it. The node is left where it is: two suites
/// declaring one identifier is an ecosystem problem, and reassigning would make the last
/// ingest right rather than making the conflict visible.
pub(super) fn Claim_Node_Id(
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

pub(super) fn Ingest_Document(
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
        Dispose_Block(store, document_uid, block.ordinal, node_uid)?;
    }

    return Ok(u32::try_from(blocks.len()).unwrap_or(u32::MAX));
}

pub(super) fn Dispose_Block(
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
    Wrap_Sql_Result(disposed)?;

    return Ok(());
}

pub(super) fn Read_Text(archive: &mut Archive, entry: &str) -> Result<String, IngestError>
{
    return archive
        .Read_Text(entry)
        .map_err(|error: ArchiveError| return IngestError::Parse(error.to_string()));
}

/// `xvpe-spec-seed:target-adapter.schema.json` — the suite, then the name inside it.
pub(super) fn Qualified_Node_Id(sibling: Sibling, entry: &str) -> String
{
    return format!("{}:{}", sibling.Suite_Id(), Path_Stem(entry));
}

/// The entry's file name, without the archive's top-level directory.
pub(super) fn Path_Stem(entry: &str) -> &str
{
    return entry.rsplit('/').next().unwrap_or(entry);
}

pub(super) fn Slug_Of_Name(name: &str) -> String
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

pub(super) fn Wrap_Sql_Result<Value>(result: rusqlite::Result<Value>) -> Result<Value, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
pub(super) mod tests
{
    use super::*;
    use crate::archive::tests::Zip_Fixture;
    use nomos_spec_store::{NodeRow, SuiteAuthority};

    #[test]
    fn Test_Claim_Node_Id_Should_Assign_A_Fresh_Identifier_And_Refuse_A_Different_Suites_Claim()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let suite = Suite_In(&mut store, Sibling::Xvpe);
        let node = store
            .Upsert_Node(NodeRow {
                node_id: "xvpe-spec-seed:a.md",
                kind: "document",
                authority: "canonical",
                representation: "document",
                title: "A",
            })
            .expect("mints");

        let claimed = Claim_Node_Id(&mut store, "xvpe-spec-seed:a.md", node, suite).expect("claims");
        assert!(claimed);

        let other_suite = Suite_In(&mut store, Sibling::Kwb);
        let contested = Claim_Node_Id(&mut store, "xvpe-spec-seed:a.md", node, other_suite).expect("checks");
        assert!(!contested);
    }

    #[test]
    fn Test_Ingest_Document_Should_Store_Its_Blocks_And_Dispose_Every_One()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let suite = Suite_In(&mut store, Sibling::Xvpe);
        let node = store
            .Upsert_Node(NodeRow {
                node_id: "xvpe-spec-seed:a.md",
                kind: "document",
                authority: "canonical",
                representation: "document",
                title: "A",
            })
            .expect("mints");

        let text = "# A\n\nFirst.\n\nSecond.\n";
        let sourced = Sourced { entry: "suite/a.md", text };

        let blocks = Ingest_Document(&mut store, suite, &sourced, node).expect("ingests");

        assert!(blocks > 0);
        let undisposed: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM source_blocks b
                 WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
                [],
                |row| return row.get(0),
            )
            .expect("queries");
        assert_eq!(undisposed, 0);
    }

    #[test]
    fn Test_Dispose_Block_Should_Record_Lineage_From_The_Block_To_The_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let node = store
            .Upsert_Node(NodeRow {
                node_id: "n",
                kind: "document",
                authority: "canonical",
                representation: "document",
                title: "n",
            })
            .expect("mints");
        let text = "# A\n\nBody.\n";
        let document = store.Put_Source_Document("a.md", "v14.36", text).expect("puts the document");
        let blocks = Segment(text);
        store.Put_Source_Blocks(document, &blocks).expect("puts the blocks");

        for block in &blocks
        {
            Dispose_Block(&mut store, document, block.ordinal, node).expect("disposes");
        }

        let disposed: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM lineage WHERE target_node_uid = ?1",
                rusqlite::params![node],
                |row| return row.get(0),
            )
            .expect("queries");
        assert_eq!(disposed, u32::try_from(blocks.len()).unwrap_or(u32::MAX));
    }

    #[test]
    fn Test_Read_Text_Should_Read_An_Entrys_Bytes_As_A_String()
    {
        let mut archive = Zip_Fixture("nomos-spec-ingest-document", "read-text", &[("suite/a.md", "# A\n\nBody.\n")]);

        let text = Read_Text(&mut archive, "suite/a.md").expect("reads");

        assert_eq!(text, "# A\n\nBody.\n");
    }

    #[test]
    fn Test_Qualified_Node_Id_Should_Combine_The_Suite_Id_With_The_Entrys_File_Name()
    {
        assert_eq!(
            Qualified_Node_Id(Sibling::Xvpe, "xvpe-spec-seed-v0.1/machine/target-adapter.schema.json"),
            "xvpe-spec-seed:target-adapter.schema.json"
        );
    }

    #[test]
    fn Test_Path_Stem_Should_Return_The_File_Name_After_The_Last_Slash()
    {
        assert_eq!(Path_Stem("xvpe-spec-seed-v0.1/00-index.md"), "00-index.md");
        assert_eq!(Path_Stem("00-index.md"), "00-index.md");
    }

    #[test]
    fn Test_Slug_Of_Name_Should_Upper_Case_And_Hyphenate_Non_Alphanumeric_Runs()
    {
        assert_eq!(Slug_Of_Name("nomos full game plan.txt"), "NOMOS-FULL-GAME-PLAN-TXT");
    }

    #[test]
    fn Test_Wrap_Sql_Result_Should_Convert_A_Sql_Error_Into_A_Store_Error()
    {
        let failure: rusqlite::Result<i64> = Err(rusqlite::Error::QueryReturnedNoRows);

        let wrapped = Wrap_Sql_Result(failure).expect_err("must wrap the failure");

        assert!(matches!(wrapped, IngestError::Store(StoreError::Sql(_))), "{wrapped}");
    }

    pub(in crate::siblings) fn Suite_In(store: &mut SpecificationStore, sibling: Sibling) -> Suite
    {
        let uid = store.Put_Suite(sibling.Suite_Id(), sibling.Title(), SuiteAuthority::Sibling).expect("puts the suite");

        return Suite { sibling, uid };
    }
}
