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

use nomos_capability::Requirement;
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, FactVariant, Guarantee, IncrementalGranularity,
};
use nomos_integration_tests::{
    Corpus, Decode_Surface, Edited, Host_Variant, Resolved_Configuration, Slice, Walk,
    SURFACE_CAPABILITY,
};
use nomos_lang_rust as rust;
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
        first
            .syntax_materialized
            .saturating_add(first.refused.len()),
        first.files_seen,
        "every file must reach exactly one outcome"
    );
    assert!(
        first.surface_materialized > 0,
        "{} groups produced no rollups; the derived layer never ran",
        first.groups_seen
    );

    // The workspace read the same corpus the providers did. A member count that disagreed
    // with the file count would mean the checkout and the walk saw different trees, and
    // every fact would be keyed on a workspace state that does not describe what was
    // parsed.
    assert_eq!(
        slice.Workspace().Snapshot().Len(),
        first.files_seen,
        "the ingested workspace and the walked corpus must be the same tree"
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

    // The positive control. Both figures above are also zero for a run that did nothing
    // at all, so the reuse counts have to show that the work was recognized rather than
    // skipped.
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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    let first = slice.Run(&corpus);
    assert_eq!(first.syntax_materialized, 5, "five of six files parse");
    assert_eq!(first.surface_materialized, 3, "three groups");

    let before = slice.Generation();
    let edited = slice.Edit(
        &mut corpus,
        ChangeSource::IdeEdit,
        "alpha/one.rs",
        "//! Rewritten.\n\npub fn Added() {}\n",
    );

    let Edited::Advanced { invalidated, .. } = edited
    else
    {
        panic!("rewriting a file is a change to the workspace: {edited:?}")
    };
    assert!(
        slice.Generation() > before,
        "the generation facts materialize into is the one the workspace produced"
    );

    assert_eq!(
        Slice::Name_Keys(&corpus, &invalidated.direct),
        vec!["nomos.cap.syntax.items of alpha/one.rs"],
        "exactly the changed file's own fact is directly invalidated"
    );
    assert_eq!(
        Slice::Name_Keys(&corpus, &invalidated.dependent),
        vec!["nomos.cap.module.surface of alpha"],
        "exactly the rollup that read it follows, along an edge the Reader recorded"
    );

    let second = slice.Run(&corpus);

    assert_eq!(
        second.Recomputed(),
        vec![
            "nomos.cap.module.surface of alpha",
            "nomos.cap.syntax.items of alpha/one.rs",
        ],
        "one file changed, so one syntax fact and one rollup recompute — and nothing in \
         beta or gamma, which nothing connects to alpha"
    );

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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    slice.Run(&corpus);

    let edited = slice.Edit(
        &mut corpus,
        ChangeSource::AgentEdit,
        "gamma/five.rs",
        "//! Rewritten.\n\nstruct Other;\n",
    );

    let Edited::Advanced { invalidated, .. } = edited
    else
    {
        panic!("rewriting a file is a change to the workspace: {edited:?}")
    };
    let reached = Slice::Name_Keys(&corpus, &invalidated.dependent);

    assert_eq!(reached, vec!["nomos.cap.module.surface of gamma"]);
    assert!(
        !reached.iter().any(|name| return name.ends_with("alpha") || name.ends_with("beta")),
        "a change in gamma reached {reached:?}"
    );

    let second = slice.Run(&corpus);

    assert_eq!(
        second.Recomputed_For(SURFACE_CAPABILITY),
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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    let first = slice.Run(&corpus);
    assert_eq!(first.surface_materialized, 3, "three groups");

    let workspace = slice.Workspace().Id();

    // Deliberately not through `Slice::Edit`. The corpus on disk is now something the
    // workspace has never been told about, which is what a checkout behind a running
    // process, or a store reopened over a tree that moved, actually looks like.
    assert!(corpus.Rewrite("beta/four.rs", "//! Rewritten.\n\npub fn Now_Public() {}\n"));

    assert_eq!(
        slice.Workspace().Id(),
        workspace,
        "nothing announced the change, so the workspace must not have moved — this test is \
         about identity, and a workspace that somehow knew would explain the recomputation \
         a second way"
    );

    let second = slice.Run(&corpus);

    assert_eq!(
        second.Recomputed(),
        vec![
            "nomos.cap.module.surface of beta",
            "nomos.cap.syntax.items of beta/four.rs",
        ],
        "a changed member makes a different rollup, with no invalidation involved"
    );

    let members = corpus.In_Group("beta");
    let fact = slice.Surface_Of(&members).expect("beta has a rollup");
    let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

    assert_eq!(
        (surface.items, surface.public),
        (6, 4),
        "and the recomputed rollup reflects the new contents rather than reusing the old \
         number under a key that no longer describes it"
    );
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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    slice.Run(&corpus);

    let edited = slice.Edit(
        &mut corpus,
        ChangeSource::IdeEdit,
        "alpha/two.rs",
        "//! Rewritten.\n\nfn Changed() {}\n",
    );

    let Edited::Advanced { invalidated, .. } = edited
    else
    {
        panic!("rewriting a file is a change to the workspace: {edited:?}")
    };

    let broadened: Vec<String> = invalidated
        .broadened
        .iter()
        .map(|record| {
            return format!(
                "{} {:?} -> {:?}",
                Slice::Name_Keys(&corpus, core::slice::from_ref(&record.key))
                    .first()
                    .cloned()
                    .unwrap_or_default(),
                record.requested,
                record.applied
            );
        })
        .collect();

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
    let corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);
    let snapshot = slice.Workspace().Snapshot().clone();

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
    let fact = slice
        .Surface_Of(&corpus.In_Group("alpha"))
        .expect("alpha has a rollup");

    assert_eq!(
        fact.snapshot,
        slice.Workspace().Id(),
        "a fact names the tree it was read from"
    );
    assert_eq!(
        slice.Snapshot(),
        slice.Workspace().Id(),
        "and the held value still agrees after a run"
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

    let key_of = |slice: &Slice, from: &Corpus| {
        let file = from
            .files
            .iter()
            .find(|file| return file.path == "alpha/one.rs")
            .expect("the precision corpus contains alpha/one.rs");

        return slice.Syntax_Key(file);
    };

    let left = key_of(&one, &corpus);
    let right = key_of(&two, &other);

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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);
    slice.Run(&corpus);

    let before = slice.Workspace().Id();
    let replaced = slice.Checkout(
        &mut corpus,
        &[
            ("alpha/one.rs", "//! Checked out.\n\npub fn Landed() {}\n"),
            ("gamma/five.rs", "//! Checked out.\n\nstruct Also;\n"),
        ],
    );

    let Edited::Advanced { invalidated, .. } = replaced
    else
    {
        panic!("a checkout that landed two files changed the workspace: {replaced:?}")
    };
    assert_ne!(slice.Workspace().Id(), before, "the workspace is another state");
    assert_eq!(
        slice.Snapshot(),
        slice.Workspace().Id(),
        "and the slice followed it. A held snapshot that stops following the workspace is \
         the pin this item removed, wearing a cache"
    );

    assert_eq!(
        Slice::Name_Keys(&corpus, &invalidated.direct),
        vec![
            "nomos.cap.syntax.items of alpha/one.rs",
            "nomos.cap.syntax.items of gamma/five.rs",
        ],
        "exactly the two members that differ"
    );
    assert_eq!(
        Slice::Name_Keys(&corpus, &invalidated.dependent),
        vec![
            "nomos.cap.module.surface of alpha",
            "nomos.cap.module.surface of gamma",
        ],
        "and the rollups that read them, along edges the Reader recorded"
    );
    assert_eq!(
        invalidated.cause.Granularity(),
        IncrementalGranularity::File,
        "a replacement that names its members is a statement about files"
    );

    let second = slice.Run(&corpus);

    assert_eq!(
        second.syntax_reused, 3,
        "beta's two files and alpha/two.rs survived a checkout that did not touch them"
    );
    assert_eq!(second.surface_reused, 1, "and beta's rollup with them");
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
    let mut corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    let first = slice.Run(&corpus);
    let before = (slice.Generation(), slice.Workspace().Id());

    let unchanged = corpus
        .files
        .iter()
        .find(|file| return file.path == "alpha/one.rs")
        .map(|file| return file.source.clone())
        .expect("the precision corpus contains alpha/one.rs");

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

    let second = slice.Run(&corpus);

    assert_eq!(
        second.Recomputed(),
        Vec::<String>::new(),
        "nothing changed, so nothing recomputes"
    );
    assert_eq!(
        second.syntax_reused, first.syntax_materialized,
        "and every fact is still there to be reused, which is what makes the line above an \
         assertion about reuse rather than about a store that lost everything"
    );

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
    let banned = ["Digest128", "From_Bytes"].join("::");
    let mut scanned = BTreeSet::new();
    let mut offending = Vec::new();
    let mut derived = 0_usize;

    for directory in ["src", "tests"]
    {
        for file in Rust_Files(&root.join(directory))
        {
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("{} is this crate's own source: {error}", file.display()));
            let named = file.strip_prefix(&root).unwrap_or(&file).display().to_string();
            scanned.insert(named.clone());

            if text.contains(&banned)
            {
                offending.push(named);
            }
            if text.contains("Content_Digest") || text.contains("Digest_Of_Parts")
            {
                derived = derived.saturating_add(1);
            }
        }
    }

    // The guard against a vacuous pass. A scan that found no files reports a clean result
    // over nothing, which is the defect `Test_The_Workspace_Should_Not_Appear_Empty` exists
    // for one crate up.
    assert!(
        scanned.len() >= 5,
        "scanned {} files under {}, which is not this crate: {scanned:?}",
        scanned.len(),
        root.display()
    );
    assert!(
        derived > 0,
        "not one scanned file mentions a content digest, so the reader is not reading Rust"
    );

    assert!(
        offending.is_empty(),
        "these files build an identity out of literal bytes: {offending:?}.\n\
         An invented identity component cannot be wrong, so it can never be observed to be \
         wrong — two machines, two toolchains and two policies all agree under it. Every \
         identity here comes from a workspace, a digest of content, or a resolved \
         composition."
    );
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
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| return extension == "rs")
            {
                found.push(path);
            }
        }
    }

    return found;
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

    let mut groups = 0_usize;
    let mut silent = 0_usize;

    for group in corpus.Groups()
    {
        let members = corpus.In_Group(&group);
        let Some(fact) = slice.Surface_Of(&members)
        else
        {
            continue;
        };
        let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

        groups = groups.saturating_add(1);
        if surface.public == 0
        {
            silent = silent.saturating_add(1);
        }
    }

    let percent = silent.saturating_mul(100).checked_div(groups).unwrap_or(0);
    eprintln!(
        "signal: {silent} of {groups} directories declare nothing publicly ({percent}%)"
    );

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
    let corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);

    let report = slice.Run(&corpus);

    assert_eq!(
        report.refused.len(),
        1,
        "one file in the precision corpus does not parse: {:?}",
        report.refused
    );
    assert_eq!(report.degraded, vec!["gamma"]);

    let members = corpus.In_Group("gamma");
    let fact = slice
        .Surface_Of(&members)
        .expect("gamma has a rollup even though one member is unreadable");
    let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

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
        let members = corpus.In_Group(group);
        let fact = slice.Surface_Of(&members).expect("a rollup per group");
        let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

        assert_eq!(surface.unreachable, 0, "{group} has no unreadable members");
        assert_eq!(surface.files, 2, "{group} has two files");
    }
}

/// The precision corpus's shape, asserted so its README cannot drift from it.
#[test]
fn Test_The_Precision_Corpus_Should_Have_The_Shape_Its_Readme_Claims()
{
    let corpus = Precision_Corpus();
    let mut slice = Slice::Over(&corpus);
    slice.Run(&corpus);

    for (group, files, items, public) in [("alpha", 2, 7, 3), ("beta", 2, 7, 3), ("gamma", 1, 3, 0)]
    {
        let members = corpus.In_Group(group);
        let fact = slice.Surface_Of(&members).expect("a rollup per group");
        let surface = Decode_Surface(&fact.payload.bytes).expect("the rollup wrote this");

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

    let needs_resolution = Requirement::New(
        CapabilityId::New(rust::CAPABILITY),
        rust::CONTRACT_VERSION,
        Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Symbol,
        ),
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
        CapabilityId::New(rust::CAPABILITY),
        rust::CONTRACT_VERSION,
        rust::Declared_Guarantee(),
    );
    assert!(slice.Registry().Resolve(&servable).Offer().is_some());
}
