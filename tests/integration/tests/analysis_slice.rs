//! The vertical slice.
//!
//! Every crate below this one passes its own tests over inputs its author chose. What
//! none of them can answer is whether the pieces compose — whether the key
//! `nomos-lang-rust` writes is the key `nomos_analysis::Reader` looks up, whether a fact
//! resolved through the registry is the fact the provider produced, whether invalidation
//! reaches a derived fact along an edge nobody declared by hand. Each is an agreement
//! between two crates, and an agreement is exactly what neither party can verify alone.
//!
//! # The three things the prototype learned expensively
//!
//! A rule firing thousands of times is background hum, and reporting it as a result
//! trains everyone to ignore the report. A check that found zero findings corpus-wide was
//! broken rather than satisfied, every time. And a negative control that has never failed
//! is not a control — it is a test whose failure mode nobody has observed.
//!
//! Each has a test here, and each is stated over its denominator.

use nomos_analysis::{FactStore, InvalidationReport, MaterializedFact};
use nomos_cap_syntax as syntax;
use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, FactVariant, Guarantee, IncrementalGranularity,
};
use nomos_integration_tests::{
    Approximate_Floor, Corpus, Decode_Surface, Edited, Host_Variant, Name_Keys, Registered,
    Resolved, Resolved_Configuration, RunReport, Slice, SourceFile, Surface, Walk,
    SURFACE_CAPABILITY,
};
use nomos_lang_rust as rust;
use nomos_lang_rust_scan as scan;
use nomos_workspace::ChangeSource;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The corpus small enough to know entirely.
const PRECISION_CORPUS: &str = "../corpus/analysis";

/// The corpus large enough to be a test.
const SCALE_CORPUS: &str = "F:/repos/xvpe";

