//! An edit goes in as markdown and comes back out as the same bytes, and identity survives.

use crate::seeded::{
    Block_Uids, Commit, Only_Document, SYNTHETIC, SYNTHETIC_PATH, With_Synthetic,
};

/// The round trip, closed: an edit goes in as markdown and comes back out of the store as the
/// same bytes.
#[test]
fn Test_An_Edit_Should_Be_Readable_Back_Out_As_What_Was_Committed()
{
    let mut store = With_Synthetic();
    let edited = SYNTHETIC.replace("First paragraph.", "First paragraph, revised.");
    assert_ne!(edited, SYNTHETIC, "the negative control changed nothing");

    Commit(&mut store, &edited, None);

    let projection = store.Record_Markdown("D-900", None).expect("projects");
    assert_eq!(projection.markdown, edited);
    assert!(projection.Is_Matching_Source(), "the stored bytes and the rows disagree");
}

/// `done_when`'s identity clause. The node, the block surrogates every lineage row hangs from,
/// and the declared relations all survive an edit to the prose.
#[test]
fn Test_Identity_Should_Survive_An_Edit()
{
    let mut store = With_Synthetic();
    let node_before = store.Node_Uid("D-900").expect("queries");
    let blocks_before = Block_Uids(&store, SYNTHETIC_PATH);
    let relations_before = store.Node_Summary("D-900").expect("queries");

    let revised = SYNTHETIC.replace("Second paragraph.", "Second paragraph, revised.");

    Commit(&mut store, &revised, None);

    let document = Only_Document(&store, "D-900");

    assert!(!blocks_before.is_empty(), "no blocks, so this test proved nothing");
    assert_eq!(store.Node_Uid("D-900").expect("queries"), node_before);
    assert_eq!(Block_Uids(&store, SYNTHETIC_PATH), blocks_before);
    assert_eq!(store.Node_Summary("D-900").expect("queries"), relations_before);
    assert_eq!(store.Declared_Relations(document.uid).expect("queries").len(), 1);
}

/// `done_when`'s rename clause. The path moves, and nothing about the record's identity does —
/// which is only true because the document row is updated rather than replaced.
#[test]
fn Test_A_Rename_Should_Be_An_Ordinary_Edit()
{
    let mut store = With_Synthetic();
    let node_before = store.Node_Uid("D-900").expect("queries");
    let blocks_before = Block_Uids(&store, SYNTHETIC_PATH);
    let moved = "docs/records/D-900-renamed.md";

    let described = Commit(&mut store, SYNTHETIC, Some(moved));

    assert!(described.contains("renamed"), "{described}");
    assert_eq!(store.Node_Uid("D-900").expect("queries"), node_before);
    assert_eq!(Block_Uids(&store, moved), blocks_before, "the rename renumbered the blocks");
    assert!(Block_Uids(&store, SYNTHETIC_PATH).is_empty(), "the old path still holds blocks");
    assert_eq!(
        store.Record_Markdown("D-900", None).expect("projects").path,
        moved
    );
}

#[test]
fn Test_A_Rename_Onto_A_Path_Another_Record_Holds_Should_Be_Refused()
{
    use nomos_spec_store::EditError;

    let mut store = With_Synthetic();
    let taken = "docs/records/D-132-the-plan-is-a-game-plan.md";

    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(SYNTHETIC, Some(taken))
        .expect("stages")
        .Preview(&store)
        .expect("previews");
    let refusal = store.Commit_Edit(&preview).expect_err("must refuse");

    assert!(matches!(refusal, EditError::PathTaken { .. }), "{refusal}");
    assert_eq!(
        store.Record_Markdown("D-900", None).expect("projects").path,
        SYNTHETIC_PATH,
        "the refused commit moved the record anyway"
    );
}

/// A shorter record leaves no orphan blocks behind, and the surrogates of the blocks that
/// remain are untouched.
#[test]
fn Test_A_Shortening_Edit_Should_Prune_The_Blocks_It_Dropped()
{
    let mut store = With_Synthetic();
    let before = Block_Uids(&store, SYNTHETIC_PATH);
    let shortened = SYNTHETIC.replace("\n## Rationale\n\nSecond paragraph.\n", "");
    assert_ne!(shortened, SYNTHETIC, "the negative control changed nothing");

    Commit(&mut store, &shortened, None);

    let after = Block_Uids(&store, SYNTHETIC_PATH);
    assert_eq!(after.len(), before.len().saturating_sub(2));
    assert_eq!(after, before.get(..after.len()).unwrap_or_default());
    assert_eq!(
        store.Record_Markdown("D-900", None).expect("projects").markdown,
        shortened
    );
}

/// Committing the bytes that were read out changes nothing, and the preview says so rather
/// than reporting every block as touched.
#[test]
fn Test_Committing_What_Was_Read_Out_Should_Change_Nothing()
{
    let store = With_Synthetic();
    let claimed = store.Claim_For_Edit("D-900", None).expect("claims");
    let markdown = claimed.Markdown().to_owned();

    let preview = claimed
        .Stage(&markdown, None)
        .expect("stages")
        .Preview(&store)
        .expect("previews");

    assert_eq!(markdown, SYNTHETIC);
    assert!(preview.Has_No_Changes(), "{}", preview.Describe());
    assert!(!preview.Is_Wording_Moved());
    assert!(preview.Describe().contains("nothing changes"), "{}", preview.Describe());
}
