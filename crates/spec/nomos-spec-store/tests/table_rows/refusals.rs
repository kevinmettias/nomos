//! What the store will not accept, and that a refusal leaves nothing behind.
//!
//! Typing a row must not become a way out of NSV-PRESERVE-002's view that does not involve
//! leaving the table, so a table without a delimiter is refused rather than stored as prose.

use crate::common::{AUTHORED, Segment, SpecificationStore, StoreError, Stored};
use nomos_spec_store::Table;

/// Discards the store so a refusal can be asserted on: the store is not `Debug`, and
/// making it so purely to write `expect_err` would widen a public API for a test.
fn Refusal(markdown: &str) -> StoreError
{
    return match Stored(markdown)
    {
        Ok(_) => panic!("expected a refusal, got a store"),
        Err(error) => error,
    };
}

#[test]
fn Test_A_Table_Without_A_Delimiter_Should_Be_Refused()
{
    let refusal = Refusal("| a | b |\n| 1 | 2 |\n");

    assert!(matches!(refusal, StoreError::Table { .. }), "{refusal}");
}

#[test]
fn Test_A_Table_With_Two_Delimiters_Should_Be_Refused()
{
    let refusal = Refusal("| a |\n| --- |\n| --- |\n| 1 |\n");

    assert!(matches!(refusal, StoreError::Table { .. }), "{refusal}");
}

/// A refused block must leave nothing behind.
#[test]
fn Test_A_Refused_Table_Should_Not_Write_Half_A_Document()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let markdown = "| a | b |\n| 1 | 2 |\n";
    let document = store
        .Put_Source_Document("doc.md", AUTHORED, markdown)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &Segment(markdown))
        .expect_err("must refuse");

    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), 0);
    assert_eq!(store.Count(Table::SourceTableRows).expect("counts"), 0);
}

/// A lineage row must name something. The three-way CHECK is what stops the new column
/// from turning the constraint into "or nothing at all".
#[test]
fn Test_A_Lineage_Row_Naming_Nothing_Should_Be_Refused()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    let targetless = store.Connection().execute(
        "INSERT INTO lineage (disposition) VALUES ('preserved-verbatim')",
        [],
    );

    assert!(targetless.is_err(), "a lineage row must name what it is about");
}