fn Precision_Corpus() -> Corpus
{
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
fn Over_The_Precision_Corpus() -> (Corpus, Slice)
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
fn Advanced(edited: Edited) -> InvalidationReport
{
    let Edited::Advanced { invalidated, .. } = edited
    else
    {
        panic!("this was supposed to be a change to the workspace: {edited:?}")
    };

    return invalidated;
}

/// One rewrite, as the door that announced it and the file it replaced.
#[derive(Clone, Copy)]
struct Rewrite<'a>
{
    source: ChangeSource,
    file: &'a str,
    text: &'a str,
}

/// A file rewritten through the workspace door, and what the change invalidated.
fn Rewritten(slice: &mut Slice, corpus: &mut Corpus, rewrite: Rewrite<'_>) -> InvalidationReport
{
    let edited = slice.Edit(corpus, rewrite.source, rewrite.file, rewrite.text);

    return Advanced(edited);
}

/// The facts an invalidation reached, directly and then along the edges the Reader recorded.
fn Reached(corpus: &Corpus, invalidated: &InvalidationReport) -> (Vec<String>, Vec<String>)
{
    return (
        Name_Keys(corpus, &invalidated.direct),
        Name_Keys(corpus, &invalidated.dependent),
    );
}

/// Every request the engine had to widen, as the fact, what was asked, and what was applied.
fn Broadenings(corpus: &Corpus, invalidated: &InvalidationReport) -> Vec<String>
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
fn Source_Of(corpus: &Corpus, path: &str) -> String
{
    return corpus
        .files
        .iter()
        .find(|file| return file.path == path)
        .map_or_else(|| panic!("the precision corpus contains {path}"), |file| return file.source.clone());
}

/// The scale corpus, or an explanation.
///
/// `NOMOS_RUST_CORPUS` overrides the root; set and unreadable is a failure rather than a
/// skip, because a gate that quietly skips its own subject is worse than one that fails.
/// Absent and unconfigured — any machine that is not this one — returns, having asserted
/// nothing and said so.
fn Scale_Corpus_Or_Skip() -> Option<Corpus>
{
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

// ---------------------------------------------------------------------------------
// Facts materialize, and a second run materializes zero
// ---------------------------------------------------------------------------------

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
        slice.Workspace().Snapshot().Len(),
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
        slice.Workspace().Snapshot().Len(),
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

// ---------------------------------------------------------------------------------
// Touching one file recomputes exactly its descendants, named
// ---------------------------------------------------------------------------------

/// The precision property, and the reason a second corpus exists.
///
/// Asserted by naming every fact recomputed and every fact invalidated, because a count is
/// satisfied by recomputing the wrong things. `alpha/one.rs` changes; its own syntax fact
/// and `alpha`'s rollup must recompute, and `alpha/two.rs`, `beta` and `gamma` must not —
/// stated as set equality so that recomputing *too little* fails as loudly as too much.
#[test]
fn Test_Touching_One_File_Should_Recompute_Exactly_Its_Descendants()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    let first = slice.Run(&corpus);
    assert_eq!(first.syntax_materialized, 5, "five of six files parse");
    assert_eq!(first.surface_materialized, 3, "three groups");

    let before = slice.Generation();
    let invalidated = Rewritten(&mut slice, &mut corpus, Rewrite {
        source: ChangeSource::IdeEdit,
        file: "alpha/one.rs",
        text: "//! Rewritten.\n\npub fn Added() {}\n",
    });
    assert!(
        slice.Generation() > before,
        "the generation facts materialize into is the one the workspace produced"
    );
    assert_eq!(
        Reached(&corpus, &invalidated),
        (
            vec!["nomos.cap.syntax.items of alpha/one.rs".to_owned()],
            vec!["nomos.cap.module.surface of alpha".to_owned()],
        ),
        "exactly the changed file's own fact, and exactly the rollup that read it, along an \
         edge the Reader recorded"
    );
    Recomputed_Only_Alpha(&slice.Run(&corpus));
}

/// One file changed, so one syntax fact and one rollup recompute — and nothing in beta or
/// gamma, which nothing connects to alpha. Stated as set equality so that recomputing *too
/// little* fails as loudly as too much.
fn Recomputed_Only_Alpha(second: &RunReport)
{
    assert_eq!(second.Recomputed(), vec![
        "nomos.cap.module.surface of alpha",
        "nomos.cap.syntax.items of alpha/one.rs",
    ]);
    assert_eq!(
        second.syntax_reused, 4,
        "the four other parseable files are reused, not recomputed"
    );
    assert_eq!(second.surface_reused, 2, "beta and gamma are reused");
}

/// The negative control for the test above.
///
/// If invalidation propagated to everything, the assertion that `alpha` recomputes would
/// still pass. This is the half that fails when it over-propagates: touching a file in
/// `gamma` must leave `alpha` and `beta` alone, so the two tests together pin the
/// descendant set from both sides.
#[test]
fn Test_Touching_A_File_Should_Not_Reach_An_Unrelated_Group()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    slice.Run(&corpus);

    let invalidated = Rewritten(&mut slice, &mut corpus, Rewrite {
        source: ChangeSource::AgentEdit,
        file: "gamma/five.rs",
        text: "//! Rewritten.\n\nstruct Other;\n",
    });
    let reached = Name_Keys(&corpus, &invalidated.dependent);
    assert_eq!(reached, vec!["nomos.cap.module.surface of gamma"]);
    assert!(
        !reached.iter().any(|name| return name.ends_with("alpha") || name.ends_with("beta")),
        "a change in gamma reached {reached:?}"
    );
    assert_eq!(
        slice.Run(&corpus).Recomputed_For(SURFACE_CAPABILITY),
        vec!["gamma"],
        "only gamma's rollup recomputes"
    );
}

/// Content-addressed identity, tested in the one place invalidation cannot mask it.
///
/// # Why this test exists
///
/// It was written because a negative control failed to fail. Rewriting
/// [`Slice::Surface_Inputs`] to ignore its members entirely — making a rollup's identity a
/// function of the directory's name and nothing else — passed every other test in this
/// file. All of them call `Touch` before re-running, so recomputation is explained twice
/// over: the fact was invalidated *and* its key changed. An invalidated fact is not held
/// whatever its key says, so the key was never the thing being tested.
///
/// This is the case where only identity can save it: the corpus changes and nothing tells
/// the store. A git checkout, an edit from another process, a fresh run over a store
/// somebody persisted. The rollup has to recompute because its inputs are part of what it
/// *is*, not because somebody remembered to announce the change.
#[test]
fn Test_A_Change_Nobody_Announced_Should_Still_Not_Be_Served_Stale()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    let first = slice.Run(&corpus);
    assert_eq!(first.surface_materialized, 3, "three groups");

    // Deliberately not through `Slice::Edit`. The corpus on disk is now something the
    // workspace has never been told about, which is what a checkout behind a running
    // process, or a store reopened over a tree that moved, actually looks like.
    let workspace = slice.Workspace().Id();
    assert!(corpus.Rewrite("beta/four.rs", "//! Rewritten.\n\npub fn Now_Public() {}\n"));
    assert_eq!(
        slice.Workspace().Id(),
        workspace,
        "nothing announced the change, so the workspace must not have moved — this test is \
         about identity, and a workspace that somehow knew would explain the recomputation \
         a second way"
    );
    assert_eq!(
        slice.Run(&corpus).Recomputed(),
        vec![
            "nomos.cap.module.surface of beta",
            "nomos.cap.syntax.items of beta/four.rs",
        ],
        "a changed member makes a different rollup, with no invalidation involved"
    );
    assert_eq!(
        Rollup_Over(&slice, &corpus, "beta"),
        (6, 4),
        "and the recomputed rollup reflects the new contents rather than reusing the old \
         number under a key that no longer describes it"
    );
}

/// A group's rollup as the pair of numbers it carries: how many items, and how many public.
fn Rollup_Over(slice: &Slice, corpus: &Corpus, group: &str) -> (u32, u32)
{
    let members = corpus.In_Group(group);
    let fact = slice.Surface_Of(&members).unwrap_or_else(|| panic!("{group} has a rollup"));
    let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

    return (surface.items, surface.public);
}

/// The cost of the rollup's coarser granularity, recorded rather than absorbed.
///
/// The cause is file-granular. The rollup can only refresh a whole directory, so the
/// engine broadens the request to `Project` and says so. A provider that had declared
/// `File` it could not deliver would produce no broadening record and a rollup that was
/// quietly stale.
#[test]
fn Test_A_Coarser_Provider_Should_Broaden_The_Invalidation_And_Say_So()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    slice.Run(&corpus);

    let invalidated = Rewritten(&mut slice, &mut corpus, Rewrite {
        source: ChangeSource::IdeEdit,
        file: "alpha/two.rs",
        text: "//! Rewritten.\n\nfn Changed() {}\n",
    });
    let broadened = Broadenings(&corpus, &invalidated);
    assert_eq!(
        broadened,
        vec!["nomos.cap.module.surface of alpha File -> Project"],
        "the rollup is the only thing that cannot refresh at file granularity"
    );
    assert_eq!(
        invalidated.cause.Granularity(),
        IncrementalGranularity::File,
        "the cause is what happened, not what any provider could deliver"
    );
}

