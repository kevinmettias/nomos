//! `nomos guard`'s parsing, and what it does before it ever asks git: no policy, or one that
//! cannot be used, is never read as clean.

use std::path::PathBuf;

use super::{ExitCode, GuardCommand, GuardContext, Guard_Command_From_String_Arguments, Run, Verb};

fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

/// A process environment that names no policy, whatever the host's own environment says.
fn Bare_Context() -> GuardContext
{
    return GuardContext { policy_from_environment: None, working_directory: PathBuf::from("."), executable: None, push_input: String::new() };
}

#[test]
fn Test_The_Hook_Verbs_Should_Read_Git_Arguments_After_The_Policy()
{
    let parsed = Guard_Command_From_String_Arguments(&Arguments("pre-push --policy p.json origin https://example.invalid/r.git")).expect("parses");
    assert_eq!(parsed, GuardCommand { verb: Verb::PrePush { remote: "origin".to_owned() }, policies: vec![PathBuf::from("p.json")] });
    let parsed = Guard_Command_From_String_Arguments(&Arguments("commit-msg --policy p.json .git/COMMIT_EDITMSG")).expect("parses");
    assert_eq!(parsed.verb, Verb::CommitMessage { file: PathBuf::from(".git/COMMIT_EDITMSG") });
    assert_eq!(Guard_Command_From_String_Arguments(&Arguments("scan")).expect("parses").verb, Verb::Scan { root: PathBuf::from(".") });
}

#[test]
fn Test_A_Repeated_Policy_Should_Be_Kept_In_Order_Rather_Than_Overwritten()
{
    let parsed = Guard_Command_From_String_Arguments(&Arguments("scan --policy first.json --root r --policy second.json")).expect("parses");
    assert_eq!(parsed, GuardCommand { verb: Verb::Scan { root: PathBuf::from("r") }, policies: vec![PathBuf::from("first.json"), PathBuf::from("second.json")] });
}

#[test]
fn Test_A_Malformed_Command_Should_Be_Refused_As_Usage()
{
    for text in ["", "push", "commit-msg --policy p.json", "pre-push", "install", "scan --root", "scan --verbose"]
    {
        assert!(Guard_Command_From_String_Arguments(&Arguments(text)).is_err(), "should be usage: {text:?}");
    }
}

#[test]
fn Test_No_Policy_Should_Refuse_A_Transition_And_Judge_Nothing_In_A_Scan()
{
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let commit = GuardCommand { verb: Verb::PreCommit, policies: Vec::new() };
    assert_eq!(Run(&commit, &Bare_Context(), &mut stdout, &mut stderr), ExitCode::Unusable, "a commit with no policy is refused, not passed");
    let scan = GuardCommand { verb: Verb::Scan { root: PathBuf::from(".") }, policies: Vec::new() };
    assert_eq!(Run(&scan, &Bare_Context(), &mut stdout, &mut stderr), ExitCode::NothingJudged, "a scan with no policy judged nothing");
}

#[test]
fn Test_A_Policy_That_Cannot_Be_Used_Should_Refuse_The_Transition()
{
    let path = std::env::temp_dir().join(format!("nomos-guard-broken-policy-{}.json", std::process::id()));
    std::fs::write(&path, r#"{ "party": "Northwind", "rules": [ { "id": "a", "why": "w", "phrase": ["x"] } ] }"#).expect("write policy");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let commit = GuardCommand { verb: Verb::PreCommit, policies: vec![path.clone()] };
    assert_eq!(Run(&commit, &Bare_Context(), &mut stdout, &mut stderr), ExitCode::Unusable);
    assert!(String::from_utf8_lossy(&stderr).contains("cannot be used"), "the reason is given");
    let missing = GuardCommand { verb: Verb::PreCommit, policies: vec![path.with_extension("absent")] };
    assert_eq!(Run(&missing, &Bare_Context(), &mut stdout, &mut stderr), ExitCode::Unusable, "a named policy that is missing refuses");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn Test_One_Unusable_Policy_Among_Several_Should_Refuse_Before_Anything_Is_Judged()
{
    let usable = std::env::temp_dir().join(format!("nomos-guard-usable-policy-{}.json", std::process::id()));
    std::fs::write(&usable, r#"{ "party": "Northwind", "rules": [ { "id": "a", "why": "w", "phrases": ["northwind"] } ] }"#).expect("write policy");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    for policies in [vec![usable.clone(), usable.with_extension("absent")], vec![usable.with_extension("absent"), usable.clone()]]
    {
        let commit = GuardCommand { verb: Verb::PreCommit, policies };
        assert_eq!(Run(&commit, &Bare_Context(), &mut stdout, &mut stderr), ExitCode::Unusable, "a missing policy refuses wherever it is named");
    }
    assert!(!String::from_utf8_lossy(&stderr).contains("REFUSED"), "nothing was judged before the missing policy refused");
    let _ = std::fs::remove_file(&usable);
}
