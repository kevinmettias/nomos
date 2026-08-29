//! Every claim this suite makes against the scale corpus.
//!
//! All four corpus-gated tests are here with [`Scale_Corpus_Or_Skip`], the root that gates
//! them, and that is forced rather than chosen. `tests/contract/src/gates.rs` resolves a test
//! to the corpora it reaches by following calls **within one file**, and none of these tests
//! names `NOMOS_RUST_CORPUS` itself. A gated test left beside the claims it belongs with
//! would be gated in fact and counted by nobody, which would drop the declared size of the
//! hole `docs/records/OD-GATE-001` measures.
//!
//! So two of the four are here on loan from the module that owns their subject, and each says
//! so where it is defined.

use nomos_integration_tests::{Corpus, Decode_Surface, RunReport, Slice, Walk};

/// The corpus large enough to be a test.
const SCALE_CORPUS: &str = "F:/repos/xvpe";

/// The scale corpus, or an explanation.
///
/// `NOMOS_RUST_CORPUS` overrides the root; set and unreadable is a failure rather than a
/// skip, because a gate that quietly skips its own subject is worse than one that fails.
/// Absent and unconfigured — any machine that is not this one — returns, having asserted
/// nothing and said so.
fn Scale_Corpus_Or_Skip() -> Option<Corpus>
{
    use std::path::PathBuf;

    let configured = std::env::var_os("NOMOS_RUST_CORPUS");
    let root = configured
        .clone()
        .map_or_else(|| return PathBuf::from(SCALE_CORPUS), PathBuf::from);

    if !root.is_dir()
    {
        assert!(
            configured.is_none(),
            "NOMOS_RUST_CORPUS is set to {}, which is not a directory",
            root.display()
        );
        eprintln!("skipped: no corpus at {}", root.display());

        return None;
    }

    return Some(Walk(&root));
}

/// The headline, over the corpus nobody wrote for this test.
#[test]
fn Test_Facts_Should_Materialize_Over_The_Real_Corpus()
{
    let Some(corpus) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut slice = Slice::Over(&corpus);
    let first = slice.Run(&corpus);

    // The last assertion is that the workspace read the same corpus the providers did. A
    // member count that disagreed with the file count would mean the checkout and the walk
    // saw different trees, and every fact would be keyed on a workspace state that does not
    // describe what was parsed.
    Report_The_Run(&slice, &corpus, &first);
    Read_The_Whole_Corpus(&first);
    assert_eq!(
        slice.Workspace().Snapshot().Length(),
        first.files_seen,
        "the ingested workspace and the walked corpus must be the same tree"
    );
}

/// What the run over the scale corpus actually saw, printed beside the tree it read.
///
/// A count with no denominator beside it is unreadable when the test fails on somebody
/// else's machine, and the scale corpus is not in this repository.
fn Report_The_Run(slice: &Slice, corpus: &Corpus, first: &RunReport)
{
    eprintln!(
        "{}: {} members ingested as one checkout, snapshot {}, variant {:?}\n\
         {}: {} files, {} syntax facts materialized, {} refused, {} groups, {} rollups, \
         {} degraded",
        corpus.root.display(),
        slice.Workspace().Snapshot().Length(),
        slice.Workspace().Id(),
        slice.Workspace().Snapshot().Variant(),
        corpus.root.display(),
        first.files_seen,
        first.syntax_materialized,
        first.refused.len(),
        first.groups_seen,
        first.surface_materialized,
        first.degraded.len()
    );
}

/// The run reached the corpus this test is about, and every file in it reached exactly one
/// outcome. Each assertion here is a denominator: a truncated walk makes all of them pass
/// having read almost nothing.
fn Read_The_Whole_Corpus(first: &RunReport)
{
    assert!(
        first.files_seen >= 5_000,
        "{} files is not this corpus; every assertion below iterates over that set and a \
         truncated walk makes all of them pass having read almost nothing",
        first.files_seen
    );
    assert!(
        first.syntax_materialized > 0,
        "{} files produced no facts at all. A run that reads everything and materializes \
         nothing is broken, not satisfied",
        first.files_seen
    );
    assert_eq!(
        first.syntax_materialized.saturating_add(first.refused.len()),
        first.files_seen,
        "every file must reach exactly one outcome"
    );
    assert!(
        first.surface_materialized > 0,
        "{} groups produced no rollups; the derived layer never ran",
        first.groups_seen
    );
}

/// The claim a content-addressed fact key exists to make: unchanged input, no work.
#[test]
fn Test_A_Second_Run_Should_Materialize_Zero()
{
    let Some(corpus) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut slice = Slice::Over(&corpus);
    let first = slice.Run(&corpus);
    let second = slice.Run(&corpus);
    eprintln!(
        "second run over {} files: {} syntax materialized (was {}), {} rollups \
         materialized (was {}), {} syntax reused, {} rollups reused",
        second.files_seen,
        second.syntax_materialized,
        first.syntax_materialized,
        second.surface_materialized,
        first.surface_materialized,
        second.syntax_reused,
        second.surface_reused
    );

    Recognized_Rather_Than_Skipped(&second, &first);
}