// ---------------------------------------------------------------------------------
// The context is derived, not invented
// ---------------------------------------------------------------------------------

/// Every component of a fact's context comes from something real.
///
/// The slice used to supply three byte-fill constants. An invented identity component
/// cannot be wrong, which is exactly why it is dangerous: two machines, two toolchains and
/// two policies all agree under it, and the disagreement they should have had is the one
/// the fact key exists to detect.
#[test]
fn Test_The_Fact_Context_Should_Come_From_The_Workspace()
{
    let (corpus, mut slice) = Over_The_Precision_Corpus();
    Every_Component_Is_Derived(&slice, &corpus);

    // The negative control, and the one that matters. If the snapshot were still a
    // constant, every one of the assertions above would pass while two entirely different
    // corpora shared one workspace identity — and one corpus's facts would answer for the
    // other's.
    let mut other = Precision_Corpus();
    assert!(other.Rewrite("beta/four.rs", "//! Different.\n\npub fn Elsewhere() {}\n"));
    assert_ne!(
        Slice::Over(&other).Workspace().Id(),
        slice.Workspace().Id(),
        "two corpora that differ in one file are two workspace states"
    );

    // And the state a fact records is the one it was measured against, which is a fact
    // about the fact rather than about the key it is filed under.
    slice.Run(&corpus);
    A_Fact_Names_The_Tree_It_Was_Read_From(&slice, &corpus);
}

/// The state a fact records is the one it was measured against, which is a statement about
/// the fact rather than about the key it is filed under — and the held value still agrees
/// with the workspace after a run.
fn A_Fact_Names_The_Tree_It_Was_Read_From(slice: &Slice, corpus: &Corpus)
{
    let alpha = slice.Surface_Of(&corpus.In_Group("alpha")).expect("alpha has a rollup");

    assert_eq!(alpha.snapshot, slice.Workspace().Id());
    assert_eq!(
        slice.Snapshot(),
        slice.Workspace().Id(),
        "and the held value still agrees after a run"
    );
}

/// Each component of the context comes from something real: the members, the workspace, the
/// binary, the composition. An invented one cannot be wrong, which is exactly why it is
/// dangerous — two machines and two toolchains agree under it, and the disagreement they
/// should have had is the one the fact key exists to detect.
fn Every_Component_Is_Derived(slice: &Slice, corpus: &Corpus)
{
    let snapshot = slice.Workspace().Snapshot();

    assert_eq!(snapshot.Len(), corpus.files.len(), "one member per file");
    assert_eq!(
        slice.Snapshot(),
        slice.Workspace().Id(),
        "the held snapshot is what the workspace is, not a second answer to it"
    );
    assert_eq!(
        snapshot.Variant(),
        &Host_Variant(),
        "the variant is what this binary was compiled as"
    );
    assert_eq!(
        snapshot.Configuration(),
        Resolved_Configuration(slice.Registry()),
        "the configuration is the composition that was just built, not a constant beside it"
    );
}

/// The assertion that closed OD-ANALYSIS-001.
///
/// # What this test used to say
///
/// It said the opposite, and the opposite was true. `alpha/one.rs` is byte-identical in
/// both corpora below — same provider, same guarantee, same semantic inputs — and the two
/// keys differed anyway, because a `FactKey` carried a `SnapshotId` and a workspace
/// snapshot is a digest over *all* of its members. A change to `beta/four.rs` re-addressed
/// the fact about `alpha/one.rs`, and the whole corpus with it.
///
/// The component is gone. A file's identity is now a statement about that file, and the two
/// workspace states it appears in are recorded beside its fact rather than folded into it.
#[test]
fn Test_An_Unchanged_File_Should_Keep_Its_Identity_Across_Workspace_States()
{
    let corpus = Precision_Corpus();
    let mut other = Precision_Corpus();
    assert!(other.Rewrite("beta/four.rs", "//! Different.\n\npub fn Elsewhere() {}\n"));

    let one = Slice::Over(&corpus);
    let two = Slice::Over(&other);
    let left = one.Syntax_Key(Alpha_One(&corpus));
    let right = two.Syntax_Key(Alpha_One(&other));
    // The premise. If the two slices sat over one workspace state, the equality below
    // would hold for a reason that has nothing to do with what this test is about.
    assert_ne!(
        one.Workspace().Id(),
        two.Workspace().Id(),
        "the two corpora must differ, or there is nothing here to be robust against"
    );
    assert_eq!(
        left.semantic_inputs, right.semantic_inputs,
        "the file itself did not change, so what the fact is computed from did not either"
    );
    assert_eq!(
        left.Digest(),
        right.Digest(),
        "and one unchanged file is one fact under both workspace states. A change to \
         beta/four.rs is not a fact about alpha/one.rs"
    );
    // The negative control. If a key ignored the file, every file in the corpus would
    // share one identity and the equality above would be worthless.
    let elsewhere = corpus
        .files
        .iter()
        .find(|file| return file.path == "alpha/two.rs")
        .expect("the precision corpus contains alpha/two.rs");
    assert_ne!(left.Digest(), one.Syntax_Key(elsewhere).Digest());
}

