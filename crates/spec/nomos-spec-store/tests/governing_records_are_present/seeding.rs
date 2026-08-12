//! Seeding twice is seeding once, and what it seeds is content rather than identity alone.

use crate::common::{Counted, Seeded};
use nomos_spec_store::{
    AUTHORED, GOVERNING_RECORD_IDS, Seed_Governing_Records, SpecificationStore, Table,
};

/// Re-ingesting a document must not renumber its blocks.
///
/// `uid` is what every lineage and omission row points at, so a write that deletes the
/// block row and inserts a replacement takes the dispositions with it. Seeding twice is
/// what exposed this; the assertion is on the uids rather than on the counts because a
/// count survives a renumbering that destroys every reference.
#[test]
fn Test_Rewriting_A_Document_Should_Not_Renumber_Its_Blocks()
{
    let mut store = Seeded();
    let before = Block_Uids(&store);

    Seed_Governing_Records(&mut store).expect("seeds again");

    assert!(!before.is_empty(), "no blocks, so this test proved nothing");
    assert_eq!(before, Block_Uids(&store), "the blocks were reinserted under new uids");
}

fn Block_Uids(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_blocks ORDER BY document_uid, ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// Seeding twice is seeding once.
#[test]
fn Test_Seeding_Should_Be_Idempotent()
{
    let mut store = Seeded();
    let before: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    Seed_Governing_Records(&mut store).expect("seeds again");

    let after: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    assert_eq!(before, after);
}

/// The records are present as content, not only as identity. Without blocks there is
/// nothing for the preservation rules to examine and every one of them reports clean
/// having looked at nothing.
///
/// The count comparison here has one subject and it is the one the message names: the store
/// made one source document per governing record. It used to have a second, incidental one
/// — `RECORDS` and `GOVERNING_RECORD_IDS` were two lists kept by hand and this caught them
/// disagreeing in length. They are now generated from one directory and cannot, so that
/// subject is impossible by construction rather than checked. The remaining hazard, two
/// registrations naming one record file, is a refusal in the reader.
#[test]
fn Test_The_Records_Should_Be_Present_As_Disposed_Content()
{
    let store = Seeded();
    let empty = Counted(
        &store,
        "SELECT count(*) FROM source_documents d
         WHERE NOT EXISTS (SELECT 1 FROM source_blocks b WHERE b.document_uid = d.uid)",
    );
    let undisposed = Counted(
        &store,
        "SELECT count(*) FROM source_blocks b
         WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
    );

    assert_eq!(
        store.Count(Table::SourceDocuments).expect("counts") as usize,
        GOVERNING_RECORD_IDS.len(),
        "one source document per governing record, or a record reached the store as an \
         identity with no content behind it"
    );
    assert!(store.Count(Table::SourceBlocks).expect("counts") >= 40);
    assert!(store.Count(Table::SourceHeadings).expect("counts") >= 16);
    assert_eq!(empty, 0, "{empty} record(s) segmented to nothing");
    assert_eq!(undisposed, 0, "{undisposed} seeded block(s) have no disposition");
}

#[test]
fn Test_Authored_Content_Should_Be_Distinguishable_From_An_Ingested_Revision()
{
    let store = Seeded();
    let revisions = Counted(
        &store,
        &format!("SELECT count(*) FROM source_documents WHERE revision <> '{AUTHORED}'"),
    );

    assert_eq!(revisions, 0, "a governing record claims to come from a corpus revision");
}

/// A carriage return in an embedded record would change every hash it produces, so the
/// same commit would validate on one machine and not on another. `.gitattributes` pins
/// it; this is what checks the pin held.
#[test]
fn Test_No_Governing_Record_Should_Carry_A_Carriage_Return()
{
    let store = Seeded();
    let offenders = Blocks_Carrying_A_Carriage_Return(&store);

    assert!(offenders.is_empty(), "CRLF reached the store from {offenders:?}");
    assert!(
        store.Count(Table::SourceBlocks).expect("counts") > 0,
        "no blocks were examined, so this found nothing by looking at nothing"
    );
}

/// The documents holding a block with a carriage return in it.
fn Blocks_Carrying_A_Carriage_Return(store: &SpecificationStore) -> Vec<String>
{
    let mut statement = store
        .Connection()
        .prepare("SELECT path, text FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid")
        .expect("prepares");

    return statement
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let text: String = row.get(1)?;

            return Ok((path, text));
        })
        .expect("queries")
        .filter_map(Result::ok)
        .filter(|(_, text)| return text.contains('\r'))
        .map(|(path, _)| return path)
        .collect();
}
