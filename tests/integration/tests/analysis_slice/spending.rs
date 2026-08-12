//! Spending the selection, per subject.
//!
//! A lowered floor is not a decision to take the weaker answer everywhere. The parser
//! answers for the files it can read, the scanner answers for the ones it refuses, and each
//! fact is filed under whichever provider actually produced it. What a run that fell back
//! must not do is report the corpus clean.

use crate::corpus::{Precision_Corpus, Surface_Of};
use nomos_analysis::FactStore;
use nomos_integration_tests::{Approximate_Floor, RunReport, Slice, SourceFile};
use nomos_lang_rust as rust;
use nomos_lang_rust_scan as scan;

/// The floor, spent.
///
/// `Test_A_Lowered_Floor_Should_Make_The_Weaker_Offer_Reachable_Without_Serving_It` above
/// asserts what the registry hands back and stops there, because when it was written
/// nothing walked the list. This is the walk: the parser answers for the five files it can
/// read, the scanner answers for the one it refuses, and each fact is filed under the
/// provider that produced it.
///
/// Note the configuration. No preference is named — a run that preferred the scanner would
/// get the scanner everywhere and buy its coverage at the price of precision on all six
/// files, which is the reflex answer `OD-CAPABILITY-001` rejected.
#[test]
fn Test_A_Lowered_Floor_Should_Be_Spent_On_The_Subjects_The_Parser_Refuses()
{
    let corpus = Precision_Corpus();
    let run = Slice::Over(&corpus).Accepting(Approximate_Floor()).Run(&corpus);

    assert!(
        run.refused.is_empty(),
        "something below the floor was still admitted and still refused: {:?}",
        run.refused
    );
    assert_eq!(
        run.Answered_By(rust::PROVIDER),
        5,
        "the parser must keep the files it can read: {:?}",
        run.answered_by
    );
    assert_eq!(
        run.Answered_By(scan::PROVIDER),
        1,
        "and the scanner must answer for exactly the one it cannot: {:?}",
        run.answered_by
    );
    assert_eq!(
        run.fell_back,
        vec![("gamma/broken.rs".to_owned(), scan::PROVIDER.to_owned())],
        "the subject that fell back is named, not counted"
    );
}

/// The negative control. A run that fell back must not read as a clean run.
///
/// This is the failure the whole decision turns on. The scanner covers the file the parser
/// refused, so `refused` empties and `degraded` empties with it — and if nothing else
/// changed, a corpus with a file no parser can read would be byte-indistinguishable from
/// one that parsed whole. Three separate things have to say otherwise, at three levels.
#[test]
fn Test_A_Run_That_Fell_Back_Should_Not_Report_The_Corpus_Clean()
{
    let corpus = Precision_Corpus();
    let parsed = Slice::Over(&corpus).Run(&corpus);
    let mut lowered = Slice::Over(&corpus).Accepting(Approximate_Floor());
    let covered = lowered.Run(&corpus);

    Said_So_At_Every_Level(&parsed, &covered);

    // The fact itself, which is the level that outlives the run.
    let surface = Surface_Of(&lowered, &corpus, "gamma");
    assert_eq!(surface.files, 2, "both members answered");
    assert_eq!(surface.unreachable, 0, "and neither was missing");
    assert_eq!(
        surface.approximate, 1,
        "one of them was pattern-matched rather than parsed, and the payload has to say \
         so — otherwise buying coverage also buys the appearance of precision"
    );
}

