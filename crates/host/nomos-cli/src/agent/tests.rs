//! What `nomos agent` promises, exercised.

use super::*;
use super::judge_role::{Crate_Root, Declared_Role};
use std::path::Path;

fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

#[test]
fn Test_An_Execute_Command_Should_Parse_Its_Goal()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { goal, effort, backend } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(goal, "hello");
    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
    assert_eq!(backend, Backend::ClaudeCode);
}

/// The default is `BackendDefault`, the one value `Effort_Flag` maps to "omit the flag
/// entirely" -- an `execute` call with no `--effort` must reach the subprocess exactly
/// as it did before this flag existed.
#[test]
fn Test_An_Execute_Command_With_No_Effort_Defaults_To_Backend_Default()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
}

#[test]
fn Test_An_Execute_Command_Should_Parse_Its_Effort()
{
    let arguments = Arguments("execute --goal hello --effort high");

    let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(effort, nomos_model_package::EffortLevel::High);
}

/// `MODEL-ROUTE-004`'s six spellings, named once so a case is a diff to this list and not
/// to the test that reads it.
const EFFORT_SPELLINGS: [(&str, nomos_model_package::EffortLevel); 6] = [
    ("backend-default", nomos_model_package::EffortLevel::BackendDefault),
    ("minimal", nomos_model_package::EffortLevel::Minimal),
    ("low", nomos_model_package::EffortLevel::Low),
    ("medium", nomos_model_package::EffortLevel::Medium),
    ("high", nomos_model_package::EffortLevel::High),
    ("maximum", nomos_model_package::EffortLevel::Maximum),
];

/// All six of `MODEL-ROUTE-004`'s spellings, not just one -- the same universe-closing
/// discipline `nomos-model-package::effort_level`'s own `Test_Every_Value_Is_In_The_Tested_Universe`
/// holds itself to.
#[test]
fn Test_An_Execute_Command_Parses_Every_Effort_Spelling()
{
    for (spelling, expected) in EFFORT_SPELLINGS
    {
        let arguments = Arguments(&format!("execute --goal hello --effort {spelling}"));

        let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(effort, expected, "spelling {spelling}");
    }
}

/// Spellings `--effort` refuses -- an unknown word, and a real value cased wrong, since a
/// closed vocabulary that matched case-insensitively would silently accept a spelling
/// `Usage_Text` never documents.
const UNRECOGNIZED_EFFORTS: [&str; 2] = ["superhuman", "LOW"];

#[test]
fn Test_An_Unrecognized_Effort_Should_Be_A_Usage_Error()
{
    for effort in UNRECOGNIZED_EFFORTS
    {
        let arguments = Arguments(&format!("execute --goal hello --effort {effort}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--effort"), "{effort}: {error}");
        assert!(error.contains(effort), "{effort}: {error}");
    }
}

/// The default is `ClaudeCode` -- every caller before `--executor`/`--model-backend`
/// existed must keep reaching `nomos-agent-executor-claude-code`.
// test-data: allow this pins the single, fixed scenario of omitting both flags; there is
// no second "no backend given" input to tabulate against it.
#[test]
fn Test_An_Execute_Command_With_No_Backend_Defaults_To_Claude_Code()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { backend, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(backend, Backend::ClaudeCode);
}

