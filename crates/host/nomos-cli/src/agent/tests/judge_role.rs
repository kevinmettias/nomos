//! `judge-role`'s own command line, and the two real reads it makes before it dispatches:
//! this repository's README row for a crate, and that crate's own manifest path.

use super::super::judge_role::{Crate_Root, Declared_Role};
use super::super::{Command, Command_From_String_Arguments, ExitCode, Run};
use super::Arguments;
use std::path::{Path, PathBuf};

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Model_Backend()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --model-backend ollama");

    let Command::JudgeRole { preferred, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(preferred.as_deref(), Some("ollama"));
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Crate_And_Default_Root()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code");

    let Command::JudgeRole { crate_name, root, effort, preferred } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(crate_name, "nomos-agent-executor-claude-code");
    assert_eq!(root, PathBuf::from("."));
    assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
    assert_eq!(preferred, None, "no backend flag was given, so nothing is preferred");
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_An_Explicit_Root()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor --root /some/tree");

    let Command::JudgeRole { root, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

    assert_eq!(root, PathBuf::from("/some/tree"));
}

#[test]
fn Test_A_Judge_Role_Command_Should_Parse_Its_Effort()
{
    let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --effort low");

    let Command::JudgeRole { effort, .. } = Command_From_String_Arguments(&arguments).expect("the arguments above are a command line this parser takes") else { panic!("wrong variant") };

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
        preferred: Some("claude-code".to_owned()),
    };
    let mut output = Vec::new();
    let mut notes = Vec::new();

    let code = Run(&command, &mut output, &mut notes);

    assert_eq!(code, ExitCode::NotFound);
}