/// A checkout invalidates the files it touched, and stops there.
///
/// The property the substrate change bought, stated over the corpus that can name its
/// facts. `GenerationCause::SnapshotReplaced` used to match a snapshot on the key, which
/// meant every fact in the store — the whole corpus, for a checkout of one file. It now
/// names the members that differ, which is what a caller replacing a workspace state
/// actually has.
#[test]
fn Test_A_Checkout_Should_Invalidate_The_Members_It_Changed_And_No_More()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    slice.Run(&corpus);

    let before = slice.Workspace().Id();
    let replaced = slice.Checkout(
        &mut corpus,
        &[
            ("alpha/one.rs", "//! Checked out.\n\npub fn Landed() {}\n"),
            ("gamma/five.rs", "//! Checked out.\n\nstruct Also;\n"),
        ],
    );
    let invalidated = Advanced(replaced);
    assert_ne!(slice.Workspace().Id(), before, "the workspace is another state");
    assert_eq!(
        slice.Snapshot(),
        slice.Workspace().Id(),
        "and the slice followed it. A held snapshot that stops following the workspace is \
         the pin this item removed, wearing a cache"
    );
    Reached_Exactly_The_Two_Members(&corpus, &invalidated);
    Kept_What_The_Checkout_Did_Not_Touch(&slice.Run(&corpus));
}

/// Beta's two files and `alpha/two.rs` survived a checkout that did not name them, and
/// beta's rollup with them.
fn Kept_What_The_Checkout_Did_Not_Touch(second: &RunReport)
{
    assert_eq!(
        second.syntax_reused, 3,
        "beta's two files and alpha/two.rs survived a checkout that did not touch them"
    );
    assert_eq!(second.surface_reused, 1, "and beta's rollup with them");
}

/// The two members that differ, the rollups that read them, and nothing else — a
/// replacement that names its members is a statement about files rather than about the tree.
fn Reached_Exactly_The_Two_Members(corpus: &Corpus, invalidated: &InvalidationReport)
{
    assert_eq!(
        Reached(corpus, invalidated),
        (
            vec![
                "nomos.cap.syntax.items of alpha/one.rs".to_owned(),
                "nomos.cap.syntax.items of gamma/five.rs".to_owned(),
            ],
            vec![
                "nomos.cap.module.surface of alpha".to_owned(),
                "nomos.cap.module.surface of gamma".to_owned(),
            ],
        )
    );
    assert_eq!(
        invalidated.cause.Granularity(),
        IncrementalGranularity::File,
        "a replacement that names its members is a statement about files"
    );
}

/// A save that changed nothing must not cost anything.
///
/// The workspace answers `Applied::Unchanged` for a change set that says what it already
/// said, and the slice does not invalidate on it. An editor's save hook does not consult
/// the previous generation before writing, so this arrives constantly — and invalidating on
/// it would discard every fact reachable from the file in order to recompute the answers
/// the store already held.
#[test]
fn Test_A_Save_That_Changed_Nothing_Should_Invalidate_Nothing()
{
    let (mut corpus, mut slice) = Over_The_Precision_Corpus();
    let first = slice.Run(&corpus);
    let before = (slice.Generation(), slice.Workspace().Id());

    let unchanged = Source_Of(&corpus, "alpha/one.rs");
    let edited = slice.Edit(&mut corpus, ChangeSource::IdeEdit, "alpha/one.rs", &unchanged);
    assert!(
        matches!(edited, Edited::Unchanged { .. }),
        "an editor rewriting a file with its own contents changed nothing: {edited:?}"
    );
    assert_eq!(
        (slice.Generation(), slice.Workspace().Id()),
        before,
        "and neither the generation nor the workspace may move for it"
    );
    Nothing_Recomputed_And_Nothing_Lost(&slice.Run(&corpus), &first);
    // The positive control. If `Edit` reported `Unchanged` for everything, the assertions
    // above would pass over a door that cannot register a change at all.
    let changed = slice.Edit(
        &mut corpus,
        ChangeSource::IdeEdit,
        "alpha/one.rs",
        "//! Actually different.\n\npub fn Moved() {}\n",
    );
    assert!(matches!(changed, Edited::Advanced { .. }), "{changed:?}");
}

/// Nothing changed, so nothing recomputes — and every fact is still there to be reused,
/// which is what makes the first half an assertion about reuse rather than about a store
/// that lost everything.
fn Nothing_Recomputed_And_Nothing_Lost(second: &RunReport, first: &RunReport)
{
    assert_eq!(second.Recomputed(), Vec::<String>::new());
    assert_eq!(
        second.syntax_reused, first.syntax_materialized,
        "and every fact is still there to be reused, which is what makes the line above an \
         assertion about reuse rather than about a store that lost everything"
    );
}