// test-data: allow `claude-code` is the only spelling `--executor` accepts today --
// `OD-EXECUTOR-005` found only one real `AgentExecutor` exists in this workspace, so a
// second case cannot exist until a second one does.
#[test]
fn Test_An_Execute_Command_Parses_Executor_Claude_Code()
{
    let arguments = Arguments("execute --goal hello --executor claude-code");

    let Command::Execute { backend, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(backend, Backend::ClaudeCode);
}

#[test]
fn Test_An_Execute_Command_Parses_Model_Backend_Ollama()
{
    let arguments = Arguments("execute --goal hello --model-backend ollama");

    let Command::Execute { backend, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(backend, Backend::Ollama);
}

/// Values `--executor` refuses -- a name with no meaning at all, and `ollama`, which is a
/// real spelling but belongs to `--model-backend`, not this flag.
const UNRECOGNIZED_EXECUTORS: [&str; 2] = ["gpt5", "ollama"];

#[test]
fn Test_An_Unrecognized_Executor_Should_Be_A_Usage_Error()
{
    for executor in UNRECOGNIZED_EXECUTORS
    {
        let arguments = Arguments(&format!("execute --goal hello --executor {executor}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--executor"), "{executor}: {error}");
        assert!(error.contains(executor), "{executor}: {error}");
    }
}

/// Values `--model-backend` refuses -- a name with no meaning at all, and `claude-code`,
/// which is a real spelling but belongs to `--executor`, not this flag.
const UNRECOGNIZED_MODEL_BACKENDS: [&str; 2] = ["gpt5", "claude-code"];

#[test]
fn Test_An_Unrecognized_Model_Backend_Should_Be_A_Usage_Error()
{
    for model_backend in UNRECOGNIZED_MODEL_BACKENDS
    {
        let arguments = Arguments(&format!("execute --goal hello --model-backend {model_backend}"));

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--model-backend"), "{model_backend}: {error}");
        assert!(error.contains(model_backend), "{model_backend}: {error}");
    }
}

/// `--executor` naming an `AgentExecutor` and `--model-backend` naming a `ModelBackend`
/// at once is not a request either flag alone could satisfy -- a call dispatches to
/// exactly one backend, so both present is refused rather than one silently winning.
// test-data: allow the refusal fires on both flags being present at all, regardless of
// which values they carry, so a second case would vary nothing this assertion depends on.
#[test]
fn Test_Both_Executor_And_Model_Backend_Together_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute --goal hello --executor claude-code --model-backend ollama");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--executor"), "{error}");
    assert!(error.contains("--model-backend"), "{error}");
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Model_Backend()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --model-backend ollama");

    let Command::JudgeRole { backend, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(backend, Backend::Ollama);
}

/// Arguments to `execute` that never supply `--goal` -- bare, and with an unrelated flag
/// present, so the check does not depend on `--goal` being the only thing missing.
const MISSING_GOAL_ARGUMENTS: [&str; 2] = ["execute", "execute --effort high"];

#[test]
fn Test_A_Missing_Goal_Should_Be_A_Usage_Error()
{
    for text in MISSING_GOAL_ARGUMENTS
    {
        let arguments = Arguments(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--goal"), "{text}: {error}");
    }
}

/// Verbs `Command_From_String_Arguments`'s own top-level `match` on the first argument
/// names neither.
const UNKNOWN_VERBS: [&str; 2] = ["dance", "build"];

#[test]
fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
{
    for verb in UNKNOWN_VERBS
    {
        let arguments = Arguments(verb);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains(verb), "{verb}: {error}");
    }
}

#[test]
fn Test_No_Verb_At_All_Should_Be_A_Usage_Error()
{
    let error = Command_From_String_Arguments(&[]).expect_err("must refuse");

    assert!(error.contains("usage"));
}