/// What the coverage bought, stated against the run that did not buy it, and what it did
/// not buy: the run itself and the rollup both have to say a weaker answer was used.
fn Said_So_At_Every_Level(parsed: &RunReport, covered: &RunReport)
{
    assert_eq!(parsed.refused.len(), 1, "{:?}", parsed.refused);
    assert_eq!(parsed.degraded, vec!["gamma"]);
    assert!(covered.refused.is_empty());
    assert!(covered.degraded.is_empty());
    assert!(
        !covered.Wholly_Chosen(),
        "a run that fell back reported itself as wholly served by the chosen provider"
    );
    assert!(
        parsed.Wholly_Chosen(),
        "the strict run admits nobody weaker, so it cannot have fallen back"
    );
    assert_eq!(
        covered.approximated,
        vec!["gamma"],
        "the group whose rollup read a weaker answer is named"
    );
    assert!(parsed.approximated.is_empty());
}

/// A directory the parser read whole must not be marked approximate.
///
/// Without this, the assertion above is satisfied by a rollup that reports every member
/// approximated, which would make the field noise rather than a signal.
#[test]
fn Test_A_Group_The_Parser_Read_Whole_Should_Not_Be_Marked_Approximate()
{
    let corpus = Precision_Corpus();
    let mut lowered = Slice::Over(&corpus).Accepting(Approximate_Floor());
    lowered.Run(&corpus);

    for group in ["alpha", "beta"]
    {
        let surface = Surface_Of(&lowered, &corpus, group);
        assert_eq!(surface.approximate, 0, "{group} parsed whole and is marked approximate");
        assert_eq!(surface.unreachable, 0, "{group} lost a member");
    }
}

/// One question, one answer: the store never holds two providers' syntax facts about one
/// file at one generation.
///
/// The property `OD-CAPABILITY-003` had to settle before the loop above could be written.
/// A `FactKey` names its provider, so nothing stops both being written — the run has to
/// stop at the first answer, and this is what checks that it does.
#[test]
fn Test_One_File_Should_Have_One_Syntax_Fact_At_A_Generation()
{
    let corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus).Accepting(Approximate_Floor());
    slice.Run(&corpus);

    let candidates = slice.Candidates();
    assert_eq!(
        candidates.len(),
        2,
        "this floor admits the parser and the scanner, and an assertion over one candidate \
         would prove nothing about a second"
    );
    for file in &corpus.files
    {
        let held = Answering(&slice, file, &candidates);
        assert_eq!(
            held.len(),
            1,
            "{} is answered by {held:?} — a file declares one thing at a generation, and \
             two providers holding facts about it is two answers under one question",
            file.path
        );
    }
}

/// Which of the admitted providers actually holds a syntax fact about this file right now.
fn Answering(slice: &Slice, file: &SourceFile, candidates: &[nomos_capability::ProviderOffer]) -> Vec<String>
{
    return candidates
        .iter()
        .filter(|offer| {
            let identity = slice.Syntax_Key_Of(file, offer).At(slice.Generation());

            return slice.Store().Current(&identity, slice.Generation()).is_some();
        })
        .map(|offer| return offer.provider.As_Str().to_owned())
        .collect();
}

/// Falling back is not re-materializing.
///
/// The second pass has to find the scanner's answer for `gamma/broken.rs` where the run
/// wrote it, which means asking the parser again, taking the refusal again, and then
/// hitting the fallback key. A run that recomputed the fallback every pass would defeat
/// the whole point of a content-addressed key for exactly the subjects that cost the most
/// to cover.
#[test]
fn Test_A_Second_Pass_Should_Reuse_The_Fallback_Answer()
{
    let corpus = Precision_Corpus();

    let mut slice = Slice::Over(&corpus).Accepting(Approximate_Floor());
    slice.Run(&corpus);
    let again = slice.Run(&corpus);

    assert_eq!(again.syntax_materialized, 0, "{:?}", again.recomputed);
    assert_eq!(again.syntax_reused, corpus.files.len());
    assert_eq!(again.surface_materialized, 0);
    assert_eq!(
        again.fell_back,
        vec![("gamma/broken.rs".to_owned(), scan::PROVIDER.to_owned())],
        "the reused answer still came from the weaker provider, and a second pass that \
         stopped saying so would report an exact corpus on the strength of a cache"
    );
}
