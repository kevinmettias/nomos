//! `nomos correct phantom-mirrors`, end to end, over real temporary trees -- never over
//! this repository's own tree, so a test run can never write a real correction into a file
//! this session did not claim.

use super::parsing::USAGE;
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

/// The name a fixture tree is filed under in the system temp directory.
///
/// A distinct type rather than a bare `&str`, so that the name and the contents a tree is
/// built from cannot be handed over in the wrong order: both would otherwise be strings, and
/// swapping them would produce a directory named after a file's whole text.
struct CaseName<'a>(&'a str);

/// A fresh temporary root holding one `a.rs` with `contents` -- the tree each test below
/// judges. Named for what the file declares, since which claim it carries is the only thing
/// that differs between them.
fn Root_Holding_A_Claim(case: CaseName<'_>, contents: &str) -> PathBuf
{
    let root = Fresh_Root(case.0);
    std::fs::write(root.join("a.rs"), contents).expect("the temporary root was created just above, so a new fixture file lands inside it");
    return root;
}

/// Runs `command` and answers with the stdout it rendered.
///
/// Every run in this file expects the same exit code -- `Ok` -- because none of them meets a
/// refusal; the assertion lives here so each test below reads as what it set up and what it
/// then observed.
fn Rendered_By(command: &CorrectCommand) -> String
{
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(command, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    assert_eq!(code, ExitCode::Ok, "{rendered} / stderr: {}", String::from_utf8_lossy(&stderr));
    return rendered;
}

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
    let root = Root_Holding_A_Claim(CaseName("nomos-cli-correct-clean-tree"), "pub fn Something() -> u32 { return 1; }\n");
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
    let root = Root_Holding_A_Claim(CaseName("nomos-cli-correct-dry-run"), PHANTOM_FIXTURE);
    let path = root.join("a.rs");
    let command = CorrectCommand { root: root.clone(), commit: false };

    let reported = Rendered_By(&command);
    let on_disk = std::fs::read_to_string(&path).expect("still readable");

    let _ignored = std::fs::remove_dir_all(&root);
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
    let root = Root_Holding_A_Claim(CaseName("nomos-cli-correct-commit"), PHANTOM_FIXTURE);
    let path = root.join("a.rs");
    let command = CorrectCommand { root: root.clone(), commit: true };

    let reported = Rendered_By(&command);
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
    let second_reported = Rendered_By(&command);
    let _ignored = std::fs::remove_dir_all(&root);

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

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this
/// group's own enum. Eight groups print an exit-code list and only those two mirrored it;
/// the other six were correct rather than guarded, which is a different thing, and
/// `OD-AGENT-004`'s amendment says a printed vocabulary is admissible only where a test
/// compares it against its authority. [`USAGE`] is prose a person reads and [`ExitCode`]
/// is what the process returns, the two were written separately, and a code added or
/// renumbered in one of them and not the other is the failure that actually happens.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    assert!(
        USAGE.starts_with("usage: nomos correct"),
        "this compared some other group's help text: {USAGE}"
    );

    let (_, spelled) = USAGE
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(ExitCode::Every_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented, implemented,
        "the usage text and ExitCode disagree about what this command can exit with"
    );
}
