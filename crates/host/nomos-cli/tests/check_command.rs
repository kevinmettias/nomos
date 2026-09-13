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
    fn With(self, name: &str, text: &str) -> Self
    {
        let path = self.root.join(name);
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
            "universe.rs",
            "/// Mirrored by `Test_Nothing_Named_This`.\npub const TABLES: &[&str] = &[];\n",
        )
        .With(UNPARSEABLE_FIXTURE, &Text_The_Parser_Refuses());

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
        .With(UNPARSEABLE_FIXTURE, &Text_The_Parser_Refuses());

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
    let tree = Tree::New("gap").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");

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
    let tree = Tree::New("no-facts").With(UNPARSEABLE_FIXTURE, &Text_The_Parser_Refuses());

    let Ran { code, said: output } = tree.Check();

    assert_eq!(
        code, 6,
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
        (&["check", "--root", "no-such-tree-anywhere-at-all"], 5, "a tree that is not there"),
        (&["check", "--rooot", "."], 2, "a mistyped flag"),
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

/// The `[Blocking]` findings this workspace's own tree carries today, and why each is
/// accepted rather than fixed -- the identical allowlist `crates/host/nomos-cli/src/gate/
/// tests.rs`'s own `ACCEPTED_BLOCKING_FINDINGS` carries, duplicated here rather than shared
/// because the two live in different crates (a library's own unit tests against a separate
/// integration-test binary) with no existing shared test-support dependency between them.
/// Being a copy, it goes stale the same way and on the same commits; `P96` is where both
/// were last emptied together.
///
/// Empty today, and that is the assertion rather than the absence of one: with no entry,
/// [`Only_Accepted_Findings_Are_Blocking`] means this workspace's own tree carries no
/// `[Blocking]` finding at all, which is strictly stronger than the two-named version it
/// replaces.
///
/// Both former entries were against the same file,
/// `tests/integration/fixtures/third-party/hex-0.4.3/lib.rs`, and they left for different
/// reasons. `single-letter-names: T` was a real gap and was fixed: `OD-CAPABILITY-014` put an
/// `impl` block's own generic parameters in the syntax payload, and the rule now exempts an
/// `Implementation` item whose own name is one of them (`P96`, `c278d896`). `abbreviations:
/// val` was *not* fixed and never will be -- this list's own previous text called it a
/// permanent, deliberate true positive, because `hex`'s author really did choose that name --
/// but that reasoning was always about the finding and never about whether this repository
/// walks the file. `P96` (`c3ff169e`) stopped the shared walk descending into a directory
/// carrying its own `standards.json`, so the vendored fixture is no longer judged from this
/// root at all, and `tests/integration/tests/calibration.rs` still judges it from its own.
///
/// A named allowlist rather than a bare count: a finding accepted the same deliberate way
/// must be added here explicitly, and anything not named here fails these tests -- neither
/// `nomos_gate_orchestration::Suppression` nor `RuleCalibration` is wired to a real config
/// file yet, so this allowlist is what stands in for that mechanism.
const ACCEPTED_BLOCKING_FINDINGS: &[&str] = &[];

/// Whether `output` carries no `[Blocking]` line other than the ones
/// [`ACCEPTED_BLOCKING_FINDINGS`] names.
fn Only_Accepted_Findings_Are_Blocking(output: &str) -> bool
{
    return output
        .lines()
        .filter(|line| return line.starts_with("[Blocking]"))
        .all(|line| return ACCEPTED_BLOCKING_FINDINGS.iter().any(|accepted| return line.starts_with(accepted)));
}

/// The gate step, one layer earlier: this workspace has nothing blocking beyond
/// [`ACCEPTED_BLOCKING_FINDINGS`]'s own two, named and accepted deliberately.
///
/// `OD-GATE-004` wired `cargo run … check --root .` into the gate, so from now on a phantom
/// introduced anywhere in this repository turns a pull request red. This asserts the same
/// thing at `cargo test` time, so the author finds out before pushing rather than after — and
/// it is the acceptance test for the step being green on the day it landed. `P71-GATE-TESTS-
/// OWN-CLEAN-TREE-ASSERTION-IS-STALE-2` measured this workspace's own tree carries exactly
/// two `[Blocking]` findings today, both named above; a bare "0 of which can fail a build"
/// assertion (this test's own shape before that item) stopped being true the moment the
/// first of the two was committed, and CI's own `Test` step failed on every push since,
/// unnoticed because nothing local ran this exact test's own predicate.
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
        Only_Accepted_Findings_Are_Blocking(&output),
        "a Blocking finding exists that ACCEPTED_BLOCKING_FINDINGS does not name -- a real, \
         new regression, or an accepted finding whose exact rendered text drifted: {output}"
    );
    assert_eq!(
        code, 0,
        "this workspace's own tree carries nothing that can fail a build today, and \
         ACCEPTED_BLOCKING_FINDINGS is empty to say so; a non-zero code here is either a real \
         regression or a finding somebody meant to accept without naming it. The file count \
         asserted above is what keeps this 0 from being a run over the wrong tree: {output}"
    );
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
        // A missing count cannot be reported as zero: zero is a meaningful answer here — it is
        // what a run over the wrong tree prints — and the callers of this function compare the
        // number to decide exactly that. Refusing to invent one keeps "the report changed
        // shape" from being read as "the run examined nothing". The whole report is printed
        // because the count's absence is only diagnosable next to what was printed instead.
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
