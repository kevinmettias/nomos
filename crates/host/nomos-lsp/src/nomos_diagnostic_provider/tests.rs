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
