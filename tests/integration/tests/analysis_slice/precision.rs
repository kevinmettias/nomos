//! Touching one file recomputes exactly its descendants, named.
//!
//! The precision property, and the reason a second corpus exists. Every assertion here names
//! the facts it expects rather than counting them, because a count is satisfied by
//! recomputing the wrong things — and it is stated as set equality, so recomputing too
//! little fails as loudly as too much.

use crate::corpus::{Broadenings, Over_The_Precision_Corpus, Reached, Rewrite, Rewritten};
use nomos_integration_tests::{
    Corpus, Decode_Surface, Name_Keys, RunReport, Slice, SURFACE_CAPABILITY
};
use nomos_workspace::ChangeSource;

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
    use nomos_contracts::IncrementalGranularity;

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