/// Nothing was recomputed, and the reuse counts say the work was recognized rather than
/// skipped. Both figures are also zero for a run that did nothing at all, which is why the
/// second pair is not optional.
fn Recognized_Rather_Than_Skipped(second: &RunReport, first: &RunReport)
{
    assert_eq!(
        second.syntax_materialized, 0,
        "the corpus did not change and {} files were recomputed anyway",
        second.syntax_materialized
    );
    assert_eq!(
        second.surface_materialized, 0,
        "the corpus did not change and {} rollups were recomputed anyway",
        second.surface_materialized
    );
    assert_eq!(
        second.syntax_reused, first.syntax_materialized,
        "every fact materialized in the first run must be reused in the second"
    );
    assert_eq!(second.surface_reused, first.surface_materialized);
}

/// Coverage bought at scale, and what it cost.
///
/// The scale half of [`crate::providers`], and here rather than there only because it is
/// gated. That module settles what the registry hands back; this settles whether taking the
/// weaker answer is worth it over a corpus nobody wrote for the question.
#[test]
fn Test_The_Weaker_Provider_Should_Answer_For_The_Whole_Scale_Corpus()
{
    use crate::corpus::Loose;

    let Some(corpus) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let parsed = Slice::Over(&corpus).Run(&corpus);
    let scanned = Loose(&corpus).Run(&corpus);
    eprintln!(
        "coverage: parser {} of {} files, {} degraded rollups; \
         scanner {} of {} files, {} degraded rollups",
        parsed.syntax_materialized,
        parsed.files_seen,
        parsed.degraded.len(),
        scanned.syntax_materialized,
        scanned.files_seen,
        scanned.degraded.len()
    );

    Bought_Coverage(&parsed, &scanned);
}

/// The parser refuses something here, so there is coverage to buy; the scanner answers for
/// every file, so it is the provider this describes; and it leaves fewer degraded rollups
/// behind, so the coverage was actually bought.
fn Bought_Coverage(parsed: &RunReport, scanned: &RunReport)
{
    assert!(
        !parsed.refused.is_empty(),
        "the parser refuses nothing in this corpus, so there is no coverage to buy and \
         this test is measuring nothing"
    );
    assert_eq!(
        scanned.syntax_materialized, scanned.files_seen,
        "the scanner answers for every file or it is not the provider this describes"
    );
    assert!(
        scanned.degraded.len() < parsed.degraded.len(),
        "the weaker provider bought no coverage: {} degraded rollups against {}",
        scanned.degraded.len(),
        parsed.degraded.len()
    );
}

/// A signal that fires on everything is not a signal.
///
/// The first of the three lessons in [`crate::lessons`], and here rather than there only
/// because it is gated: the claim is a rate, and a rate needs a corpus large enough for one.
///
/// The rollup's derived judgement is "this directory declares nothing publicly". Over the
/// scale corpus it must fire on some directories and not on most: at one extreme it is
/// noise nobody can act on, at the other it is a check that never ran. Both bounds are
/// asserted, and the measured rate is reported next to the denominator that makes it
/// meaningful.
#[test]
fn Test_A_Derived_Signal_Should_Be_Neither_Silent_Nor_Background_Hum()
{
    let Some(corpus) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut slice = Slice::Over(&corpus);
    slice.Run(&corpus);
    let (groups, silent) = Publicly_Silent(&slice, &corpus);
    let percent = silent.saturating_mul(100).checked_div(groups).unwrap_or(0);
    eprintln!("signal: {silent} of {groups} directories declare nothing publicly ({percent}%)");
    assert!(
        silent > 0,
        "not one of {groups} directories declares nothing publicly. A check that finds \
         nothing across a whole corpus is broken rather than satisfied — that is the \
         single most repeated finding from the prototype"
    );
    assert!(
        percent < 75,
        "{silent} of {groups} directories ({percent}%) trip this. A signal that fires on \
         three quarters of its subjects is background hum, and reporting it as a result \
         teaches everyone to ignore the report"
    );
}

/// How many of the corpus's groups have a rollup at all, and how many of those declare
/// nothing publicly.
fn Publicly_Silent(slice: &Slice, corpus: &Corpus) -> (usize, usize)
{
    let mut groups = 0_usize;
    let mut silent = 0_usize;

    for group in corpus.Groups()
    {
        let Some(fact) = slice.Surface_Of(&corpus.In_Group(&group))
        else
        {
            continue;
        };
        let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

        groups = groups.saturating_add(1);
        silent = silent.saturating_add(usize::from(surface.public == 0));
    }

    return (groups, silent);
}
