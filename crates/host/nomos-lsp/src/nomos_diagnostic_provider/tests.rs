//! What this crate still decides, now that the protocol is the engine's.
//!
//! # Why the loop is no longer asserted here
//!
//! It is not this crate's any more. That a judged file is published, that a file which went
//! clean is published empty so its markers clear, that a file nothing ever published is not
//! cleared for no reason, that a one-based line becomes a zero-based span -- all of it moved
//! down with the server and is asserted in `xvpe-language-server-backend-lsp`'s own suite.
//!
//! What remains is the half that is genuinely this workspace's, and it is the half that was
//! never about the protocol in the first place: that the workspace and the fact store are
//! *reused* across two judgements rather than rebuilt, which is what `OD-ANALYSIS-009`'s
//! second amendment named this crate as the case for.

use super::*;

/// A fresh, empty directory under the system temporary directory.
fn Root(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates a fresh directory");

    return root;
}

#[test]
fn Test_Judging_Twice_Should_Reuse_The_Same_Workspace_And_Advance_Its_Generation_On_A_Real_Edit()
{
    let root = Root("nomos-lsp-provider-reuse-edited-root");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");

    let mut provider = NomosDiagnosticProvider::New();

    let _ignored = provider.Diagnose(&root);
    let generation_after_first =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Also_Ok() {}\n").expect("writable");
    let _ignored = provider.Diagnose(&root);
    let generation_after_second =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(
        generation_after_second > generation_after_first,
        "a real edit reusing the same workspace must advance its generation, not repeat it: \
         {generation_after_first:?} then {generation_after_second:?}"
    );
}

#[test]
fn Test_Judging_An_Untouched_Tree_Twice_Should_Not_Advance_The_Generation_A_Second_Time()
{
    let root = Root("nomos-lsp-provider-reuse-untouched-root");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");

    let mut provider = NomosDiagnosticProvider::New();

    let _ignored = provider.Diagnose(&root);
    let generation_after_first =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = provider.Diagnose(&root);
    let generation_after_second =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(
        generation_after_second, generation_after_first,
        "an untouched file must not advance the generation a second time: \
         {generation_after_first:?} then {generation_after_second:?}"
    );
}

#[test]
fn Test_A_Root_That_Cannot_Be_Walked_Should_Report_Nothing()
{
    let mut provider = NomosDiagnosticProvider::New();

    let judged = provider.Diagnose(std::path::Path::new("a-directory-that-does-not-exist"));

    assert!(judged.is_empty(), "a root that is not a directory yields no judgement");
}

/// Two sources, three judgements, and one file edited between the second and the third.
///
/// # Why three and not two
///
/// A second unchanged judgement answering what the first answered is consistent with a
/// provider that reused everything and with one that recomputed everything, which is the
/// same vacuity `tests/incremental_equivalence.rs` states about its own comparisons. The
/// third judgement is what separates them, because only a provider that really kept the
/// store can charge for one source and not the other.
///
/// # Why the assertion is arithmetic over deltas rather than three pinned numbers
///
/// [`nomos_analysis::MemoryFactStore::Materializations`] counts every store write, and most
/// of them are not the syntax family: the dependency, lint and policy providers run their
/// own subprocess against `root` and write again on every call, so an unchanged judgement
/// here costs something rather than nothing. That per-call cost is the second delta, and
/// subtracting it is what leaves the part this test is actually about. Pinning the three
/// totals instead would make this test fail the day a family is added, for a reason that
/// has nothing to do with reuse.
#[test]
fn Test_A_Second_Judgement_Should_Re_Derive_A_Fact_Only_For_The_Source_That_Moved()
{
    let root = Root("nomos-lsp-provider-selective-invalidation");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("writable");

    let mut provider = NomosDiagnosticProvider::New();

    let cold_judgement = provider.Diagnose(&root);
    let cold = provider.store.Materializations();

    let _ignored = provider.Diagnose(&root);
    let unchanged = provider.store.Materializations().saturating_sub(cold);

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Added() {}\n").expect("writable");
    let _ignored = provider.Diagnose(&root);
    let edited = provider.store.Materializations().saturating_sub(cold).saturating_sub(unchanged);

    let _ignored = std::fs::remove_dir_all(&root);

    assert!(
        !cold_judgement.is_empty(),
        "the cold judgement found nothing, so every count below is about a tree that was \
         never really judged"
    );
    assert!(
        unchanged < cold,
        "an unchanged second judgement cost {unchanged} against a cold {cold}: reuse that \
         charges the same as a cold run is a cache nothing is reading"
    );
    assert_eq!(
        cold.saturating_sub(unchanged),
        2,
        "the cold judgement should have paid for both sources' syntax on top of the \
         per-call cost of {unchanged}, and paid for {} instead",
        cold.saturating_sub(unchanged)
    );
    assert_eq!(
        edited.saturating_sub(unchanged),
        1,
        "editing one of two sources should re-derive one syntax fact and reuse the other's, \
         and re-derived {} on top of the per-call cost of {unchanged}",
        edited.saturating_sub(unchanged)
    );
}

