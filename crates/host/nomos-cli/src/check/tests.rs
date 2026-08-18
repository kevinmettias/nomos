//! What this module promises, exercised.
//!
//! What used to compose the registry, ingest a walk and judge it by hand
//! (`Composed::Over(sources).Findings(sources)`) moved with that composition to
//! `nomos-check-orchestration`'s own test suite -- it is a statement about that crate's
//! seam now, not about this one. What is left here is black-box: every test below drives
//! `Run` end to end, over a real temporary tree, the way the shipped binary is actually
//! called.

use super::*;
use super::parsing::USAGE;

/// ---- the exit-code policy the gate step rests on ----
///
/// Every code this group can leave the process with, written twice on purpose.
///
/// The array is what the assertions below iterate. [`Labelled`] is an exhaustive `match`,
/// so a variant added to [`ExitCode`] fails to compile *there* — which is the only
/// mechanism available without a derive that a code-adder cannot walk past, and it stops
/// them inside the function they have to extend. What the match does not force is adding
/// the new code to this array; that residual is closed from the other side by
/// `Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With`, which
/// compares the array against the usage text a person reads.
///
/// # Why this is not `ExitCode::All()`
///
/// That was the tidier shape and it was tried. An `All()` in an inherent implementation is
/// a *declared universe* — `nomos-rules` finds it by that exact name — so the enum this
/// gate step's policy rests on would need a row in
/// `tests/contract/tests/completeness_universes.rs` saying what compares the list against
/// the reality it enumerates. Measured: without that row,
/// `Test_The_Declared_Table_Should_Match_What_Is_Derived` and
/// `Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror` go red naming
/// `ExitCode::All`, and the scanned total goes from sixteen universes to seventeen. That
/// file is outside `P10-CHECK-GATE`'s territory, so the census stays private here, where
/// it is not a universe at all. Promoting it is worth doing by whoever holds that file
/// next — the mechanism refusing an unclassified list is the rule working, not an
/// obstacle.
fn Every_Exit_Code() -> [ExitCode; 5]
{
    return [
        ExitCode::Ok,
        ExitCode::Violations,
        ExitCode::Usage,
        ExitCode::Unreadable,
        ExitCode::Vacuous,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Violations => "Violations",
        ExitCode::Usage => "Usage",
        ExitCode::Unreadable => "Unreadable",
        ExitCode::Vacuous => "Vacuous",
    };
}

/// The whole exit-code policy as one assertion, and the reason the workflow needs no
/// branch.
///
/// `OD-GATE-004` decided that zero is the only success and implemented it by writing no
/// policy: Actions fails a step on any non-zero exit. So this is the only place in the
/// tree where the policy is checkable. If [`ExitCode::Vacuous`] were renumbered to `0`
/// "because there is nothing to report", CI would start passing runs that judged nothing
/// and nothing else would notice.
#[test]
fn Test_Only_Ok_Should_Carry_The_Passing_Exit_Code()
{
    for code in Every_Exit_Code()
    {
        assert_eq!(
            code.Value() == 0,
            code == ExitCode::Ok,
            "{} exits {}, and the gate reads zero and only zero as success",
            Labelled(code),
            code.Value()
        );
    }
}

/// The codes this file documents are the codes this group can exit with.
///
/// `P10-CHECK-GATE`'s `done_when` asks that the codes the gate rests on be "the ones
/// crates/host/nomos-cli/src/check.rs documents, read from there rather than restated".
/// The workflow honours the second half by restating nothing. This is what makes the
/// first half true of *this* file: [`USAGE`] is prose a person reads and [`ExitCode`] is
/// what the process returns, the two were written separately, and a code added or
/// renumbered in one of them and not the other is the failure that actually happens.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let (_, spelled) = USAGE
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: \
         {spelled}"
    );
    assert_eq!(
        documented, implemented,
        "the usage text and ExitCode disagree about what this command can exit with, \
         and the gate step reads its policy off the latter"
    );
}

#[test]
fn Test_A_Root_Should_Default_To_Here()
{
    assert_eq!(Parse(&[]).expect("no arguments is valid").root, PathBuf::from("."));
}

#[test]
fn Test_A_Given_Root_Should_Win()
{
    let arguments = vec!["--root".to_owned(), "somewhere".to_owned()];

    assert_eq!(
        Parse(&arguments).expect("--root is valid").root,
        PathBuf::from("somewhere")
    );
}