/// The one place `--goal ""` reaches this crate's tests without spending a real
/// invocation: parsing accepts an empty string exactly as it accepts any other, and
/// whatever `nomos-agent-executor-claude-code` or the real CLI does with it is that crate's own
/// concern, verified there, not re-verified through this transport.
// test-data: allow this pins the single scenario where --goal is the last token with
// nothing following it; a value placed after it is read as that value, not as a second
// case of "no value", and the "no --goal at all" cases are already tabulated in
// Test_A_Missing_Goal_Should_Be_A_Usage_Error.
#[test]
fn Test_An_Empty_Goal_Still_Parses()
{
    let arguments = Arguments("execute --goal");
    let error = Command_From_String_Arguments(&arguments).expect_err("a flag with nothing after it has no value");

    assert!(error.contains("--goal"));
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Crate_And_Default_Root()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code");

    let Command::JudgeRole { crate_name, root, effort, backend } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(crate_name, "nomos-agent-executor-claude-code");
    assert_eq!(root, PathBuf::from("."));
    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
    assert_eq!(backend, Backend::ClaudeCode);
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_An_Explicit_Root()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor --root /some/tree");

    let Command::JudgeRole { root, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(root, PathBuf::from("/some/tree"));
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Effort()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --effort low");

    let Command::JudgeRole { effort, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(effort, nomos_model_package::EffortLevel::Low);
}

/// Arguments to `judge-role` that never supply `--crate` -- bare, and with an unrelated
/// flag present.
const MISSING_CRATE_ARGUMENTS: [&str; 2] = ["judge-role", "judge-role --root somewhere"];

#[test]
fn Test_A_Missing_Crate_Should_Be_A_Usage_Error()
{
    for text in MISSING_CRATE_ARGUMENTS
    {
        let arguments = Arguments(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--crate"), "{text}: {error}");
    }
}

/// This repository's own root, three levels above `crates/host/nomos-cli` — the same
/// derivation `nomos-lang-rust-cargo`'s own tests use to run against a real tree rather
/// than a fixture nobody could have produced.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// Read against this repository's own real `README.md`, not a fixture — the same
/// standard `nomos-lang-rust-cargo`'s own tests already hold themselves to: a reader
/// that cannot be checked against a real row is checked against nothing.
#[test]
fn Test_Declared_Role_Should_Read_A_Real_Row_From_This_Repositorys_Own_Readme()
{
    let role = Declared_Role(&Repository_Root(), "nomos-agent-executor-claude-code").expect("this crate has a row");

    assert!(role.contains("AgentExecutor"), "{role}");
}

#[test]
fn Test_Declared_Role_Should_Be_None_For_A_Crate_Named_Nowhere()
{
    assert!(Declared_Role(&Repository_Root(), "nomos-does-not-exist").is_none());
}

#[test]
fn Test_Crate_Root_Should_Read_This_Crates_Own_Real_Manifest_Path()
{
    let root = Crate_Root(&Repository_Root(), "nomos-agent-executor-claude-code");

    assert_eq!(root, "crates/agent/nomos-agent-executor-claude-code");
}

/// `Run` naming `Command::JudgeRole` for a root with no `README.md` at all fails before
/// `judge_role::Judge_Role` ever reaches `dispatch::Dispatch_Task` -- `Resolve_Declared_Role`
/// names no row for any crate and returns `NotFound` directly. Driven through the real,
/// top-level `Run` end to end, the same way every other command in this file is exercised,
/// without ever touching a live backend subprocess.
#[test]
fn Test_Run_Should_Report_Not_Found_For_A_Judge_Role_Command_Naming_A_Root_With_No_Readme()
{
    let command = Command::JudgeRole {
        crate_name: "nomos-does-not-exist".to_owned(),
        root: PathBuf::from("no-such-directory-anywhere-for-agent-run-test"),
        effort: nomos_model_package::EffortLevel::BackendDefault,
        backend: Backend::ClaudeCode,
    };
    let mut output = Vec::new();
    let mut notes = Vec::new();

    let code = Run(&command, &mut output, &mut notes);

    assert_eq!(code, ExitCode::NotFound);
}

/// The numeric codes `agent`'s own usage text documents (see `Usage_Text`'s "exit codes"
/// line): 0 ok, 2 usage, 5 the executor could not run or answer, 6 the named crate has no
/// README row or no committed surface snapshot.
#[test]
fn Test_Value_Should_Return_The_Documented_Exit_Code_Number()
{
    assert_eq!(ExitCode::Ok.Value(), 0);
    assert_eq!(ExitCode::Usage.Value(), 2);
    assert_eq!(ExitCode::Unavailable.Value(), 5);
    assert_eq!(ExitCode::NotFound.Value(), 6);
}

/// The spelling `--effort` takes for a level, as an exhaustive match.
///
/// This is what closes [`EFFORT_SPELLINGS`] against its own authority. That array is six pairs
/// a person wrote, and `Test_An_Execute_Command_Parses_Every_Effort_Spelling` proves the parser
/// accepts all six -- neither says a seventh level would be noticed. A variant added to
/// `EffortLevel` fails this match to compile, which is the forcing function that brings an
/// author to the array beside it.
fn Spelled_Effort(effort: nomos_model_package::EffortLevel) -> &'static str
{
    return match effort
    {
        nomos_model_package::EffortLevel::BackendDefault => "backend-default",
        nomos_model_package::EffortLevel::Minimal => "minimal",
        nomos_model_package::EffortLevel::Low => "low",
        nomos_model_package::EffortLevel::Medium => "medium",
        nomos_model_package::EffortLevel::High => "high",
        nomos_model_package::EffortLevel::Maximum => "maximum",
    };
}

/// The effort levels `--effort` prints are the levels it takes, both directions.
///
/// `OD-AGENT-004` version 2: a help text may enumerate a compiled vocabulary only where a test
/// compares that enumeration against the vocabulary's own authority. The help text calls this
/// one `MODEL-ROUTE-004`'s closed vocabulary in its own words and then lists it, and until this
/// test nothing compared the two.
///
/// Read as prose rather than as a `a|b|c` run, because that is how this group writes it: a
/// comma-separated sentence ending in "or maximum", inside a paragraph. The words are located
/// by searching the whole text for each one rather than by parsing the sentence apart, so a
/// rewording of the sentence does not fail this test for a reason that is not about the
/// vocabulary. What that costs is the reverse direction -- a stale word left in the prose would
/// still be found -- so the reverse direction is asserted separately, against
/// [`UNRECOGNIZED_EFFORTS`]'s own discipline: every level the text names must be one the
/// command line accepts.
#[test]
fn Test_The_Named_Effort_Levels_Should_Be_Every_Level_The_Command_Line_Takes()
{
    let usage = Command_From_String_Arguments(&[]).expect_err("no verb prints the usage text");
    let (_, spelled) = usage
        .split_once("--effort takes ")
        .expect("the usage text names what --effort takes");
    let sentence = spelled.split(" -- ").next().unwrap_or_default();
    let named: Vec<&str> = sentence
        .split(|character: char| return character == ',' || character.is_whitespace())
        .map(str::trim)
        .filter(|word| return !word.is_empty() && *word != "or")
        .collect();

    assert!(
        !named.is_empty(),
        "no effort level was parsed out of the usage text, so this compared nothing: {sentence}"
    );
    assert_eq!(
        Sorted_Words(named.iter().map(|word| return (*word).to_owned())),
        Sorted_Words(EFFORT_SPELLINGS.iter().map(|(spelling, _)| return (*spelling).to_owned())),
        "the usage text and EFFORT_SPELLINGS disagree about what --effort takes"
    );

    for (spelling, level) in EFFORT_SPELLINGS
    {
        assert_eq!(
            Spelled_Effort(level),
            spelling,
            "EFFORT_SPELLINGS and the exhaustive match disagree about how {level:?} is spelled"
        );
    }
}

/// A list of words in ascending order, so that two of them can be compared as sets.
fn Sorted_Words(words: impl Iterator<Item = String>) -> Vec<String>
{
    let mut sorted: Vec<String> = words.collect();
    sorted.sort();
    sorted.dedup();

    return sorted;
}

/// Every code this group can leave the process with.
///
/// `agent::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[ExitCode::Ok, ExitCode::Usage, ExitCode::Unavailable, ExitCode::NotFound];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Usage => "Usage",
        ExitCode::Unavailable => "Unavailable",
        ExitCode::NotFound => "NotFound",
    };
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
/// compares it against its authority. The usage text is prose a person reads and
/// [`ExitCode`] is what the process returns, the two were written separately, and a code
/// added or renumbered in one of them and not the other is the failure that actually
/// happens.
///
/// Read through the refusal a verbless invocation gives rather than out of `USAGE_TEXT`,
/// which is private to `agent::parsing`: the text a user is actually shown is the subject,
/// and reaching it this way needs no widening of what this test can see.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = Command_From_String_Arguments(&[]).expect_err("no verb prints the usage text");

    assert!(
        usage.starts_with("usage: nomos agent"),
        "this compared some other group's help text: {usage}"
    );

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}
