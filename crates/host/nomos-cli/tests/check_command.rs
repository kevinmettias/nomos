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
//!
//! # Since `OD-GATE-004` these are assertions about CI
//!
//! The `Rules` step of `.github/workflows/gate.yml` runs this binary over this workspace,
//! and an exit code is the only thing a workflow step can observe. So every code asserted
//! here is now a pull request going green or red, which raises the bar on the *fixtures*
//! rather than on the assertions: a control is only worth as much as the tree it models.
//!
//! One of them was not worth much. Until this record the headline
//! — a phantom mirror must fail the command — was asserted over a one-file tree, and the
//! tree CI judges always holds `tests/corpus/analysis/gamma/broken.rs`, a committed fixture
//! the parser refuses on purpose. Measured at `fb55790`: the one-file fixture exited `1` and
//! the same fixture with `broken.rs` copied beside it exited `0`, the phantom rendered
//! `[Advisory]`. The headline was green for a reason that did not hold on the real workspace.
//! `OD-RULES-002` fixed the code and added the two-file case as a `check.rs` unit test
//! through `check::Run`; `OD-GATE-004` is where the *binary-level* fixture stops modelling a
//! tree nobody has, because it is what makes the binary's exit code a CI outcome.
//!
//! Nothing here shells out to `cargo`. `OD-GATE-002` records why a test that runs `cargo`
//! from inside `cargo test` deadlocks on the target-directory lock; `CARGO_BIN_EXE_nomos` is
//! the prebuilt program and needs no nested build.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, as cargo built it.
const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// The fewest files a run over this repository must have examined to have looked at it.
///
/// A floor in the shape `FEWEST_GOVERNING_RECORDS` uses, not a count: any item may raise it
/// and only a deliberate deletion lowers it. It exists because
/// `Test_This_Workspace_Should_Have_Nothing_That_Can_Fail_A_Build` would otherwise be
/// satisfied by a run over an empty tree — this repository's most-repeated defect appearing
/// inside the test that guards against it. Measured at 195 files on the commit that wired
/// the gate step.
const FEWEST_FILES_IN_THIS_WORKSPACE: usize = 150;

/// This repository's root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// The text of a file this workspace's parser refuses, taken from the workspace itself.
///
/// Not a hand-written approximation. `tests/corpus/analysis/gamma/broken.rs` is committed,
/// carries a byte order mark mid-file, and is the file every CI run over this tree walks and
/// cannot read — so a fixture holding *this* text is the tree the gate judges rather than one
/// resembling it. `tests/integration/tests/analysis_slice.rs` asserts the corpus README's
/// `broken.rs | Parses | no` row, so the fixture cannot be quietly repaired to make these
/// assertions easier either.
fn Text_The_Parser_Refuses() -> String
{
    let path = Repository_Root().join("tests/corpus/analysis/gamma/broken.rs");

    return std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "this workspace's own unparseable fixture must be readable at {}: {error}",
            path.display()
        )
    });
}

/// A scratch tree, removed when the test ends.
/// What one run of the binary exited with, and what it said.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct Ran
{
    code: i32,
    said: String,
}

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

    /// Runs `nomos check` over it, returning what it exited with and what it said.
    fn Check(&self) -> Ran
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

/// Runs the binary and returns what it exited with and what it said.
fn Run(arguments: &[&str]) -> Ran
{
    let finished = Command::new(Path::new(NOMOS))
        .args(arguments)
        .output()
        .expect("the binary cargo just built must be runnable");

    return Ran {
        code: finished.status.code().unwrap_or(-1),
        said: String::from_utf8_lossy(&finished.stdout).into_owned(),
    };
}

/// The headline, over the shape of tree the gate actually judges.
///
/// A universe naming a check that does not exist reads as covered and checks nothing, and
/// the command must stop a build over it — **including when the same run holds a file the
/// parser could not read**, because the workspace this command is now wired to judge always
/// holds one. The unreadable file is the fixture, not decoration: dropping it is what made
/// this assertion pass for a reason that did not hold on the real tree.
///
/// Measured, so this is falsifiable by a real prior state of the code rather than by a
/// hypothetical weakening. At `fb55790` this exact two-file tree exited `0` with the phantom
/// rendered `[Advisory]`, because one unreadable subject set an incompleteness flag over the
/// whole run and every phantom anywhere was downgraded; at `a5da564` it exits `1` with the
/// phantom `[Blocking]`. Restoring a run-wide downgrade turns this red.
///
/// What it adds over `check.rs`'s own
/// `Test_A_Phantom_Should_Block_Though_The_Tree_Holds_A_File_The_Parser_Refuses`, which
/// asserts the same property in process, is that the code survives `main`'s
/// `u8::try_from(code).unwrap_or(1)` and reaches a process — the only thing a workflow step
/// can see.
#[test]
fn Test_A_Phantom_Should_Fail_The_Command_Though_The_Tree_Holds_A_File_The_Parser_Refuses()
{
    let tree = Tree::New("phantom")
        .With(
            "universe.rs",
            "/// Mirrored by `Test_Nothing_Named_This`.\npub const TABLES: &[&str] = &[];\n",
        )
        .With("broken.rs", &Text_The_Parser_Refuses());

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, 1, "a false claim of coverage must fail: {output}");
    assert!(output.contains("Blocking"), "{output}");
    assert!(output.contains("Test_Nothing_Named_This"), "{output}");
    assert!(
        output.contains("1 of which can fail a build"),
        "the phantom is the finding that blocks: {output}"
    );
    assert!(
        output.contains("2 file(s) examined, 1 with a syntax fact"),
        "the run must still report the file it could not read rather than having quietly \
         excluded it: {output}"
    );
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

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, 0, "{output}");
    assert!(output.contains("0 of which can fail a build"), "{output}");
}

