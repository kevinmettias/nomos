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

/// The byte this fixture's configuration digest repeats across its length. Which byte it is
/// carries no meaning; that it is fixed is what makes two runs comparable.
const FIXTURE_CONFIGURATION_BYTE: u8 = 0x42;

// The variant is built into a local rather than inlined into `Workspace::Empty`, so that the
// value handed to the constructor has a name.
fn Fresh_Workspace() -> Workspace
{
    let variant = BuildVariant::New("x86_64-pc-windows-msvc", "dev", "1.85", ["analysis"]);
    let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([
        FIXTURE_CONFIGURATION_BYTE;
        Digest128::BYTE_LENGTH
    ]));

    return Workspace::Empty(variant, configuration);
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
    workspace
        .Apply(&changes)
        .expect("src/a.rs is workspace-relative, which is the only thing the door refuses");

    assert_ne!(
        workspace.Content_Of("src/a.rs"),
        Some(Content_Digest("fn different() {}".as_bytes()))
    );
}
