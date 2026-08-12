//! What the door refuses, and the transaction that leaves nothing behind when it does.

use crate::common::{Previewed, SYNTHETIC, SYNTHETIC_PATH, With_Synthetic};
use nomos_spec_store::{EditError, Table};

#[test]
fn Test_A_Staged_Text_Naming_A_Different_Record_Should_Be_Refused()
{
    let store = With_Synthetic();

    let edited = SYNTHETIC.replace("id: D-900", "id: D-901");
    let refusal = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(&edited, None)
        .expect_err("must refuse");

    assert!(matches!(refusal, EditError::IdentityChanged { .. }), "{refusal}");
}

/// An edit this surface cannot reproduce is refused rather than rewritten. Accepting it would
/// mean the commit changed bytes the author did not touch, and the next read-out would
/// disagree with the file for a reason nothing recorded.
#[test]
fn Test_An_Edit_This_Surface_Would_Not_Write_Should_Be_Refused()
{
    let store = With_Synthetic();

    for (markdown, why) in [
        (SYNTHETIC.replace("First paragraph.\n\n", "First paragraph.\n\n\n"), "a second blank line"),
        (format!("\u{feff}{SYNTHETIC}"), "a byte order mark"),
        (SYNTHETIC.replace("    type: relates-to", "    relation: relates-to"), "the other relation spelling"),
    ]
    {
        let refusal = store
            .Claim_For_Edit("D-900", None)
            .expect("claims")
            .Stage(&markdown, None)
            .expect_err(&format!("{why} must be refused"));

        assert!(matches!(refusal, EditError::NotCanonical { .. }), "{why}: {refusal}");
    }
}

/// Content leaves this store one way: an omission row carrying its reason and the decision
/// that allowed it. An edit that would delete the block underneath one is refused, because
/// deleting both is the pair of events the schema exists to prevent.
#[test]
fn Test_An_Edit_Should_Not_Delete_A_Block_A_Justified_Omission_Points_At()
{
    let mut store = With_Synthetic();
    store
        .Connection()
        .execute(
            "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
             SELECT b.uid, 'dropped', 'settled elsewhere', 'D-129'
             FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.path = ?1 ORDER BY b.ordinal DESC LIMIT 1",
            rusqlite::params![SYNTHETIC_PATH],
        )
        .expect("records an omission");

    let edited = SYNTHETIC.replace("\n## Rationale\n\nSecond paragraph.\n", "");
    let preview = Previewed(&store, &edited);
    let refusal = store.Commit_Edit(&preview).expect_err("must refuse");

    assert!(refusal.to_string().contains("justification"), "{refusal}");
    assert_eq!(
        store.Record_Markdown("D-900", None).expect("projects").markdown,
        SYNTHETIC,
        "the refused commit wrote part of itself"
    );
}

/// The vocabulary is governed, and an edit does not get to extend it. `relation_types` is a
/// foreign key for exactly this reason, and it had never been reached by an author before.
#[test]
fn Test_A_Relation_Type_Nothing_Declares_Should_Be_Refused()
{
    let mut store = With_Synthetic();

    let edited = SYNTHETIC.replace("type: relates-to", "type: invented-by-an-author");
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(&edited, None)
        .expect("stages")
        .Preview(&store)
        .expect("previews");

    assert!(store.Commit_Edit(&preview).is_err());
    assert_eq!(
        store.Record_Markdown("D-900", None).expect("projects").markdown,
        SYNTHETIC,
        "the refused commit left the record edited"
    );
}

/// A failed commit writes nothing. `In_Transaction` is what makes an edit a transaction
/// against the store rather than a sequence of writes that can stop halfway.
#[test]
fn Test_A_Refused_Commit_Should_Leave_Every_Table_As_It_Was()
{
    let mut store = With_Synthetic();
    let before: Vec<u32> = Table::All()
        .iter()
        .map(|table| return store.Count(*table).expect("counts"))
        .collect();

    let edited = SYNTHETIC.replace("type: relates-to", "type: invented-by-an-author");
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(&edited, None)
        .expect("stages")
        .Preview(&store)
        .expect("previews");
    assert!(store.Commit_Edit(&preview).is_err());

    let after: Vec<u32> = Table::All()
        .iter()
        .map(|table| return store.Count(*table).expect("counts"))
        .collect();
    assert_eq!(before, after);
}
