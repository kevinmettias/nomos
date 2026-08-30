//! The context is derived, not invented.
//!
//! Every component of a fact's identity comes from something real. The slice used to supply
//! three byte-fill constants, and an invented identity component cannot be wrong — which is
//! exactly why it is dangerous: two machines, two toolchains and two policies all agree
//! under it, and the disagreement they should have had is the one the fact key exists to
//! detect.

use crate::corpus::{
    Advanced, Alpha_One, Over_The_Precision_Corpus, Precision_Corpus, Reached, Source_Of,
};
use nomos_analysis::InvalidationReport;
use nomos_integration_tests::{
    Corpus, Edited, Host_Variant, Resolved_Configuration, RunReport, Slice
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

    assert_eq!(snapshot.Length(), corpus.files.len(), "one member per file");
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
    use nomos_contracts::IncrementalGranularity;

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

    Assert_Rewriting_A_File_With_Its_Own_Contents_Moves_Nothing(&mut slice, &mut corpus, &first);
    // The positive control. If `Edit` reported `Unchanged` for everything, the assertions
    // above would pass over a door that cannot register a change at all.
    Assert_An_Actual_Edit_Still_Registers_As_A_Change(&mut slice, &mut corpus);
}

/// An editor's save hook does not consult the previous generation before writing, so this
/// arrives constantly — and invalidating on it would discard every fact reachable from the
/// file in order to recompute the answers the store already held.
fn Assert_Rewriting_A_File_With_Its_Own_Contents_Moves_Nothing(
    slice: &mut Slice,
    corpus: &mut Corpus,
    first: &RunReport,
)
{
    use nomos_workspace::ChangeSource;

    let before = (slice.Generation(), slice.Workspace().Id());

    let unchanged = Source_Of(corpus, "alpha/one.rs");
    let edited = slice.Edit(corpus, ChangeSource::IdeEdit, "alpha/one.rs", &unchanged);
    assert!(
        matches!(edited, Edited::Unchanged { .. }),
        "an editor rewriting a file with its own contents changed nothing: {edited:?}"
    );
    assert_eq!(
        (slice.Generation(), slice.Workspace().Id()),
        before,
        "and neither the generation nor the workspace may move for it"
    );
    Assert_Nothing_Recomputed_And_Nothing_Lost(&slice.Run(corpus), first);
}

/// The positive control: a real edit still advances the generation and reports as such.
fn Assert_An_Actual_Edit_Still_Registers_As_A_Change(slice: &mut Slice, corpus: &mut Corpus)
{
    use nomos_workspace::ChangeSource;

    let changed = slice.Edit(
        corpus,
        ChangeSource::IdeEdit,
        "alpha/one.rs",
        "//! Actually different.\n\npub fn Moved() {}\n",
    );
    assert!(matches!(changed, Edited::Advanced { .. }), "{changed:?}");
}

/// Nothing changed, so nothing recomputes — and every fact is still there to be reused,
/// which is what makes the first half an assertion about reuse rather than about a store
/// that lost everything.
fn Assert_Nothing_Recomputed_And_Nothing_Lost(second: &RunReport, first: &RunReport)
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
        // A file skipped on a read error leaves both counters short: it never enters
        // `found.read`, and it can never enter `offending`. The ban would then be reported as
        // holding over a file nobody opened, which is the scanner passing by not looking.
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