/// Nothing in this crate invents an identity.
///
/// The literal form of the defect this item removed: `[0x51; 16]` as a snapshot,
/// `[0x52; 16]` as a variant, `[0x53; 16]` as a configuration. Asserted by reading the
/// crate's own source, because the type system cannot tell a digest of something from
/// sixteen bytes somebody typed — both are a [`nomos_contracts::Digest128`].
///
/// The banned spelling is assembled from its parts so that this file is subject to its own
/// rule. A scanner that had to exempt itself would leave the one file nobody is checking as
/// the easiest place to put the exemption.
#[test]
fn Test_Nothing_In_The_Slice_Should_Invent_An_Identity()
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scan = Scan_This_Crate(&root);

    // The guard against a vacuous pass. A scan that found no files reports a clean result
    // over nothing, which is the defect `Test_The_Workspace_Should_Not_Appear_Empty` exists
    // for one crate up.
    assert!(
        scan.read.len() >= 5,
        "scanned {} files under {}, which is not this crate: {:?}",
        scan.read.len(),
        root.display(),
        scan.read
    );
    assert!(
        scan.derived > 0,
        "not one scanned file mentions a content digest, so the reader is not reading Rust"
    );
    assert!(
        scan.offending.is_empty(),
        "these files build an identity out of literal bytes: {:?}.\n\
         An invented identity component cannot be wrong, so it can never be observed to be \
         wrong — two machines, two toolchains and two policies all agree under it. Every \
         identity here comes from a workspace, a digest of content, or a resolved \
         composition.",
        scan.offending
    );
}

/// What one pass over this crate's own source found.
///
/// `read` is the denominator, `offending` is the finding, and `derived` is the evidence
/// that the reader was reading Rust at all rather than an empty tree.
struct Scanned
{
    read: BTreeSet<String>,
    offending: Vec<String>,
    derived: usize,
}

/// Reads every `.rs` file this crate owns and looks for an identity built out of literal
/// bytes.
///
/// The banned spelling is assembled from its parts so that this file is subject to its own
/// rule. A scanner that had to exempt itself would leave the one file nobody is checking as
/// the easiest place to put the exemption.
fn Scan_This_Crate(root: &Path) -> Scanned
{
    let banned = ["Digest128", "From_Bytes"].join("::");
    let mut found = Scanned {
        read: BTreeSet::new(),
        offending: Vec::new(),
        derived: 0,
    };

    for directory in ["src", "tests"]
    {
        for file in Rust_Files(&root.join(directory))
        {
            Judge_One(&mut found, root, &file, &banned);
        }
    }

    return found;
}

/// One file read and filed: counted, and reported when it builds an identity by hand.
fn Judge_One(found: &mut Scanned, root: &Path, file: &Path, banned: &str)
{
    let text = std::fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("{} is this crate's own source: {error}", file.display()));
    let named = file.strip_prefix(root).unwrap_or(file).display().to_string();

    found.read.insert(named.clone());
    if text.contains(banned)
    {
        found.offending.push(named);
    }
    if text.contains("Content_Digest") || text.contains("Digest_Of_Parts")
    {
        found.derived = found.derived.saturating_add(1);
    }
}

/// Every `.rs` file under a directory, recursively.
fn Rust_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };
        for entry in entries.flatten()
        {
            File_Or_Directory(entry.path(), &mut found, &mut pending);
        }
    }

    return found;
}

/// One entry filed: a directory to descend into later, a Rust file to keep, or neither.
fn File_Or_Directory(path: PathBuf, found: &mut Vec<PathBuf>, pending: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path);
    }
    else if path.extension().is_some_and(|extension| return extension == "rs")
    {
        found.push(path);
    }
}

// ---------------------------------------------------------------------------------
// Two providers of one capability
// ---------------------------------------------------------------------------------

/// The one file every provider test asks about, out of the corpus that holds it.
fn Alpha_One(corpus: &Corpus) -> &SourceFile
{
    return corpus
        .files
        .iter()
        .find(|file| return file.path == "alpha/one.rs")
        .expect("the precision corpus contains alpha/one.rs");
}

/// One provider's answer about one file, read back out of its own store.
fn Answer_About(slice: &Slice, file: &SourceFile) -> MaterializedFact
{
    let key = slice.Syntax_Key(file).At(slice.Generation());

    return slice
        .Store()
        .Current(&key, slice.Generation())
        .expect("this provider answered for alpha/one.rs");
}

/// A slice whose floor admits an approximate answer and which asks for the scanner.
///
/// The pairing is the point: lowering the floor is what makes the preference reachable, and
/// naming the preference without lowering the floor gets the parser back.
fn Loose(corpus: &Corpus) -> Slice
{
    return Slice::Over(corpus)
        .Accepting(Approximate_Floor())
        .Preferring(scan::PROVIDER);
}

/// A group's decoded rollup, which is where every claim about coverage is finally settled.
fn Surface_Of(slice: &Slice, corpus: &Corpus, group: &str) -> Surface
{
    let members = corpus.In_Group(group);
    let fact = slice.Surface_Of(&members).unwrap_or_else(|| panic!("{group} has a rollup"));

    return Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");
}

