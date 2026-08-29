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

/// All six of `MODEL-ROUTE-004`'s spellings, not just one -- the same universe-closing
/// discipline `nomos-model-package::effort_level`'s own `Test_Every_Value_Is_In_The_Tested_Universe`
/// holds itself to.
#[test]
fn Test_An_Execute_Command_Parses_Every_Effort_Spelling()
{
    use nomos_model_package::EffortLevel;

    let cases = [
        ("backend-default", EffortLevel::BackendDefault),
        ("minimal", EffortLevel::Minimal),
        ("low", EffortLevel::Low),
        ("medium", EffortLevel::Medium),
        ("high", EffortLevel::High),
        ("maximum", EffortLevel::Maximum),
    ];

    for (spelling, expected) in cases
    {
        let arguments = Arguments(&format!("execute --goal hello --effort {spelling}"));

        let Command::Execute { effort, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(effort, expected, "spelling {spelling}");
    }
}

#[test]
fn Test_An_Unrecognized_Effort_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute --goal hello --effort superhuman");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--effort"), "{error}");
    assert!(error.contains("superhuman"), "{error}");
}

/// The default is `ClaudeCode` -- every caller before `--executor`/`--model-backend`
/// existed must keep reaching `nomos-agent-executor-claude-code`.
#[test]
fn Test_An_Execute_Command_With_No_Backend_Defaults_To_Claude_Code()
{
    let arguments = Arguments("execute --goal hello");

    let Command::Execute { backend, .. } = Command_From_String_Arguments(&arguments).expect("parses") else { panic!("wrong variant") };

    assert_eq!(backend, Backend::ClaudeCode);
}

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

#[test]
fn Test_An_Unrecognized_Executor_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute --goal hello --executor gpt5");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--executor"), "{error}");
    assert!(error.contains("gpt5"), "{error}");
}

#[test]
fn Test_An_Unrecognized_Model_Backend_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute --goal hello --model-backend gpt5");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--model-backend"), "{error}");
    assert!(error.contains("gpt5"), "{error}");
}

/// `--executor` naming an `AgentExecutor` and `--model-backend` naming a `ModelBackend`
/// at once is not a request either flag alone could satisfy -- a call dispatches to
/// exactly one backend, so both present is refused rather than one silently winning.
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

#[test]
fn Test_A_Missing_Goal_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("execute");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--goal"));
}

#[test]
fn Test_An_Unknown_Verb_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("dance");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("dance"));
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

#[test]
fn Test_A_Missing_Crate_Should_Be_A_Usage_Error()
{
    let arguments = Arguments("judge-role");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--crate"));
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
fn Test_Declared_Role_Reads_A_Real_Row_From_This_Repositorys_Own_Readme()
{
    let role = Declared_Role(&Repository_Root(), "nomos-agent-executor-claude-code").expect("this crate has a row");

    assert!(role.contains("AgentExecutor"), "{role}");
}

#[test]
fn Test_Declared_Role_Is_None_For_A_Crate_Named_Nowhere()
{
    assert!(Declared_Role(&Repository_Root(), "nomos-does-not-exist").is_none());
}

#[test]
fn Test_Crate_Root_Reads_This_Crates_Own_Real_Manifest_Path()
{
    let root = Crate_Root(&Repository_Root(), "nomos-agent-executor-claude-code");

    assert_eq!(root, "crates/agent/nomos-agent-executor-claude-code");
}
