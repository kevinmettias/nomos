//! The seam between `nomos_workspace` and `nomos_model`, driven through `nomos_workspace`'s
//! own public API — the same view a real consumer has.
//!
//! `Workspace::Content_Of` answers with exactly the `nomos_model::Content_Digest` of the
//! content a change set wrote. The inline `#[cfg(test)] mod local_tests` in
//! `src/workspace.rs` proves this only through this crate's own private access; this file
//! proves the same property through the public surface a real consumer has.

use nomos_contracts::{ConfigurationId, Digest128};
use nomos_model::Content_Digest;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

fn Fresh_Workspace() -> Workspace
{
    return Workspace::Empty(
        BuildVariant::New("x86_64-pc-windows-msvc", "dev", "1.85", ["analysis"]),
        ConfigurationId::From_Digest(Digest128::From_Bytes([0x42; Digest128::BYTE_LENGTH])),
    );
}

/// The happy path: content written through the one door reads back at exactly
/// `nomos_model::Content_Digest` of the bytes submitted.
#[test]
fn Test_Content_Of_Should_Answer_With_Nomos_Models_Own_Content_Digest()
{
    let mut workspace = Fresh_Workspace();
    let changes = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("src/a.rs", "fn a() {}");

    workspace.Apply(&changes).expect("a workspace-relative path applies");

    assert_eq!(
        workspace.Content_Of("SRC/A.RS"),
        Some(Content_Digest("fn a() {}".as_bytes())),
        "lookup normalizes the path and answers with nomos_model's own digest"
    );
}

/// The negative control: different content must not share a digest, or the equality above
/// would hold for any two files.
#[test]
fn Test_Different_Content_Should_Not_Share_Nomos_Models_Digest()
{
    let mut workspace = Fresh_Workspace();
    let changes = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("src/a.rs", "fn a() {}");
    workspace.Apply(&changes).expect("applies");

    assert_ne!(
        workspace.Content_Of("src/a.rs"),
        Some(Content_Digest("fn different() {}".as_bytes()))
    );
}
