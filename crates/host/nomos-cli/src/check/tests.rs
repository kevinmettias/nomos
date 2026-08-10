//! What this module promises, exercised.

use super::*;
use nomos_contracts::{Applicability, GateCategory};
use super::parsing::USAGE;
use super::facts::Ingested;

/// The composition this command really ships, over sources a test wrote by hand.
///
/// Not a stub. The registry holds the real contract and the real parser's real offer,
/// and the store holds facts the real provider produced — which is what makes every
/// assertion below a statement about the shipped binary rather than about a fixture.
struct Composed
{
    registry: Registry,
    store: MemoryFactStore,
    context: Context,
}

impl Composed
{
    fn Over(sources: &[SourceFile]) -> Self
    {
        let registry = Registered();
        let mut refused = Vec::new();
        let context = Ingested(sources, Path::new("."), &registry, &mut refused)
            .expect("the fixture is a valid tree");
        let mut store = MemoryFactStore::New();
        let _written = Materialize_Syntax(sources, &context, &mut store);

        return Self {
            registry,
            store,
            context,
        };
    }

    fn Findings(&self, sources: &[SourceFile]) -> Vec<Finding>
    {
        let mut reader = Reader::On(&self.store, &self.registry, self.context.clone());

        return Check_Completeness_Mirrors(sources, &mut reader);
    }
}

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

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

    let mut documented: Vec<i32> = spelled
        .split_whitespace()
        .filter_map(|word| return word.parse::<i32>().ok())
        .collect();
    documented.sort_unstable();

    let mut implemented: Vec<i32> = Every_Exit_Code()
        .iter()
        .map(|code| return code.Value())
        .collect();
    implemented.sort_unstable();

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

#[test]
fn Test_A_Blocking_Finding_Should_Exit_Nonzero()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let findings = Composed::Over(&sources).Findings(&sources);
    let mut stdout = Vec::new();

    assert_eq!(
        Report(&findings, Examined { files: 1, facts: 1 }, &mut stdout),
        ExitCode::Violations
    );
}

/// An advisory finding is reported and does not stop anybody. Twelve of them exist
/// in this workspace today, and a gate that can never be green is one everybody
/// learns to bypass.
#[test]
fn Test_An_Advisory_Finding_Should_Not_Fail_The_Run()
{
    let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];
    let findings = Composed::Over(&sources).Findings(&sources);
    let mut stdout = Vec::new();

    assert_eq!(
        Report(&findings, Examined { files: 1, facts: 1 }, &mut stdout),
        ExitCode::Ok
    );
    assert!(!findings.is_empty(), "there is something to report");
}

/// The counts are part of the result. Without them, a broken walk and a clean tree
/// render the same line.
#[test]
fn Test_The_Report_Should_Say_How_Much_Was_Looked_At()
{
    let mut stdout = Vec::new();

    let _code = Report(&[], Examined { files: 41, facts: 39 }, &mut stdout);

    let rendered = String::from_utf8(stdout).expect("output is utf-8");

    assert!(rendered.contains("41 file(s) examined"), "{rendered}");
    assert!(rendered.contains("39 with a syntax fact"), "{rendered}");
}

/// ---- the shipped binary consults a fact ----
///
/// The assertion `P10-FACT-BYPASS` turns on. The parser this command registers really
/// produces the fact, and the rule's verdict really depends on it: with the defining
/// file's fact in the store the claim resolves and nothing is reported, and with it
/// withheld the claim does not resolve. Same text, same rule, different store.
#[test]
fn Test_The_Composed_Command_Should_Resolve_A_Mirror_Through_A_Real_Fact()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_The_Real_Provider_Found_This`.\n\
         pub const T: &[&str] = &[];\n",
    );
    let checking = Source(
        "b.rs",
        "#[cfg(test)]\nmod tests\n{\n    #[test]\n    fn Test_The_Real_Provider_Found_This()\n    {\n    }\n}\n",
    );
    let whole = vec![declaring.clone(), checking.clone()];

    let resolved = Composed::Over(&whole).Findings(&whole);
    assert!(
        resolved.is_empty(),
        "the registered parser must find the check in b.rs: {resolved:?}"
    );

    // The store is told about the declaring file only; the rule is handed both.
    let short = Composed::Over(&[declaring]).Findings(&whole);
    assert!(
        short
            .iter()
            .any(|finding| return finding.subject_name == "T"),
        "with b.rs's fact withheld the claim must not resolve: {short:?}"
    );
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
    let root = std::env::temp_dir().join("nomos-check-phantom-beside-broken");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    std::fs::write(
        root.join("a.rs"),
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    )
    .expect("writable");
    std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

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

/// The floor is the rule's, and the run this command composes meets it.
///
/// Asserted here because this is the only place in the workspace where the rule's
/// requirement and a real provider's offer are both nameable. If it ever failed, every
/// subject would be reported unread and the command would report that it could not run
/// — which is honest, and is not what anybody installed it for.
#[test]
fn Test_The_Registered_Provider_Should_Satisfy_The_Rules_Floor()
{
    let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];
    let findings = Composed::Over(&sources).Findings(&sources);

    assert!(
        findings
            .iter()
            .all(|finding| return finding.applicability == Applicability::Supported),
        "the registered parser must serve nomos_rules::Syntax_Requirement: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| return finding.gate == GateCategory::Advisory),
        "{findings:?}"
    );
}
