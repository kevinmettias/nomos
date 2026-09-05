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

/// `ExitCode::All()`'s own mirror, named in its doc comment.
///
/// The match has no wildcard arm. A variant added to [`ExitCode`] without a matching arm
/// added here fails this file to *compile*, not merely to pass — the property `D-134` asks
/// a closed enum's mirror to have, and the shape `DocumentKind::All` and `Component::All`
/// already close it with. This used to be a private census array in this file
/// (`Every_Exit_Code`), kept private because promoting it to `ExitCode::All()` needed a row
/// in `tests/contract/tests/completeness_universes/table.rs`, a file outside the item that
/// wired the gate step's territory; that row now exists.
#[test]
fn Test_Every_ExitCode_Should_Be_Matched_Exhaustively()
{
    fn Ordinal(code: ExitCode) -> usize
    {
        return match code
        {
            ExitCode::Ok => 0,
            ExitCode::Violations => 1,
            ExitCode::Usage => 2,
            ExitCode::Unreadable => 3,
            ExitCode::Vacuous => 4,
        };
    }

    for (index, code) in ExitCode::All().iter().enumerate()
    {
        assert_eq!(
            Ordinal(*code),
            index,
            "{} is not matched at the position ExitCode::All() puts it, so the exhaustive \
             match and the universe have drifted apart",
            Labelled(*code)
        );
    }
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
    for code in ExitCode::All().iter().copied()
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

/// The numeric codes `check`'s own usage text documents (see [`USAGE`]'s "exit codes"
/// line): 0 nothing blocking, 1 findings that can fail a build, 2 usage, 5 unreadable
/// tree, 6 nothing was judged.
#[test]
fn Test_Value_Should_Return_The_Documented_Exit_Code_Number()
{
    assert_eq!(ExitCode::Ok.Value(), 0);
    assert_eq!(ExitCode::Violations.Value(), 1);
    assert_eq!(ExitCode::Usage.Value(), 2);
    assert_eq!(ExitCode::Unreadable.Value(), 5);
    assert_eq!(ExitCode::Vacuous.Value(), 6);
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
fn Test_All_Should_Match_The_Documented_Exit_Codes()
{
    let (_, spelled) = USAGE
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| word.parse().ok()));
    let implemented = Sorted(ExitCode::All().iter().map(|code| code.Value()));

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
    assert_eq!(Check_Command_From_String_Arguments(&[]).expect("no arguments is valid").root, PathBuf::from("."));
}

#[test]
fn Test_A_Given_Root_Should_Win()
{
    let arguments = vec!["--root".to_owned(), "somewhere".to_owned()];

    assert_eq!(
        Check_Command_From_String_Arguments(&arguments).expect("--root is valid").root,
        PathBuf::from("somewhere")
    );
}

/// A mistyped flag must not be silently ignored into a default. `nomos check --rooot x`
/// walking the current directory instead would report on the wrong tree and say
/// nothing about it.
#[test]
fn Test_Check_Command_From_String_Arguments_Should_Refuse_An_Unknown_Flag()
{
    let arguments = vec!["--rooot".to_owned(), "x".to_owned()];

    let error = Check_Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--rooot"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A tree that is not there is not a clean tree.
#[test]
fn Test_Run_Should_Report_Unreadable_For_A_Missing_Root()
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
        "/// Mirrored by `Test_Renamed_Away`.\npub const TABLE: &[&str] = &[];\n",
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
    let (code, rendered) = Broken_Provider_Run();
    let (clean_code, clean_rendered) = Clean_Run();

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

/// A tree with one clean file and one the parser genuinely refuses to read -- the run and
/// its rendered stdout, with the temporary root cleaned up before returning.
fn Broken_Provider_Run() -> (ExitCode, String)
{
    let root = std::env::temp_dir().join("nomos-check-coverage-debt-beside-clean");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(root.join("a.rs"), "pub fn ok()\n{\n}\n").expect("writable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);

    return (code, rendered);
}

/// A tree with nothing for the parser to refuse -- the run and its rendered stdout, with
/// the temporary root cleaned up before returning.
///
/// A real, if minimal, Cargo.toml is required: without one, `cargo metadata` cannot find a
/// workspace here at all, and the dependency-edges provider reports `ProviderUnavailable`
/// for a reason that has nothing to do with what this test means by "clean": a tree with no
/// findings, not a tree the provider cannot even see.
fn Clean_Run() -> (ExitCode, String)
{
    let root = std::env::temp_dir().join("nomos-check-clean-only");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(root.join("a.rs"), "pub fn ok()\n{\n}\n").expect("writable");
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"nomos-check-clean-only-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .expect("writable");
    std::fs::create_dir_all(root.join("src")).expect("the src directory is creatable");
    std::fs::write(root.join("src").join("lib.rs"), "").expect("writable");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);

    return (code, rendered);
}

/// One file claiming a mirror nothing declares, beside one the parser genuinely refuses.
fn A_Tree_With_A_Phantom_Beside_A_Refusal() -> PathBuf
{
    let root = std::env::temp_dir().join("nomos-check-phantom-beside-broken");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(
        root.join("a.rs"),
        "/// Mirrored by `Test_Renamed_Away`.\npub const TABLE: &[&str] = &[];\n",
    )
    .expect("writable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

    return root;
}
