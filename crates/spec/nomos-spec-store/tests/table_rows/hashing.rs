//! A row hashes its own line, not the block around it.
//!
//! Made against the model rather than a store, because the claim is about what
//! `Table_Rows` produces — the store only persists whatever the row already says its hashes
//! are.

use nomos_spec_model::{SourceBlock, Table_Rows};

/// The separator carries no content, so normalization is what tells the two hashes apart
/// nowhere — but the row must still hash its own line, not the block's.
#[test]
fn Test_A_Row_Should_Hash_Its_Own_Line()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: nomos_spec_model::BlockKind::Prose,
        heading_path: Vec::new(),
        text: "| a | b |\n| --- | --- |".to_owned(),
    };
    let rows = Table_Rows(&block);

    let first = rows.first().expect("a row");
    assert_eq!(first.Content_Hash(), nomos_spec_model::ContentHash::Of("| a | b |"));
    assert_ne!(
        first.Content_Hash(),
        block.Content_Hash(),
        "the row hashed the whole block"
    );
}
