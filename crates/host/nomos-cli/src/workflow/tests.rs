//! What `nomos workflow` answers, driven end to end through [`Run`] and its parser.

use super::*;
use nomos_contracts::RuleId;
use nomos_gate_orchestration::{GateCommand, RuleSelector};
use nomos_workflow_orchestration::{Body, CheckBody, CommitIntent, CorrectionBody, GateBody};

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_A_Check_Run()
{
    let arguments: Vec<String> = ["run", "--check", "--root", "."].iter().map(|value| return (*value).to_owned()).collect();

    let command = Command_From_String_Arguments(&arguments).expect("the argv literal above names exactly one body flag, which this parser accepts");

    assert!(matches!(command.body, Body::Check(_)), "{command:?}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_A_Correct_Run()
{
    let arguments: Vec<String> = ["run", "--correct", "--root", ".", "--commit"].iter().map(|value| return (*value).to_owned()).collect();

    let command = Command_From_String_Arguments(&arguments).expect("the argv literal above names exactly one body flag, which this parser accepts");

    assert!(matches!(command.body, Body::Correction(ref correction) if correction.commit.Is_A_Commit()), "{command:?}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_A_Gate_Run()
{
    let arguments: Vec<String> =
        ["run", "--gate", "--root", ".", "--rule", "naming-convention"].iter().map(|value| return (*value).to_owned()).collect();

    let command = Command_From_String_Arguments(&arguments).expect("the argv literal above names exactly one body flag, which this parser accepts");

    assert!(matches!(command.body, Body::Gate(ref gate) if gate.command.rules.include == vec![RuleId::New("naming-convention")]), "{command:?}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_A_Claude_Code_Run()
{
    let arguments: Vec<String> =
        ["run", "--executor", "claude-code", "--goal", "say hello"].iter().map(|value| return (*value).to_owned()).collect();

    let command = Command_From_String_Arguments(&arguments).expect("the argv literal above names exactly one body flag, which this parser accepts");

    assert!(matches!(command.body, Body::ClaudeCode(ref task) if task.goal == "say hello"), "{command:?}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Refuse_More_Than_One_Body()
{
    let arguments: Vec<String> =
        ["run", "--check", "--executor", "claude-code"].iter().map(|value| return (*value).to_owned()).collect();

    let error = Command_From_String_Arguments(&arguments).expect_err("two bodies named");

    assert!(error.contains("exactly one"), "{error}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Refuse_No_Body_Named()
{
    let arguments: Vec<String> = ["run"].iter().map(|value| return (*value).to_owned()).collect();

    let error = Command_From_String_Arguments(&arguments).expect_err("no body named");

    assert!(error.contains("is required"), "{error}");
}

#[test]
fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
{
    let arguments: Vec<String> = ["not-a-real-verb"].iter().map(|value| return (*value).to_owned()).collect();

    let error = Command_From_String_Arguments(&arguments).expect_err("no such verb");

    assert!(error.contains("unknown command"), "{error}");
}

#[test]
fn Test_Run_Should_Report_Unavailable_Over_A_Root_That_Is_Not_A_Directory()
{
    let root = std::env::temp_dir().join("nomos-cli-workflow-missing-root");
    let _ignored = std::fs::remove_dir_all(&root);
    let body = CheckBody::New(root, Vec::new(), Vec::new());
    let command = WorkflowCommand { body: Body::Check(body) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Unavailable);
}

#[test]
fn Test_Run_Should_Report_Vacuous_Over_An_Empty_Directory()
{
    let root = std::env::temp_dir().join("nomos-cli-workflow-empty-root");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates an empty directory");
    let body = CheckBody::New(root.clone(), Vec::new(), Vec::new());
    let command = WorkflowCommand { body: Body::Check(body) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Vacuous);
}

#[test]
fn Test_Run_Should_Report_Ok_Over_A_Clean_Real_Source_File()
{
    let root = std::env::temp_dir().join("nomos-cli-workflow-clean-root");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates a directory");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the directory created by the line above is where this file lands");
    let body = CheckBody::New(root.clone(), Vec::new(), vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)]);
    let command = WorkflowCommand { body: Body::Check(body) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
}

/// One source file naming a mirror that no tree holds, which is what gives `--correct` a real
/// blocking claim to stage and then, under `--commit`, to write back.
const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
        /// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n\
        pub const THINGS: &[&str] = &[\"a\"];\n";

#[test]
fn Test_Run_Should_Commit_A_Real_Phantom_Claim_Through_Correct()
{
    let root = std::env::temp_dir().join("nomos-cli-workflow-correct-commit-root");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates a directory");
    std::fs::write(root.join("a.rs"), PHANTOM_FIXTURE).expect("the directory created by the line above is where this file lands");
    let body = CorrectionBody::New(root.clone(), Vec::new(), CommitIntent::Commit);
    let command = WorkflowCommand { body: Body::Correction(body) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let corrected = std::fs::read_to_string(root.join("a.rs")).expect("still readable");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
    assert_eq!(corrected, "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n");
}

#[test]
fn Test_Run_Should_Report_Ok_For_A_Passing_Gate()
{
    let result = Gate_Run_Over(GateFixture { directory: "nomos-cli-workflow-gate-passing-root", source: "pub fn Ok() {}\n" });

    assert_eq!(result.code, ExitCode::Ok, "{}", result.stderr);
}

#[test]
fn Test_Run_Should_Report_Refused_For_A_Failing_Gate()
{
    let result = Gate_Run_Over(GateFixture {
        directory: "nomos-cli-workflow-gate-failing-root",
        source: "pub fn Something(a: i32, b: i32, c: i32, d: i32, e: i32) {}\n",
    });

    assert_eq!(result.code, ExitCode::Refused, "{}", result.stderr);
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Refused => "Refused",
        ExitCode::Usage => "Usage",
        ExitCode::Unavailable => "Unavailable",
        ExitCode::Vacuous => "Vacuous",
    };
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this group's own
/// enum. Eight groups print an exit-code list and only those two mirrored it; the other six were
/// correct rather than guarded, which is a different thing, and `OD-AGENT-004`'s amendment says a
/// printed vocabulary is admissible only where a test compares it against its authority.
/// [`Documented_Exit_Codes`] reads the help text's half and [`Every_Exit_Code`] the enum's.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let documented = Documented_Exit_Codes();
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; \
         the enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}

/// The codes the usage text documents, in ascending order -- and the guard that the text it read
/// really is this group's own, since a help text belonging to some other command would compare two
/// unrelated lists and pass.
///
/// [`super::parsing::USAGE_TEXT`] is prose a person reads and [`ExitCode`] is what the process
/// returns, the two were written separately, and a code added or renumbered in one of them and not
/// the other is the failure that actually happens.
fn Documented_Exit_Codes() -> Vec<i32>
{
    use super::parsing::USAGE_TEXT;

    assert!(
        USAGE_TEXT.starts_with("usage: nomos workflow"),
        "this compared some other group's help text: {USAGE_TEXT}"
    );

    let (_, spelled) = USAGE_TEXT
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");

    return Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
}

/// Every code this group can leave the process with.
///
/// Here rather than beside [`ExitCode`] because that enum is `pub` and a zero-argument inherent
/// `All()` on it would be both a public-surface move and a declared universe in
/// `nomos_rules::universe::Is_The_Variant_List`'s sense, owing a row in
/// `tests/contract/tests/completeness_universes` -- neither file is this item's territory.
/// [`Labelled`]'s match has no wildcard arm, so a variant added to [`ExitCode`] fails this module
/// to *compile* rather than to pass, which is the forcing function that brings an author to this
/// array in the same edit.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::Refused,
        ExitCode::Usage,
        ExitCode::Unavailable,
        ExitCode::Vacuous,
    ];
}

/// A list of codes in ascending order, so that two of them can be compared as sets -- two readers
/// of the same codes reach it, so it sits in the trailing region rather than beside either.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// One gate run's own two inputs: the temp directory name it uses, and the source that decides
/// which disposition it reaches.
struct GateFixture
{
    directory: &'static str,
    source: &'static str,
}

/// One gate run's own answer: the code it exited with, and what it wrote to stderr, which is the
/// only place a gate says why it refused.
struct GateResult
{
    code: ExitCode,
    stderr: String,
}

/// The two gate dispositions are one run over two fixtures, so the run is written here once and
/// each test names only the source that decides which disposition it gets.
fn Gate_Run_Over(fixture: GateFixture) -> GateResult
{
    let root = std::env::temp_dir().join(fixture.directory);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the system temp directory is writable, and the line above cleared this name");
    std::fs::write(root.join("a.rs"), fixture.source).expect("the directory created by the line above is where this file lands");
    let gate = GateBody::New(Vec::new(), GateCommand { root: root.clone(), rules: RuleSelector { include: vec![RuleId::New(nomos_rules::PARAMETER_COUNT)] }, ..GateCommand::default() });
    let command = WorkflowCommand { body: Body::Gate(gate) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let report = String::from_utf8_lossy(&stderr).into_owned();

    let _ignored = std::fs::remove_dir_all(&root);
    return GateResult { code, stderr: report };
}