/// A floor only one offer clears resolves to that one.
#[test]
fn Test_A_Requirement_Only_One_Provider_Satisfies_Should_Resolve_To_That_One()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection: parsed,
        applicability: how,
    } = Slice::Over(&corpus).Resolved();

    assert_eq!(parsed.chosen.provider.As_Str(), rust::PROVIDER);
    assert_eq!(how, Applicability::Supported);
    assert!(
        parsed.alternatives.is_empty(),
        "only one offer clears this floor, so there is nothing to have been chosen over: {:?}",
        parsed.alternatives
    );
    // The scanner is registered and cannot serve this floor. Without that, the assertion
    // above passes over a registry that still has only one offer in it.
    assert_eq!(
        Loose(&corpus).Resolved().selection.chosen.provider.As_Str(),
        scan::PROVIDER,
        "the scanner is in the registry and can be reached"
    );
}

/// A preference that cannot be served is a fallback, and the caller is told.
///
/// This is the branch that was unreachable with one provider: naming a preference always
/// got it, so `SupportedWithFallback` had never been produced. The answer still stands —
/// the floor was met — and its provenance is not what was asked for, which is a thing the
/// caller has to be able to record.
#[test]
fn Test_A_Preference_That_Cannot_Be_Served_Should_Report_A_Fallback()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection,
        applicability: how,
    } = Slice::Over(&corpus).Preferring(scan::PROVIDER).Resolved();

    assert_eq!(
        selection.chosen.provider.As_Str(),
        rust::PROVIDER,
        "the preference cannot meet the floor, so it must not be honoured"
    );
    assert_eq!(
        how,
        Applicability::SupportedWithFallback,
        "and the caller must be told, or it cannot record why the answer came from \
         somewhere else"
    );
    // The positive control. A preference that *can* be served is not a fallback, and
    // without this the assertion above would pass over a registry that never honours one.
    assert_eq!(Loose(&corpus).Resolved().applicability, Applicability::Supported);
}

/// Two providers' answers about one file are two facts.
///
/// The key names the provider and its guarantee, so a store holding both holds them apart.
/// Filing them together would make "what does this file declare" answerable two ways under
/// one address, and whichever was written last would win.
#[test]
fn Test_Facts_From_Two_Providers_Should_Not_Share_A_Key()
{
    let corpus = Precision_Corpus();
    let file = Alpha_One(&corpus);
    let parsed = Slice::Over(&corpus).Syntax_Key(file);
    let scanned = Loose(&corpus).Syntax_Key(file);

    assert_eq!(parsed.subject, scanned.subject, "one file");
    assert_eq!(
        parsed.semantic_inputs, scanned.semantic_inputs,
        "and one set of bytes, so the two are answering the same question"
    );
    assert_ne!(parsed.provider, scanned.provider);
    assert_ne!(parsed.guarantee, scanned.guarantee);
    assert_ne!(
        parsed.Digest(),
        scanned.Digest(),
        "a weak answer and a strong one about one file must not share an address"
    );
}

/// The disagreement, over the corpus that can name it.
///
/// A parser refuses `gamma/broken.rs` — a stray byte order mark mid-file, which is not
/// valid Rust — so `gamma`'s rollup covers one of its two members and says so. A
/// line-reader has no refusal case: it answers for every file, weakly.
///
/// This is the trade the capability system exists to make explicit. Not "is there an
/// answer" but "what is this answer worth, and did the caller ask for one that good".
#[test]
fn Test_The_Weaker_Provider_Should_Answer_Where_The_Parser_Refuses()
{
    let corpus = Precision_Corpus();
    let parsed = Slice::Over(&corpus).Run(&corpus);
    assert_eq!(parsed.refused.len(), 1, "{:?}", parsed.refused);
    assert_eq!(parsed.degraded, vec!["gamma"]);

    let mut loose = Loose(&corpus);
    Answered_For_Everything(&loose.Run(&corpus));

    // What the coverage cost. The scanner reads gamma's second file and reports items in
    // it, which the parser could not — and the rollup that follows is an approximation,
    // which is exactly what the caller asked for by lowering its floor.
    let gamma = loose.Surface_Of(&corpus.In_Group("gamma")).expect("gamma has a rollup");
    let surface = Decode_Surface(&gamma.payload.bytes).expect("the rollup wrote this");
    assert_eq!(surface.files, 2, "both of gamma's files answered");
    assert_eq!(surface.unreachable, 0);
}

/// A line-reader has no refusal case, so every file gets an answer and no rollup is missing
/// a member — six files, six answers, where the parser managed five.
fn Answered_For_Everything(scanned: &RunReport)
{
    assert!(
        scanned.refused.is_empty(),
        "a line-reader has no refusal case: {:?}",
        scanned.refused
    );
    assert!(
        scanned.degraded.is_empty(),
        "and so no rollup is missing a member: {:?}",
        scanned.degraded
    );
    assert_eq!(
        scanned.syntax_materialized, 6,
        "six files, six answers, where the parser managed five"
    );
}

