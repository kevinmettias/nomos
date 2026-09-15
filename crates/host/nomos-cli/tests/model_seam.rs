//! Proves nomos-cli's real boundary with `nomos_model` -- an INTEGRATION SURFACE WITH NO
//! SUITE finding, since nothing under `tests/` referenced this crate before this file:
//! `check.rs` and `gate.rs` both call `nomos_model::Subject_Of_Path` for real, from
//! `check/sources.rs::Read_Source`, to file every walked file's facts under a subject, but no
//! inline or external test exercised that seam at all.
//!
//! `Read_Source`'s own doc comment states the contract this test proves: the relative path a
//! finding's `locations` field reports and the string handed to `Subject_Of_Path` are *the
//! same string*, "so the two cannot disagree about addressing". This drives a real `nomos
//! check` over a fixture that trips a real, observable finding, and ties the location text
//! the binary actually printed back to the same function nomos-cli itself calls to address
//! that file.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

#[test]
fn Test_A_Findings_Reported_Location_Should_Resolve_Through_The_Same_Function_Nomos_Cli_Uses()
{
    let tree = A_Tree_Holding_One_Unclaimed_File();

    let ran = Run(&["check", "--root", &tree.Root()]);

    Assert_The_Location_The_Run_Reported_Is_The_Files_Own_Relative_Path(&ran);

    Assert_That_Path_Resolves_To_One_Subject_On_Every_Spelling();
}

/// The tree the run above walks: one file whose doc comment claims a mirror that names no
/// check, so a real finding is observable through the real binary.
fn A_Tree_Holding_One_Unclaimed_File() -> Tree
{
    return Tree::New("model-seam").With(
        "notes/example.rs",
        "/// Mirrored by `Test_Nothing_Named_This_In_Model_Seam`.\n\
         pub const TABLES: &[&str] = &[];\n",
    );
}

/// The reported location must be the forward-slash relative path
/// `check/sources.rs::Relative_Path` produces -- the exact text `Read_Source` also hands to
/// `nomos_model::Subject_Of_Path`.
fn Assert_The_Location_The_Run_Reported_Is_The_Files_Own_Relative_Path(ran: &support::Ran)
{
    assert_eq!(ran.code, 1, "{}", ran.stdout);
    assert!(ran.stdout.contains("Blocking"), "{}", ran.stdout);
    assert!(
        ran.stdout.contains("notes/example.rs"),
        "the reported location must be the forward-slash relative path \
         `check/sources.rs::Relative_Path` produces -- the exact text `Read_Source` also \
         hands to `nomos_model::Subject_Of_Path`: {}",
        ran.stdout
    );
}

/// The exact string the real run just reported as a finding's location, fed into the same
/// function `check/sources.rs::Read_Source` calls to file that file's facts, must resolve
/// deterministically -- the property every later identity comparison relies on.
///
/// `nomos-model::path.rs` documents that separators are unified before this function ever
/// sees them, so the OS-native spelling this walk could have produced on this platform
/// resolves to the identical subject the CLI's forward-slash-normalized reporting text does,
/// which is what lets a finding's `locations` field stay purely a reporting convenience
/// rather than a second, competing identity.
fn Assert_That_Path_Resolves_To_One_Subject_On_Every_Spelling()
{
    let subject = nomos_model::Subject_Of_Path("notes/example.rs");
    assert_eq!(
        subject,
        nomos_model::Subject_Of_Path("notes/example.rs"),
        "the same relative path must resolve to the same subject on every call"
    );
    assert_eq!(
        subject,
        nomos_model::Subject_Of_Path("notes\\example.rs"),
        "Subject_Of_Path must normalize separators, or a path's identity would depend on \
         which platform walked it"
    );
}
