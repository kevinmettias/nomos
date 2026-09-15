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

/// The suite's own part that judges this repository rather than a scratch tree.
///
/// Declared through `#[path]` rather than as a bare `mod`, the way `list_tells_the_truth.rs`
/// and `scratch_ledger/board.rs` are: a bare `mod` in a test binary root resolves against
/// `tests/`, which cargo would then build as a further test target.
#[path = "check_command/this_workspace.rs"]
mod this_workspace;

/// The binary under test, as cargo built it.
const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// This repository's root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// Where a copy of this workspace's committed unparseable fixture is written in a scratch
/// tree, and why the directory is part of the fixture rather than incidental.
///
/// The committed original is `tests/corpus/analysis/gamma/broken.rs`, and its first line is
/// `use super::*;`. `Check_No_Wildcard_Imports` exempts that exact idiom in a file its own
/// `Is_Test_Or_Example_Source` recognizes, which the committed path is and a bare `broken.rs`
/// at a scratch root is not -- so a copy written at the root picks up a blocking finding the
/// original never has, and the tests below need this file to be inert about everything except
/// being unreadable. Writing it under `tests/` restores the exemption by restoring the fact
/// the exemption is about, rather than by editing committed content or asserting around the
/// finding.
const UNPARSEABLE_FIXTURE: &str = "tests/broken.rs";

/// The exit code for a run that judged nothing.
///
/// An empty answer because something the run expected was not there — a walk that found no
/// source, or one that materialized no fact for any of it. Apart from `0` on purpose, which
/// is the whole subject of [`Test_A_Tree_With_No_Source_Should_Not_Report_Clean`].
const EXIT_ABSENT: i32 = 6;

/// The exit code for a root that could not be walked at all.
///
/// [`Text_The_Parser_Refuses`]'s own file is a subject the walk cannot read; a root that is
/// not there is the walk itself failing, which is why the two do not share a code.
const EXIT_STORE_ERROR: i32 = 5;

/// The exit code for a command line the binary refuses.
const EXIT_USAGE: i32 = 2;

/// The name one file of a scratch tree is written at, relative to that tree's root.
///
/// A distinct type rather than a bare `&str`: [`Tree::With`] takes a name and a text, both
/// strings and both adjacent, so a caller who transposed them would write a file named
/// after its own contents and be told nothing about it.
struct FileName<'a>(&'a str);

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
        // Falling back to a hand-written approximation is the failure this fixture exists to
        // prevent, and a `Result` here invites exactly that at the call site. If the committed
        // file has moved or been repaired, the tests below are no longer about the tree CI
        // walks and must stop rather than quietly test a substitute; the path is printed
        // because a moved fixture and an unreadable one need different fixes.
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

    /// Writes one file into the tree, creating any directory its name asks for.
    ///
    /// Nested names matter here: [`UNPARSEABLE_FIXTURE`] has to land under a directory that
    /// makes it a test source, the way its committed original is one.
    fn With(self, name: FileName<'_>, text: &str) -> Self
    {
        let path = self.root.join(name.0);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("parent directories are creatable");
        }
        std::fs::write(path, text).expect("writing into a directory just created");
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
            FileName("universe.rs"),
            "/// Mirrored by `Test_Nothing_Named_This`.\npub const TABLES: &[&str] = &[];\n",
        )
        .With(FileName(UNPARSEABLE_FIXTURE), &Text_The_Parser_Refuses());

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
            FileName("universe.rs"),
            "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];\n",
        )
        .With(
            FileName("guard.rs"),
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
            FileName("universe.rs"),
            "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];\n",
        )
        .With(
            FileName("guard.rs"),
            "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
        )
        .With(FileName(UNPARSEABLE_FIXTURE), &Text_The_Parser_Refuses());

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
///
/// Four findings, not one: `tree` is a temporary directory with no `Cargo.toml` and no
/// `deny.toml`, so the dependency-edges provider, the lint-diagnostics `ToolProvider` and
/// the dependency-policy `ToolProvider` can none of them run there, and each absence is
/// reported as its own advisory finding rather than silently producing zero findings for
/// that capability -- the same honesty this test's own name is about, three times over.
/// `dependency-policy`'s own absence joined this count once
/// `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT` made `cargo deny` refuse honestly over a
/// root with no `deny.toml` of its own, rather than silently walking upward to an
/// ancestor's.
#[test]
fn Test_An_Admitted_Gap_Should_Be_Reported_Without_Failing()
{
    let tree = Tree::New("gap").With(FileName("universe.rs"), "pub const TABLES: &[&str] = &[];\n");

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, 0, "{output}");
    assert!(output.contains("Advisory"), "{output}");
    assert!(output.contains("4 finding(s)"), "{output}");
}

/// A run that judged nothing is not a clean run. The sibling workspace shipped
/// `check-standards-tree /nonexistent` walking nothing, finding nothing and reporting
/// CLEAN, and then found the same defect in three more checks.
#[test]
fn Test_A_Tree_With_No_Source_Should_Not_Report_Clean()
{
    let tree = Tree::New("vacuous").With(FileName("README.md"), "no rust here\n");

    let Ran { code, said: output } = tree.Check();

    assert_eq!(code, EXIT_ABSENT, "an empty walk must not share an exit code with success");
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
    let tree = Tree::New("no-facts").With(FileName(UNPARSEABLE_FIXTURE), &Text_The_Parser_Refuses());

    let Ran { code, said: output } = tree.Check();

    assert_eq!(
        code, EXIT_ABSENT,
        "a run that materialized no fact judged nothing, and a clean result would mean only \
         that the analysis never ran: {output}"
    );
}

/// Arguments the invocation refuses before any tree is walked, paired with the exit code
/// each refusal reports, and why: a missing root is a different failure from a tree that is
/// clean or one that judged nothing, and a mistyped flag must not fall back to walking the
/// current directory and reporting on the wrong tree.
fn Refused_Before_Any_Tree_Is_Walked() -> Vec<(&'static [&'static str], i32, &'static str)>
{
    return vec![
        (&["check", "--root", "no-such-tree-anywhere-at-all"], EXIT_STORE_ERROR, "a tree that is not there"),
        (&["check", "--rooot", "."], EXIT_USAGE, "a mistyped flag"),
    ];
}

#[test]
fn Test_An_Invocation_Refused_Before_Walking_Should_Report_Its_Own_Code()
{
    for (arguments, expected_code, why) in Refused_Before_Any_Tree_Is_Walked()
    {
        let Ran { code, .. } = Run(arguments);

        assert_eq!(code, expected_code, "{why}: {arguments:?}");
    }
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