/// The two providers do not merely differ in guarantee — they differ in what they say.
///
/// Asserted over a file both can read, so the disagreement is about method rather than
/// about one of them having refused. If their payloads were identical the whole
/// arrangement would be theatre: two names for one answer, and no reason for a caller to
/// care which one it got.
#[test]
fn Test_The_Two_Providers_Should_Disagree_About_A_File_Both_Can_Read()
{
    let corpus = Precision_Corpus();
    let file = Alpha_One(&corpus);
    let mut strict = Slice::Over(&corpus);
    let mut loose = Loose(&corpus);
    strict.Run(&corpus);
    loose.Run(&corpus);

    let parsed = Answer_About(&strict, file);
    let scanned = Answer_About(&loose, file);
    assert_eq!(
        parsed.payload.schema, scanned.payload.schema,
        "one schema: what they share is the shape of an answer, which is the interface"
    );
    assert_ne!(
        parsed.payload.bytes, scanned.payload.bytes,
        "and different bytes, or there would be no reason for a caller to care which \
         provider answered"
    );
    assert_ne!(
        parsed.evidence, scanned.evidence,
        "a parse is verified and a pattern match is approximate, and the fact says which"
    );
    eprintln!(
        "alpha/one.rs — parsed: {:?}\n              scanned: {:?}",
        String::from_utf8_lossy(&parsed.payload.bytes),
        String::from_utf8_lossy(&scanned.payload.bytes)
    );
}

/// Coverage bought at scale, and what it cost.
#[test]
fn Test_The_Weaker_Provider_Should_Answer_For_The_Whole_Scale_Corpus()
{
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

/// Two providers, one contract, and neither of them wrote it.
///
/// The composition is where this is observable at all. `nomos-cap-syntax` cannot name either
/// provider — it sits below both, deliberately, so that neither party can change the terms
/// the other is bound by. Each provider names the contract and not its peer. So the only
/// place all three are visible at once is here, which is the same reason the rest of this
/// file exists.
///
/// The ceiling is the sharp end. It bounds what *either* provider may claim, and while it
/// lived in `nomos-lang-rust` that crate could have raised or lowered what its peer was
/// permitted to promise, in a file the peer could not open.
#[test]
fn Test_Both_Providers_Should_Offer_Against_A_Contract_Neither_Declares()
{
    let registry = Registered();
    let capability = CapabilityId::New(syntax::CAPABILITY);
    let contract = registry
        .Declared()
        .find(|declared| return declared.id == capability)
        .expect("the syntax capability is declared");

    assert_eq!(
        contract.ceiling,
        syntax::Ceiling(),
        "the terms in the registry are the contract crate's, not a provider's"
    );
    assert_eq!(
        registry
            .Offers(&capability)
            .iter()
            .map(|offer| return offer.provider.As_Str())
            .collect::<Vec<&str>>(),
        vec![scan::PROVIDER, rust::PROVIDER],
        "both providers offer against the one contract"
    );
    // The ceiling leaves room neither provider occupies. Without this the assertion above
    // would pass over a ceiling that is merely the incumbent's guarantee restated — which
    // is a ceiling that has to be raised whenever somebody improves something, and one that
    // silently forbids a better second provider.
    for offer in registry.Offers(&capability)
    {
        assert_ne!(
            offer.guarantee,
            syntax::Ceiling(),
            "{} claims exactly the ceiling, so the ceiling is describing an implementation \
             rather than bounding the capability",
            offer.provider
        );
    }
}

/// What the registry does when more than one offer clears the floor.
///
/// # The rule, over the composition it was decided for
///
/// The strongest usable offer answers, and every offer it was chosen over comes back with
/// it. `nomos.lang.rust.scan` still sorts first by name and no longer wins by it, which is
/// the whole of what changed: selection stopped being a consequence of spelling.
///
/// This test used to assert the opposite — `..._Should_Resolve_By_Name_Order_Until_
/// Something_Says_Otherwise`, whose name recorded that the behaviour was observed rather
/// than intended. `docs/records/OD-CAPABILITY-001` records what decided it.
#[test]
fn Test_The_Strongest_Usable_Offer_Should_Answer_Whatever_The_Providers_Are_Called()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection,
        applicability: how,
    } = Slice::Over(&corpus).Accepting(Approximate_Floor()).Resolved();

    assert_eq!(how, Applicability::Supported);
    Answered_Despite_Its_Name(&selection);
}

/// The parser answers because it is stronger, not because of how it is spelled.
///
/// The name order really is against it here — `nomos.lang.rust.scan` sorts first — which is
/// what makes this a statement about names rather than a coincidence, and the guarantee
/// ranked the two, so the choice is a decision rather than a tiebreak.
fn Answered_Despite_Its_Name(selection: &nomos_capability::Selection)
{
    let passed_over = selection
        .alternatives
        .first()
        .expect("the scanner clears this floor too")
        .provider
        .clone();

    assert_eq!(
        selection.chosen.provider.As_Str(),
        rust::PROVIDER,
        "both offers clear this floor and the parser is strictly stronger, so it answers \
         — despite the scanner's name sorting first"
    );
    assert!(
        selection.chosen.provider.As_Str() > passed_over.As_Str(),
        "and the name order really is against it here, or this test proves nothing about \
         names: {} against {passed_over}",
        selection.chosen.provider
    );
    assert!(
        !selection.Passed_Over_Stronger(),
        "nothing usable was stronger than what answered, which is the rule"
    );
    assert!(
        selection.Unranked().is_empty(),
        "and the guarantee ranked them, so this composition's provider choice is a \
         decision rather than a tiebreak: {:?}",
        selection.Unranked()
    );
}

