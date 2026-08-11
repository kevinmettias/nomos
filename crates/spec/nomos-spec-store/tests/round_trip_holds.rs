//! `D-129`'s round trip: a record read out of the store as markdown, and an edit written
//! back as a transaction that previews what it changes before committing.
//!
//! The assertions that matter are the two that could not be made before this existed. The
//! first is byte-identity from the *rows* rather than from the blob — the store holds enough
//! to be the substrate, or it holds a copy. The second is that the preview answers the
//! question `D-129` calls mandatory, and answers it for this repository's own records rather
//! than only for the corpus-ingested ones a statement table covers.

use nomos_spec_store::{
    AUTHORED,
    BlockChange,
    DocumentSource,
    EditError,
    EditPreview,
    GOVERNING_RECORD_IDS,
    NodeRow,
    NormativeOutcome,
    Seed_Governing_Records,
    SpecificationStore,
    Table,
};

/// The one document behind a record.
fn Only_Document(store: &SpecificationStore, id: &str) -> DocumentSource
{
    return store
        .Documents_Behind(id, None)
        .expect("queries")
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("{id} has no document behind it"));
}

/// How many edges join two nodes, in the direction named.
fn Edge_Count(store: &SpecificationStore, from: &str, to: &str) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = ?1 AND t.node_id = ?2",
            rusqlite::params![from, to],
            |row| return row.get(0),
        )
        .expect("queries");
}

/// What committing this edit to the synthetic record would change.
fn Previewed(store: &SpecificationStore, markdown: &str) -> EditPreview
{
    return store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(markdown, None)
        .expect("stages")
        .Preview(store)
        .expect("previews");
}

/// The synthetic record with its two sections in the other order and nothing else changed.
fn Swapped() -> String
{
    return SYNTHETIC.replace(
        "## Decision\n\nFirst paragraph.\n\n## Rationale\n\nSecond paragraph.\n",
        "## Rationale\n\nSecond paragraph.\n\n## Decision\n\nFirst paragraph.\n",
    );
}

/// A record written through the door, so the edit tests do not depend on the shape of any
/// particular governing record.
const SYNTHETIC: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                         status: accepted\nversion: 1\n\
                         authority: canonical-normative-record\ntags:\n  - testing\n\
                         relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                         # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                         ## Rationale\n\nSecond paragraph.\n";

const SYNTHETIC_PATH: &str = "docs/records/D-900-a-synthetic-record.md";

fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds");
    return store;
}

fn With_Synthetic() -> SpecificationStore
{
    let mut store = Seeded();
    store
        .Put_Record(SYNTHETIC_PATH, AUTHORED, SYNTHETIC)
        .expect("writes the synthetic record");
    return store;
}

/// Commits an edit through all four steps, so no test spells the sequence out twice.
fn Commit(store: &mut SpecificationStore, markdown: &str, rename: Option<&str>) -> String
{
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(markdown, rename)
        .expect("stages")
        .Preview(store)
        .expect("previews");
    let described = preview.Describe();

    store.Commit_Edit(&preview).expect("commits");

    return described;
}

