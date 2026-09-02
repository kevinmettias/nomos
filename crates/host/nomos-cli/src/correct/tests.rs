//! `nomos correct phantom-mirrors`, end to end, over real temporary trees -- never over
//! this repository's own tree, so a test run can never write a real correction into a file
//! this session did not claim.

use super::{CorrectCommand, ExitCode, Run};
use std::path::PathBuf;

/// Removes and recreates `name` under the system temp directory, so a test starts from a
/// clean tree regardless of what an earlier run left behind. The same pattern
/// `check::sources::tests::Fresh_Root` already uses.
fn Fresh_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    return root;
}

/// A single-file fixture declaring one universe whose claimed mirror does not exist
/// anywhere in the tree -- a real blocking Phantom, produced by the real rule, not a
/// hand-built `Finding`.
const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
/// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n\
pub const THINGS: &[&str] = &[\"a\"];\n";

#[test]
fn Test_An_Unreadable_Root_Should_Refuse()
{
    let root = PathBuf::from("this/does/not/exist/anywhere/on/this/machine");
    let command = CorrectCommand { root, commit: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Unreadable);
}

#[test]
fn Test_An_Empty_Tree_Should_Be_Vacuous()
{
    let root = Fresh_Root("nomos-cli-correct-empty-tree");
    let command = CorrectCommand { root: root.clone(), commit: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Vacuous, "an empty tree must not report the same code as a clean run");
}

#[test]
fn Test_A_Tree_With_No_Phantom_Should_Be_Clean()
{
    let root = Fresh_Root("nomos-cli-correct-clean-tree");
    std::fs::write(root.join("a.rs"), "pub fn Something() -> u32 { return 1; }\n").expect("writable");
    let command = CorrectCommand { root: root.clone(), commit: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let reported = String::from_utf8_lossy(&stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Ok, "{reported} / stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(reported.contains("clean"), "{reported}");
}

#[test]
fn Test_A_Dry_Run_Should_Report_The_Phantom_And_Change_Nothing_On_Disk()
{
    let root = Fresh_Root("nomos-cli-correct-dry-run");
    let path = root.join("a.rs");
    std::fs::write(&path, PHANTOM_FIXTURE).expect("writable");
    let command = CorrectCommand { root: root.clone(), commit: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let reported = String::from_utf8_lossy(&stdout).into_owned();
    let on_disk = std::fs::read_to_string(&path).expect("still readable");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Ok, "{reported} / stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(reported.contains("dry run"), "{reported}");
    assert_eq!(on_disk, PHANTOM_FIXTURE, "a dry run must not touch the file");
}

/// The whole point: a real blocking Phantom finding, produced by the real rule over a real
/// temp tree, becomes a real committed correction that strikes exactly the false claim and
/// nothing else -- and a second run over the corrected tree reports the same universe
/// clean rather than finding a second phantom where the first one used to be.
#[test]
fn Test_Committing_Should_Strike_The_Claim_On_Disk_And_Leave_A_Clean_Rerun()
{
    let root = Fresh_Root("nomos-cli-correct-commit");
    let path = root.join("a.rs");
    std::fs::write(&path, PHANTOM_FIXTURE).expect("writable");
    let command = CorrectCommand { root: root.clone(), commit: true };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let reported = String::from_utf8_lossy(&stdout).into_owned();

    assert_eq!(code, ExitCode::Ok, "{reported} / stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(reported.contains("committed"), "{reported}");

    let corrected = std::fs::read_to_string(&path).expect("still readable");
    assert_eq!(
        corrected,
        "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n",
        "only the phantom claim's own line should be gone"
    );

    // Rerun over the corrected tree: the same universe now declares no mirror at all,
    // which is an admitted-gap advisory finding -- not a Phantom, so this command reports
    // the tree clean rather than finding a second candidate.
    let mut second_stdout = Vec::new();
    let mut second_stderr = Vec::new();
    let second_code = Run(&command, &mut second_stdout, &mut second_stderr);
    let second_reported = String::from_utf8_lossy(&second_stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(second_code, ExitCode::Ok, "{second_reported}");
    assert!(second_reported.contains("clean"), "{second_reported}");
}

/// Every code this group can leave the process with is matched exhaustively -- fails to
/// compile, not merely to pass, if a variant is added to [`ExitCode`] without an arm here.
#[test]
fn Test_Every_ExitCode_Should_Be_Matched_Exhaustively()
{
    for code in ExitCode::Every_Code()
    {
        let _named = match code
        {
            ExitCode::Ok => "ok",
            ExitCode::Refused => "refused",
            ExitCode::Usage => "usage",
            ExitCode::Unreadable => "unreadable",
            ExitCode::Vacuous => "vacuous",
        };
    }
}

#[test]
fn Test_Only_Ok_Should_Carry_The_Passing_Exit_Code()
{
    for code in ExitCode::Every_Code()
    {
        assert_eq!(code.Value() == 0, *code == ExitCode::Ok, "{code:?}");
    }
}
