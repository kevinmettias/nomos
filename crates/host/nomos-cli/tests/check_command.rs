//! `nomos check` as a user meets it: the real binary, a real tree, a real exit code.
//!
//! `P10-FIRST-CHECK` asks for a rule that is "reachable from the binary as a command
//! rather than from a test only". Every other test of this rule calls a function. This
//! one runs the program, and it is the only one that can fail if the subcommand is never
//! wired into `main`, if the exit code is discarded on the way out, or if the group is
//! spelled differently in the dispatcher than in the usage text.
//!
//! That gap is not hypothetical. `main` renders `u8::try_from(code).unwrap_or(1)`, so any
//! exit code this group invents that does not survive that conversion arrives as a plain
//! failure — and a `Vacuous` run that reports as an ordinary violation is exactly the
//! confusion the separate code was introduced to prevent.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, as cargo built it.
const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// A scratch tree, removed when the test ends.
struct Tree
{
    root: PathBuf,
}

impl Tree
{
    /// Makes one, named for the test that asked, so a failure leaves an identifiable
    /// directory behind rather than an anonymous one.
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-check-{name}-{}", std::process::id()));

        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory under the temp directory");

        return Self { root };
    }

    /// Writes one file into the tree.
    fn With(self, name: &str, text: &str) -> Self
    {
        std::fs::write(self.root.join(name), text).expect("writing into a directory just created");
        return self;
    }

    /// Runs `nomos check` over it, returning the exit code and stdout.
    fn Check(&self) -> (i32, String)
    {
        return Run(&["check", "--root", &self.root.display().to_string()]);
    }
}

impl Drop for Tree
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// Runs the binary and returns its exit code and stdout.
fn Run(arguments: &[&str]) -> (i32, String)
{
    let finished = Command::new(Path::new(NOMOS))
        .args(arguments)
        .output()
        .expect("the binary cargo just built must be runnable");

    return (
        finished.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&finished.stdout).into_owned(),
    );
}

/// The headline. A universe naming a check that does not exist reads as covered and
/// checks nothing, and the command must stop a build over it.
#[test]
fn Test_A_Phantom_Mirror_Should_Fail_The_Command()
{
    let tree = Tree::New("phantom").With(
        "universe.rs",
        "/// Mirrored by `Test_Nothing_Named_This`.\npub const TABLES: &[&str] = &[];\n",
    );

    let (code, output) = tree.Check();

    assert_eq!(code, 1, "a false claim of coverage must fail: {output}");
    assert!(output.contains("Blocking"), "{output}");
    assert!(output.contains("Test_Nothing_Named_This"), "{output}");
}

/// The control that stops the previous test passing for the wrong reason. The same tree
/// with the named check present must succeed — otherwise the command is a counter of
/// universes rather than a judge of them.
#[test]
fn Test_A_Mirror_That_Exists_Should_Pass_The_Command()
{
    let tree = Tree::New("mirrored")
        .With(
            "universe.rs",
            "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];\n",
        )
        .With(
            "guard.rs",
            "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
        );

    let (code, output) = tree.Check();

    assert_eq!(code, 0, "{output}");
    assert!(output.contains("0 of which can fail a build"), "{output}");
}

/// An admitted gap is honest. Thirteen exist in this workspace, and blocking on them
/// would make a gate that can never be green.
#[test]
fn Test_An_Admitted_Gap_Should_Be_Reported_Without_Failing()
{
    let tree = Tree::New("gap").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");

    let (code, output) = tree.Check();

    assert_eq!(code, 0, "{output}");
    assert!(output.contains("Advisory"), "{output}");
    assert!(output.contains("1 finding(s)"), "{output}");
}

/// A run that judged nothing is not a clean run. The sibling workspace shipped
/// `check-standards-tree /nonexistent` walking nothing, finding nothing and reporting
/// CLEAN, and then found the same defect in three more checks.
#[test]
fn Test_A_Tree_With_No_Source_Should_Not_Report_Clean()
{
    let tree = Tree::New("vacuous").With("README.md", "no rust here\n");

    let (code, output) = tree.Check();

    assert_eq!(code, 6, "an empty walk must not share an exit code with success");
    assert!(!output.contains("0 finding(s)"), "{output}");
}

/// A tree that is not there is a different failure from a tree that is clean, and from
/// one that judged nothing.
#[test]
fn Test_A_Root_That_Does_Not_Exist_Should_Be_Unreadable()
{
    let (code, _output) = Run(&["check", "--root", "no-such-tree-anywhere-at-all"]);

    assert_eq!(code, 5);
}

/// A mistyped flag must not fall back to walking the current directory and reporting on
/// the wrong tree.
#[test]
fn Test_A_Mistyped_Flag_Should_Be_A_Usage_Error()
{
    let (code, _output) = Run(&["check", "--rooot", "."]);

    assert_eq!(code, 2);
}

/// The group has to be discoverable, or a command nobody can find is a command nobody
/// runs.
#[test]
fn Test_The_Binary_Should_Name_The_Check_Group()
{
    let finished = Command::new(Path::new(NOMOS))
        .output()
        .expect("the binary must run with no arguments");

    let usage = String::from_utf8_lossy(&finished.stderr);

    assert!(usage.contains("check"), "{usage}");
}