fn Block_Uids(store: &SpecificationStore, path: &str) -> Vec<i64>
{
    return store
        .Connection()
        .prepare(
            "SELECT b.uid FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.path = ?1 ORDER BY b.ordinal",
        )
        .and_then(|mut statement| {
            return statement
                .query_map(rusqlite::params![path], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// The claim this whole item rests on. Rendering happens from the declared front matter and
/// the block rows, so passing means the store holds enough to reproduce the file — which is
/// what `D-129` means by the store being the substrate and markdown being a surface.
#[test]
fn Test_Every_Governing_Record_Should_Project_To_Its_Own_Bytes()
{
    let store = Seeded();
    let mut checked = 0_usize;

    for id in GOVERNING_RECORD_IDS
    {
        Assert_Projects_To_Its_Own_Bytes(&store, id);
        checked = checked.saturating_add(1);
    }

    assert_eq!(
        checked,
        GOVERNING_RECORD_IDS.len(),
        "the loop skipped records, so a pass here would mean less than it says"
    );
}

/// One record renders back to the bytes it was seeded from, and its two content addresses
/// agree about that.
fn Assert_Projects_To_Its_Own_Bytes(store: &SpecificationStore, id: &str)
{
    let projection = store
        .Record_Markdown(id, None)
        .unwrap_or_else(|error| panic!("{id}: {error}"));
    let source = Only_Document(store, id);

    assert_eq!(
        projection.markdown, source.text,
        "{id} does not render back to the bytes it was seeded from"
    );
    assert!(projection.Matches_Source(), "{id}: the content addresses disagree");
}

/// The negative control for the test above, and the reason the projection is worth having:
/// point the document at entirely different bytes and the markdown is unchanged, because it
/// was never read from there. Without this, an implementation that echoed the blob would pass
/// every other assertion in this file.
#[test]
fn Test_A_Projection_Should_Come_From_The_Rows_And_Not_The_Blob()
{
    let mut store = Seeded();
    let before = store.Record_Markdown("D-132", None).expect("projects");

    let other = store.Put_Blob(b"not a record at all").expect("writes a blob");
    store
        .Connection()
        .execute(
            "UPDATE source_documents SET blob_uid = ?2 WHERE path LIKE '%D-132%'",
            rusqlite::params![0_i64, other],
        )
        .expect("repoints the document");

    let after = store.Record_Markdown("D-132", None).expect("still projects");

    assert_eq!(after.markdown, before.markdown, "the projection followed the blob");
    assert!(
        !after.Matches_Source(),
        "the document now holds different bytes, and the projection must say so"
    );
}

/// Why `record_relations` exists rather than reading the graph. `relations` is completed with
/// inverses on the way in, and `relates-to` is its own inverse, so a record that is merely the
/// target of one has an outgoing edge it never wrote. Rendering front matter from the graph
/// would put that edge in the file.
#[test]
fn Test_The_Graph_Holds_An_Edge_The_Record_Never_Declared()
{
    let store = Seeded();
    let in_graph = Edge_Count(&store, "OD-LEDGER-001", "OD-LEDGER-009");
    let document = Only_Document(&store, "OD-LEDGER-001");
    let declared = store.Declared_Relations(document.uid).expect("queries");

    assert_eq!(in_graph, 1, "the inverse edge is not there, so this proves nothing");
    assert!(
        !declared
            .iter()
            .any(|relation| return relation.target == "OD-LEDGER-009"),
        "OD-LEDGER-001 declares an edge its file does not carry"
    );
    assert!(
        !store
            .Record_Markdown("OD-LEDGER-001", None)
            .expect("projects")
            .markdown
            .contains("OD-LEDGER-009"),
        "the projection invented a relation"
    );
}

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
    assert!(projection.Matches_Source(), "the stored bytes and the rows disagree");
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

/// The mandatory sentence, in the three cases that are not the same answer.
#[test]
fn Test_The_Preview_Should_Say_Whether_Normative_Wording_Moved()
{
    let store = With_Synthetic();

    let reworded = SYNTHETIC.replace("First paragraph.", "First paragraph, differently.");
    let reflowed = SYNTHETIC.replace("Second paragraph.", "Second  paragraph.");
    let appended = SYNTHETIC.replace(
        "Second paragraph.\n",
        "Second paragraph.\n\nA third paragraph.\n",
    );

    for (markdown, moved, why) in [
        (&reworded, true, "rewording a paragraph moves its wording"),
        (&reflowed, false, "whitespace is not wording under the normalizer"),
        (&appended, false, "adding a paragraph moves nothing that was there"),
    ]
    {
        let preview = Previewed(&store, markdown);

        assert_ne!(markdown.as_str(), SYNTHETIC, "the {why} case changed nothing");
        Assert_Answers_The_Mandatory_Question(&preview, moved, why);
    }
}

/// The preview says whether normative wording moved, and says it in those words.
fn Assert_Answers_The_Mandatory_Question(preview: &EditPreview, moved: bool, why: &str)
{
    assert_eq!(preview.Wording_Moved(), moved, "{why}: {}", preview.Describe());
    assert!(
        preview.Describe().contains("normative wording"),
        "the preview does not answer the mandatory question: {}",
        preview.Describe()
    );
}

/// A reflow reports as a reflow rather than as a rewording, because the normalizer — the
/// authority the whole preservation ledger uses — says the content is the same.
#[test]
fn Test_A_Reflowed_Block_Should_Be_Told_From_A_Reworded_One()
{
    let store = With_Synthetic();

    let of = |markdown: &str| {
        return store
            .Claim_For_Edit("D-900", None)
            .expect("claims")
            .Stage(markdown, None)
            .expect("stages")
            .Preview(&store)
            .expect("previews")
            .Blocks()
            .to_vec();
    };

    assert!(matches!(
        of(&SYNTHETIC.replace("Second paragraph.", "Second  paragraph.")).as_slice(),
        [BlockChange::Reflowed { .. }]
    ));
    assert!(matches!(
        of(&SYNTHETIC.replace("Second paragraph.", "A different sentence.")).as_slice(),
        [BlockChange::Reworded { .. }]
    ));
}

/// Moving a section is the case block ordinals alone would report as four rewordings. The
/// preview says one thing moved, and says the wording did not change on the way.
#[test]
fn Test_A_Moved_Block_Should_Report_As_Moved()
{
    let store = With_Synthetic();
    let swapped = Swapped();
    let preview = Previewed(&store, &swapped);

    assert_ne!(swapped, SYNTHETIC, "the negative control changed nothing");
    assert!(
        preview
            .Blocks()
            .iter()
            .all(|change| return matches!(change, BlockChange::Moved { .. })),
        "a pure reordering reported something other than movement: {}",
        preview.Describe()
    );
    assert!(preview.Wording_Moved(), "{}", preview.Describe());
}

/// The strong answer, where the store has the evidence for it. A normative statement recorded
/// against a record is followed by its canonical text rather than by its position, so the
/// preview can say which statement moved and where it went.
#[test]
fn Test_A_Recorded_Statement_Should_Be_Followed_Through_The_Edit()
{
    let store = With_Synthetic();
    store
        .Connection()
        .execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             SELECT uid, 'AGT-001', 'Requirement', 'Second paragraph.', 'sha256:aa'
             FROM nodes WHERE node_id = 'D-900'",
            [],
        )
        .expect("records a statement");
    let swapped = Swapped();
    let preview = Previewed(&store, &swapped);
    let movement = preview.Statements().first().expect("the statement is recorded");

    assert_eq!(movement.statement_id, "AGT-001");
    assert!(
        matches!(movement.outcome, NormativeOutcome::Moved { .. }),
        "{:?}",
        movement.outcome
    );
    assert!(preview.Describe().contains("AGT-001"), "{}", preview.Describe());
}

/// With no statement recorded, the preview must still answer — and must say what it derived
/// the answer from. Printing nothing would read as *no wording moved*, which is the shape
/// `OD-GATE-001` is about.
#[test]
fn Test_A_Record_With_No_Recorded_Statement_Should_Still_Get_An_Answer()
{
    let store = With_Synthetic();

    let edited = SYNTHETIC.replace("First paragraph.", "Something else.");
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(&edited, None)
        .expect("stages")
        .Preview(&store)
        .expect("previews");

    assert!(preview.Statements().is_empty(), "the fixture recorded a statement");
    assert!(preview.Wording_Moved());
    assert!(
        preview.Describe().contains("no normative statement is recorded"),
        "the preview does not say what its answer rests on: {}",
        preview.Describe()
    );
}

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

/// A relation the author adds reaches the graph, and one they remove leaves it — including its
/// inverse, or half the fact stays behind.
#[test]
fn Test_Editing_A_Relation_Should_Move_The_Graph_Both_Ways()
{
    let mut store = With_Synthetic();
    let retargeted = SYNTHETIC.replace(
        "  - target: D-129\n    type: relates-to\n",
        "  - target: D-130\n    type: affects\n",
    );

    Commit(&mut store, &retargeted, None);

    assert_eq!(Edge_Count(&store, "D-900", "D-130"), 1, "the added edge is missing");
    assert_eq!(Edge_Count(&store, "D-130", "D-900"), 1, "its inverse is missing");
    assert_eq!(Edge_Count(&store, "D-900", "D-129"), 0, "the removed edge is still there");
    assert_eq!(
        Edge_Count(&store, "D-129", "D-900"),
        0,
        "half the removed fact stayed behind"
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

/// Reading out a record nothing wrote through this door is a distinct answer from reading out
/// one that does not exist. A corpus document arrives as blocks and lineage and declares no
/// front matter, so it can be read, hashed and preserved but not rendered back.
#[test]
fn Test_A_Document_With_No_Declared_Front_Matter_Should_Say_So()
{
    let store = An_Ingested_Document();
    let refusal = store.Record_Markdown("VOL-001", None).expect_err("must refuse");

    assert!(matches!(refusal, EditError::NotAuthored { .. }), "{refusal}");
    assert!(
        matches!(
            store.Record_Markdown("VOL-999", None).expect_err("must refuse"),
            EditError::NoSuchRecord { .. }
        ),
        "an unknown identifier and an unauthored document report the same way"
    );
}

/// A document that arrived as blocks and lineage rather than through the authoring door, so
/// it can be read, hashed and preserved but not rendered back.
fn An_Ingested_Document() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let uid = store
        .Put_Source_Document("volumes/one.md", "v14.36", "# Volume\n\nBody.\n")
        .expect("writes");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "VOL-001",
            kind: "volume",
            authority: "canonical",
            representation: "document",
            title: "Volume",
        })
        .expect("writes a node");
    store
        .Put_Source_Blocks(uid, &nomos_spec_model::Segment("# Volume\n\nBody.\n"))
        .expect("writes blocks");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_node_uid)
             SELECT uid, 'preserved-verbatim', ?1 FROM source_blocks WHERE document_uid = ?2",
            rusqlite::params![node, uid],
        )
        .expect("disposes");

    return store;
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
    assert!(preview.Changes_Nothing(), "{}", preview.Describe());
    assert!(!preview.Wording_Moved());
    assert!(preview.Describe().contains("nothing changes"), "{}", preview.Describe());
}
