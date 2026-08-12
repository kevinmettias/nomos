//! The mandatory sentence: whether normative wording moved, and what the answer rests on.

use crate::common::{Previewed, SYNTHETIC, Swapped, With_Synthetic};
use nomos_spec_store::{BlockChange, EditPreview, NormativeOutcome};

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