/// What a provider that kept its state answers is what a provider that never had any
/// answers about the same tree.
///
/// The equivalence `tests/incremental_equivalence.rs` proves over the seam, asserted here
/// over the surface an editor actually calls -- and it is the one falsifier this boundary
/// has for the reassessment cache itself. A cache that wrongly reused a rule whose family
/// moved would still return diagnostics, and they would be the ones from before the edit;
/// the fresh provider is what says they are not.
#[test]
fn Test_A_Reusing_Provider_Should_Answer_What_A_Fresh_One_Answers_Over_The_Same_Tree()
{
    let root = Root("nomos-lsp-provider-clean-recompute-equivalence");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("writable");

    let mut reusing = NomosDiagnosticProvider::New();

    let cold = reusing.Diagnose(&root);
    let unchanged = reusing.Diagnose(&root);

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Also_Ok() {}\n").expect("writable");
    let after_edit = reusing.Diagnose(&root);

    let recomputed = NomosDiagnosticProvider::New().Diagnose(&root);

    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(unchanged, cold, "an unchanged tree judged twice must answer the same thing twice");
    assert_ne!(
        after_edit, cold,
        "the edit must change what the provider answers, or this fixture proves nothing \
         about invalidation -- reusing everything and recomputing everything both pass an \
         unchanged comparison"
    );
    assert_eq!(
        after_edit, recomputed,
        "a provider that carried its workspace, store and reassessment cache across the \
         edit answered differently from one that had never judged this tree at all"
    );
}

/// A cold judgement is what the unreassessed path produces, argument for argument.
///
/// [`nomos_check_orchestration::Run`] is the seam `nomos-cli::check` calls, and
/// `Run_Reassessing` is that same pipeline handed a cache. Asserting the two agree on a
/// cold tree is what says this crate changed which entry point it calls and nothing else --
/// not the selection it passes, not the variant, and not the launcher. A wiring mistake
/// there would otherwise show up only as diagnostics an editor renders and no test reads.
#[test]
fn Test_A_Cold_Judgement_Should_Equal_What_The_Unreassessed_Run_Path_Produces()
{
    let root = Root("nomos-lsp-provider-cold-run-equivalence");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("writable");

    let judged = NomosDiagnosticProvider::New().Diagnose(&root);
    let expected = Judged_By_Run(&root);

    let _ignored = std::fs::remove_dir_all(&root);

    assert!(
        !expected.is_empty(),
        "the unreassessed path found nothing over this tree, so the comparison below would \
         hold over two empty lists"
    );
    assert_eq!(
        judged, expected,
        "a cold judgement must be what the unreassessed Run path produces over the same tree"
    );
}

/// `root` judged through [`nomos_check_orchestration::Run`], rendered the way
/// [`NomosDiagnosticProvider::Diagnose`] renders its own.
///
/// Spelled out here rather than shared with the provider: a helper both called would
/// compare each against itself, and what is being asserted is that two entry points agree.
fn Judged_By_Run(root: &std::path::Path) -> Vec<SourceDiagnostic>
{
    let sources = Walked_Sources(root).expect("a directory that was just written walks");
    let mut workspace = None;
    let mut store = MemoryFactStore::New();

    let outcome = nomos_check_orchestration::Run(
        &sources,
        nomos_check_orchestration::RunContext {
            variant: Host_Variant(),
            root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut workspace,
            store: &mut store,
        },
        &[],
    );

    let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a walked tree with two readable sources must reach a judgement");
    };

    let architecture = nomos_repo_policy::architecture::Discover_Workspace(root, &FILE_SYSTEM).unwrap_or_default();

    return findings.iter().flat_map(|finding| return Diagnostics_For(&architecture, finding)).collect();
}