/// What the lowered floor bought, and that it did not cost the files it was not for.
///
/// The situation that raised OD-CAPABILITY-001: a caller lowers its floor to `Approximate`
/// because a parser refuses seven files in the scale corpus, and wants an answer for those
/// seven without giving up the exact answer for the other 7,573.
///
/// Both of the reflex rules fail it. Name order serves the scanner for every file, so the
/// coverage costs precision everywhere. Strongest-wins alone serves the parser for every
/// file, so lowering the floor buys *nothing* — the same seven files go unanswered and the
/// caller cannot tell its requirement changed anything. The selection is what makes the
/// floor mean something: the parser answers, and the offer the floor admitted is reachable
/// for the subjects the parser cannot serve.
///
/// The registry cannot spend it, because a requirement names a capability and not a
/// subject, and which files a parser will refuse is not knowable until it reads them.
/// Spending it per subject is `Slice::Dispatch`'s to do and is not done here.
#[test]
fn Test_A_Lowered_Floor_Should_Make_The_Weaker_Offer_Reachable_Without_Serving_It()
{
    let corpus = Precision_Corpus();
    let parsed = Slice::Over(&corpus).Resolved().selection;
    let lowered = Slice::Over(&corpus)
        .Accepting(Approximate_Floor())
        .Resolved()
        .selection;

    assert_eq!(
        parsed.chosen, lowered.chosen,
        "lowering the floor must not change who answers; it widens what is admitted, and \
         the strongest thing admitted did not change"
    );
    assert!(
        parsed.Weaker().is_empty(),
        "the strict floor admits the scanner nowhere, so there is nothing to fall back to"
    );
    assert_eq!(
        lowered
            .Weaker()
            .iter()
            .map(|offer| return offer.provider.As_Str().to_owned())
            .collect::<Vec<_>>(),
        vec![scan::PROVIDER.to_owned()],
        "and the lowered one admits exactly the scanner, which is what the caller widened \
         its requirement to reach"
    );
}

// ---------------------------------------------------------------------------------
// Spending the selection, per subject
// ---------------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------------
// The three prototype lessons
// ---------------------------------------------------------------------------------

/// A signal that fires on everything is not a signal.
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


/// A hole in the corpus stays visible.
///
/// `gamma/broken.rs` cannot be read, so `gamma`'s rollup covers one of its two files. It
/// reports that rather than reporting a smaller directory, because a rollup that silently
/// omits what it could not read is indistinguishable from one whose inputs were all fine.
#[test]
fn Test_A_Rollup_Over_A_Refused_Member_Should_Report_A_Degraded_Answer()
{
    let (corpus, mut slice) = Over_The_Precision_Corpus();
    let report = slice.Run(&corpus);
    assert_eq!(
        report.refused.len(),
        1,
        "one file in the precision corpus does not parse: {:?}",
        report.refused
    );
    assert_eq!(report.degraded, vec!["gamma"]);

    let surface = Surface_Of(&slice, &corpus, "gamma");
    assert_eq!(surface.files, 1, "one of gamma's two files was readable");
    assert_eq!(
        surface.unreachable, 1,
        "and the other is counted, not dropped. A rollup reporting files=1 with \
         unreachable=0 would be claiming gamma has one file"
    );
    // The undegraded groups, as the control. If every rollup reported unreachable members
    // the assertion above would pass over a slice that could read nothing.
    for group in ["alpha", "beta"]
    {
        let whole = Surface_Of(&slice, &corpus, group);
        assert_eq!(whole.unreachable, 0, "{group} has no unreadable members");
        assert_eq!(whole.files, 2, "{group} has two files");
    }
}

/// The precision corpus's shape, asserted so its README cannot drift from it.
#[test]
fn Test_The_Precision_Corpus_Should_Have_The_Shape_Its_Readme_Claims()
{
    let (corpus, mut slice) = Over_The_Precision_Corpus();
    slice.Run(&corpus);

    for (group, files, items, public) in [("alpha", 2, 7, 3), ("beta", 2, 7, 3), ("gamma", 1, 3, 0)]
    {
        let surface = Surface_Of(&slice, &corpus, group);
        assert_eq!(
            (surface.files, surface.items, surface.public),
            (files, items, public),
            "{group} does not match the table in tests/corpus/analysis/README.md"
        );
    }
}

/// Resolution is a value, and a caller that needs more than any provider offers is told so
/// rather than served something weaker.
///
/// The composition's honesty check: the registry holds two providers, and a requirement no
/// offer reaches resolves to `MissingCapability` — coverage debt — rather than to
/// `NotApplicable`, which would be a statement about the subject that only a rule may make.
#[test]
fn Test_An_Unmeetable_Requirement_Should_Report_Coverage_Debt()
{
    let slice = Slice::Composed();
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Symbol,
    );
    let needs_resolution = Requirement::New(
        CapabilityId::New(syntax::CAPABILITY),
        syntax::CONTRACT_VERSION,
        guarantee,
    );

    let resolved = slice.Registry().Resolve(&needs_resolution);
    assert!(resolved.Offer().is_none());
    assert_eq!(
        resolved.Applicability(),
        Applicability::MissingCapability,
        "nothing offers this, which is coverage debt. NotApplicable would say the subject \
         does not bind the rule, and the registry is in no position to say that"
    );
    // The positive control. If resolution refused everything the assertion above would
    // pass over a composition that serves nobody.
    let servable = Requirement::New(
        CapabilityId::New(syntax::CAPABILITY),
        syntax::CONTRACT_VERSION,
        rust::Declared_Guarantee(),
    );
    assert!(slice.Registry().Resolve(&servable).Offer().is_some());
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