/// The other half of the headline's control, and the one that stops the over-correction.
///
/// The same two-file tree, with the claimed check present in a third file. A run holding a
/// file the parser refuses must still *pass* when nothing false is claimed — otherwise the
/// remedy for the false green would be "an unreadable file fails the run", which would make
/// this gate step red on this workspace forever, since `broken.rs` is committed and walked on
/// every run.
#[test]
fn Test_A_Mirror_That_Exists_Should_Still_Pass_Beside_A_File_The_Parser_Refuses()
{
    let tree = Tree::New("mirrored-beside-broken")
        .With(
            "universe.rs",
            "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];\n",
        )
        .With(
            "guard.rs",
            "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
        )
        .With("broken.rs", &Text_The_Parser_Refuses());

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, 0, "{output}");
    assert!(output.contains("0 of which can fail a build"), "{output}");
    assert!(
        output.contains("3 file(s) examined, 2 with a syntax fact"),
        "{output}"
    );
}

/// An admitted gap is honest. Twelve exist in this workspace, and blocking on them
/// would make a gate that can never be green.
#[test]
fn Test_An_Admitted_Gap_Should_Be_Reported_Without_Failing()
{
    let tree = Tree::New("gap").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");

    let Ran { code, said: output } = tree.Check();

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

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, 6, "an empty walk must not share an exit code with success");
    assert!(!output.contains("0 finding(s)"), "{output}");
}

/// The second shape of vacuity, through the shipped binary, which is the only shape the gate
/// step can see.
///
/// A walk that found source and materialized no fact for any of it. `check.rs`'s
/// `Test_A_Run_That_Materialized_No_Facts_Should_Not_Report_Clean` covers this in process; it
/// is a different branch from the one above — reached *after* the walk, once the provider has
/// been consulted and has answered for nothing — and until `OD-GATE-004` nothing exercised it
/// through a process. Collapsing the two leaves one of the vacuity guards untested where it
/// counts.
#[test]
fn Test_A_Run_That_Materialised_No_Facts_Should_Not_Exit_Zero()
{
    let tree = Tree::New("no-facts").With("broken.rs", &Text_The_Parser_Refuses());

    let Ran { code, said: output } = tree.Check();

    assert_eq!(
        code, 6,
        "a run that materialized no fact judged nothing, and a clean result would mean only \
         that the analysis never ran: {output}"
    );
}

/// A tree that is not there is a different failure from a tree that is clean, and from
/// one that judged nothing.
#[test]
fn Test_A_Root_That_Does_Not_Exist_Should_Be_Unreadable()
{
    let Ran { code, .. } = Run(&["check", "--root", "no-such-tree-anywhere-at-all"]);

    assert_eq!(code, 5);
}

/// A mistyped flag must not fall back to walking the current directory and reporting on
/// the wrong tree.
#[test]
fn Test_A_Mistyped_Flag_Should_Be_A_Usage_Error()
{
    let Ran { code, .. } = Run(&["check", "--rooot", "."]);

    assert_eq!(code, 2);
}

/// The gate step, one layer earlier: this workspace has nothing that can fail a build.
///
/// `OD-GATE-004` wired `cargo run … check --root .` into the gate, so from now on a phantom
/// introduced anywhere in this repository turns a pull request red. This asserts the same
/// thing at `cargo test` time, so the author finds out before pushing rather than after — and
/// it is the acceptance test for the step being green on the day it landed. Measured over the
/// tree this commit produces: 195 files, 194 with a syntax fact, 14 findings, 0 of which can
/// fail a build, exit `0`. The fourteen are twelve admitted gaps, which are advisory by `D-134` decision 4,
/// and the two `broken.rs` lines.
///
/// The file-count floor is not decoration. Without it this assertion is satisfied by a run
/// over an empty tree, which is this repository's most-repeated defect appearing inside the
/// test that guards against it — the same reason `boundaries.rs` carries
/// `Test_The_Workspace_Should_Not_Appear_Empty`.
///
/// It walks the real tree, so `cargo test -p nomos-cli` now depends on this repository's
/// contents. That is already true of `governing_records_are_present.rs` and of
/// `gate_covers_finish.rs`, and the run costs about a second.
#[test]
fn Test_This_Workspace_Should_Have_Nothing_That_Can_Fail_A_Build()
{
    let root = Repository_Root();
    let Ran { code, said: output } = Run(&["check", "--root", &root.display().to_string()]);
    assert!(
        output.contains("file(s) examined"),
        "the run did not reach its own report, so nothing below is about this workspace: \
         {output}"
    );
    let examined = Files_Examined(&output);

    assert!(
        examined >= FEWEST_FILES_IN_THIS_WORKSPACE,
        "{examined} file(s) examined under {}, and this workspace holds no fewer than \
         {FEWEST_FILES_IN_THIS_WORKSPACE}. A run over a tree this small has not looked at \
         this repository, so the clean verdict below would mean nothing",
        root.display()
    );
    assert!(
        output.contains("0 of which can fail a build"),
        "something in this workspace can now fail a build, and the gate's Rules step is red: \
         {output}"
    );
    assert_eq!(code, 0, "{output}");
}

/// The count the report opens with, which is how a run over the wrong tree is told from a run
/// over this one.
fn Files_Examined(output: &str) -> usize
{
    return output
        .split_once(" file(s) examined")
        .and_then(|(before, _)| {
            return before
                .split_whitespace()
                .next_back()
                .and_then(|count| return count.parse().ok());
        })
        .unwrap_or_else(|| panic!("the report must carry a file count: {output}"));
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