/// A mistyped flag must not be silently ignored into a default. `nomos check --rooot x`
/// walking the current directory instead would report on the wrong tree and say
/// nothing about it.
#[test]
fn Test_An_Unknown_Flag_Should_Refuse()
{
    let arguments = vec!["--rooot".to_owned(), "x".to_owned()];

    let error = Parse(&arguments).expect_err("must refuse");

    assert!(error.contains("--rooot"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A tree that is not there is not a clean tree.
#[test]
fn Test_A_Missing_Root_Should_Be_Unreadable()
{
    let command = CheckCommand {
        root: PathBuf::from("no-such-directory-anywhere"),
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    assert_eq!(Run(&command, &mut stdout, &mut stderr), ExitCode::Unreadable);
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// A run that materialized nothing must not print a clean tree.
///
/// Every file the walk found is one the provider refuses, so the store is empty. That
/// is `Vacuous` — the answer is empty because something expected was not there — and
/// not `Ok`.
#[test]
fn Test_A_Run_That_Materialized_No_Facts_Should_Not_Report_Clean()
{
    let root = std::env::temp_dir().join("nomos-check-no-facts");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;").expect("writable");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(code, ExitCode::Vacuous);
    assert!(
        String::from_utf8_lossy(&stderr).contains("no syntax fact was materialized"),
        "{}",
        String::from_utf8_lossy(&stderr)
    );
}

/// And the control for it: a tree the provider *can* read exits on what the rule found
/// rather than on vacuity. Without this the test above is satisfied by a command that
/// always reports `Vacuous`.
#[test]
fn Test_A_Run_That_Materialized_Facts_Should_Judge_Rather_Than_Refuse()
{
    let root = std::env::temp_dir().join("nomos-check-with-facts");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(
        root.join("a.rs"),
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    )
    .expect("writable");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(
        code,
        ExitCode::Violations,
        "{}",
        String::from_utf8_lossy(&stdout)
    );
}

/// ---- one unreadable file does not silence the rest of the tree ----
///
/// The measurement `OD-RULES-002` was opened against, reproduced as a directory. Two
/// files: one the real parser reads, declaring a mirror that resolves to nothing, and
/// one the real parser refuses. Before that record this run exited `0` and printed the
/// phantom as `[Advisory]`, because `broken.rs` set one incompleteness flag over the
/// whole run — and this workspace always holds such a file, so the guard could never
/// block on anything.
///
/// Asserted through `Run` and not through the rule, because the thing that was wrong
/// was the exit code of the shipped binary. The provider here is the registered one, so
/// the refusal is a real refusal rather than a withheld fixture.
#[test]
fn Test_A_Phantom_Should_Block_Though_The_Tree_Holds_A_File_The_Parser_Refuses()
{
    let root = A_Tree_With_A_Phantom_Beside_A_Refusal();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(code, ExitCode::Violations, "{rendered}");
    assert_ne!(code.Value(), ExitCode::Ok.Value(), "{rendered}");
    assert!(
        rendered.contains("2 file(s) examined, 1 with a syntax fact"),
        "the run must still report what it could not read: {rendered}"
    );
    assert!(
        rendered.contains("1 of which can fail a build"),
        "the phantom is the finding that blocks: {rendered}"
    );
    assert!(
        rendered.contains("[Blocking]") && rendered.contains("Test_Renamed_Away"),
        "{rendered}"
    );
}

/// ---- `OD-COMPLETENESS-004`'s negative control ----
///
/// A tree where the real parser refuses one file and no universe claims anything, so the
/// run exits `Ok` — nothing blocks. Before `OD-COMPLETENESS-004` that exit code was the
/// whole story, and it is the same code a tree with no unreadable file at all would exit
/// with. This is the case `done_when` names: a subject the run could not judge must not
/// render the same as a subject that was judged clean, even though neither one fails the
/// build.
#[test]
fn Test_A_Provider_Refusal_Must_Not_Render_The_Same_As_A_Clean_Run()
{
    let root = std::env::temp_dir().join("nomos-check-coverage-debt-beside-clean");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(root.join("a.rs"), "pub fn ok() {}\n").expect("writable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);

    let clean_root = std::env::temp_dir().join("nomos-check-clean-only");
    let _ignored = std::fs::remove_dir_all(&clean_root);
    std::fs::create_dir_all(&clean_root).expect("the temporary root is creatable");
    std::fs::write(clean_root.join("a.rs"), "pub fn ok() {}\n").expect("writable");

    let mut clean_stdout = Vec::new();
    let mut clean_stderr = Vec::new();
    let clean_code = Run(&CheckCommand { root: clean_root.clone() }, &mut clean_stdout, &mut clean_stderr);
    let clean_rendered = String::from_utf8_lossy(&clean_stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&clean_root);

    // Neither run fails the build: nothing declares a mirror, so there is nothing to be a
    // phantom about, and the refusal is advisory. That is exactly why the exit code alone
    // cannot be the thing that tells these two runs apart.
    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert_eq!(clean_code, ExitCode::Ok, "{clean_rendered}");
    assert_eq!(code, clean_code, "the exit code is not where this distinction lives");

    assert_ne!(
        rendered, clean_rendered,
        "a run carrying a real provider refusal rendered identically to a clean run"
    );
    assert!(rendered.contains("claim: incomplete"), "{rendered}");
    assert!(clean_rendered.contains("claim: complete"), "{clean_rendered}");
    assert!(
        !clean_rendered.contains("DependencyUnavailable") && !clean_rendered.contains("Unparseable"),
        "{clean_rendered}"
    );
}

/// One file claiming a mirror nothing declares, beside one the parser genuinely refuses.
fn A_Tree_With_A_Phantom_Beside_A_Refusal() -> PathBuf
{
    let root = std::env::temp_dir().join("nomos-check-phantom-beside-broken");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(
        root.join("a.rs"),
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    )
    .expect("writable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

    return root;
}
