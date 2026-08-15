//! The corpus every claim below starts from, and the readings taken of it.
//!
//! One place to say what the precision corpus is, what an edit invalidated, and how a fact
//! or a rollup is named — so that a test reads as the claim it makes rather than as the
//! fixture it needs. `Answer_About` and `Publicly_Silent` are reached from one module each
//! and stay there.

use nomos_analysis::InvalidationReport;
use nomos_integration_tests::{
    Approximate_Floor, Corpus, Decode_Surface, Edited, Name_Keys, Slice, SourceFile, Surface, Walk
};
use nomos_workspace::ChangeSource;

/// The corpus small enough to know entirely.
pub(crate) const PRECISION_CORPUS: &str = "../corpus/analysis";
pub(crate) fn Precision_Corpus() -> Corpus
{
    use std::path::Path;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(PRECISION_CORPUS);
    let corpus = Walk(&root);

    assert!(
        corpus.unreadable.is_empty(),
        "the precision corpus is in this repository and must be readable: {:?}",
        corpus.unreadable
    );
    assert_eq!(
        corpus.files.len(),
        6,
        "the precision corpus is six files across three groups, and every assertion \
         below is written against that shape. Found: {:?}",
        corpus.files.iter().map(|file| return &file.path).collect::<Vec<&String>>()
    );

    return corpus;
}

/// The precision corpus with a fresh slice over it, which is where most tests below start.
pub(crate) fn Over_The_Precision_Corpus() -> (Corpus, Slice)
{
    let corpus = Precision_Corpus();
    let slice = Slice::Over(&corpus);

    return (corpus, slice);
}

/// What an edit invalidated, given that the edit was a change at all.
///
/// The `else` arm is the premise rather than the subject: a test asserting on what an edit
/// invalidated has nothing to say if the workspace never moved, so it fails here with the
/// outcome it did get instead of asserting over an empty report.
pub(crate) fn Advanced(edited: Edited) -> InvalidationReport
{
    let Edited::Advanced { invalidated, .. } = edited
    else
    {
        // `Edited::Unchanged` carries no report, so the only thing to return instead is an
        // empty one — and every caller's assertion is about what the invalidation reached.
        // All of them hold over nothing, so the edit that never landed would go unnoticed.
        panic!("this was supposed to be a change to the workspace: {edited:?}")
    };

    return invalidated;
}

/// One rewrite, as the door that announced it and the file it replaced.
#[derive(Clone, Copy)]
pub(crate) struct Rewrite<'a>
{
    pub(crate) source: ChangeSource,
    pub(crate) file: &'a str,
    pub(crate) text: &'a str,
}

/// A file rewritten through the workspace door, and what the change invalidated.
pub(crate) fn Rewritten(slice: &mut Slice, corpus: &mut Corpus, rewrite: Rewrite<'_>) -> InvalidationReport
{
    let edited = slice.Edit(corpus, rewrite.source, rewrite.file, rewrite.text);

    return Advanced(edited);
}

/// The facts an invalidation reached, directly and then along the edges the Reader recorded.
pub(crate) fn Reached(corpus: &Corpus, invalidated: &InvalidationReport) -> (Vec<String>, Vec<String>)
{
    return (
        Name_Keys(corpus, &invalidated.direct),
        Name_Keys(corpus, &invalidated.dependent),
    );
}

/// Every request the engine had to widen, as the fact, what was asked, and what was applied.
pub(crate) fn Broadenings(corpus: &Corpus, invalidated: &InvalidationReport) -> Vec<String>
{
    return invalidated
        .broadened
        .iter()
        .map(|record| {
            let named = Name_Keys(corpus, core::slice::from_ref(&record.key));

            return format!(
                "{} {:?} -> {:?}",
                named.first().cloned().unwrap_or_default(),
                record.requested,
                record.applied
            );
        })
        .collect();
}

/// One named file's contents, out of the corpus that holds it.
pub(crate) fn Source_Of(corpus: &Corpus, path: &str) -> String
{
    return corpus
        .files
        .iter()
        .find(|file| return file.path == path)
        // Callers name corpus paths as literals. A path the corpus does not hold is a test
        // editing a file no provider reads, and an empty string in its place would turn that
        // into an edit which correctly invalidated nothing.
        .map_or_else(|| panic!("the precision corpus contains {path}"), |file| return file.source.clone());
}

/// The one file every provider test asks about, out of the corpus that holds it.
pub(crate) fn Alpha_One(corpus: &Corpus) -> &SourceFile
{
    return corpus
        .files
        .iter()
        .find(|file| return file.path == "alpha/one.rs")
        .expect("the precision corpus contains alpha/one.rs");
}

/// A slice whose floor admits an approximate answer and which asks for the scanner.
///
/// The pairing is the point: lowering the floor is what makes the preference reachable, and
/// naming the preference without lowering the floor gets the parser back.
pub(crate) fn Loose(corpus: &Corpus) -> Slice
{
    use nomos_lang_rust_scan as scan;

    return Slice::Over(corpus)
        .Accepting(Approximate_Floor())
        .Preferring(scan::PROVIDER);
}

/// A group's decoded rollup, which is where every claim about coverage is finally settled.
pub(crate) fn Surface_Of(slice: &Slice, corpus: &Corpus, group: &str) -> Surface
{
    let members = corpus.In_Group(group);
    // No rollup means the run produced no fact for this group at all. Every coverage claim
    // downstream is decoded out of this payload, so the alternative is not a weaker answer —
    // it is no answer, arriving at the assertions as a group whose surface is simply empty.
    let fact = slice.Surface_Of(&members).unwrap_or_else(|| panic!("{group} has a rollup"));

    return Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");
}
