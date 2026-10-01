//! `nomos guard install`'s scripts, run by real `git` with the built binary: they refuse a
//! commit and a push that carry an invented party's material, admit clean work, and still run
//! the repository's own hooks (`OD-POLICY-002` decision 6). Installed with two policies, they
//! refuse each party's material, and one policy standing aside silences no other (version 2).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

const POLICY: &str = r#"{
  "party": "Northwind",
  "identities": { "email_domains": ["northwind.example.invalid"] },
  "rules": [
    { "id": "party-name", "why": "names the party", "phrases": ["northwind"], "boundary": "anywhere" },
    { "id": "ticket", "why": "its tracker", "tickets": ["NW"] }
  ]
}"#;

/// A second invented party, unrelated to the first.
const SECOND_POLICY: &str = r#"{
  "party": "Fabrikam",
  "rules": [
    { "id": "second-party-name", "why": "names the second party", "phrases": ["fabrikam"], "boundary": "anywhere" }
  ]
}"#;

/// The first party's policy, declaring `repository` one of its own, so it stands aside there.
fn Policy_Owning(repository: &Path) -> String
{
    let path = repository.to_string_lossy().replace('\\', "/");
    return format!(
        r#"{{ "party": "Northwind", "rules": [ {{ "id": "party-name", "why": "names the party", "phrases": ["northwind"], "boundary": "anywhere" }} ],
            "own_repositories": {{ "paths": ["{path}"] }} }}"#
    );
}

fn Scratch(name: &str) -> PathBuf
{
    return std::env::temp_dir().join(format!("nomos-guard-hooks-{name}-{}", std::process::id()));
}

fn Git(directory: &Path, arguments: &[&str]) -> Output
{
    return Command::new("git").arg("-C").arg(directory).args(arguments).output().expect("git runs");
}

fn Git_Ok(directory: &Path, arguments: &[&str]) -> String
{
    let output = Git(directory, arguments);
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim().to_owned();
}

struct Setup
{
    repository: PathBuf,
    marker: PathBuf,
}

/// A repository whose `core.hooksPath` is the scripts installed for `policies`, holding one hook
/// of its own.
fn Guarded_Repository(name: &str, policies: &[String]) -> Setup
{
    let scratch = Scratch(name);
    let _ = std::fs::remove_dir_all(&scratch);
    let repository = scratch.join("repository");
    let hooks = scratch.join("hooks");
    std::fs::create_dir_all(&repository).expect("create repository");
    std::fs::create_dir_all(&hooks).expect("create hooks");
    let mut install = Command::new(NOMOS);
    install.args(["guard", "install", "--into"]).arg(&hooks);
    for (index, text) in policies.iter().enumerate()
    {
        let policy = scratch.join(format!("party-{index}.json"));
        std::fs::write(&policy, text).expect("write policy");
        install.arg("--policy").arg(&policy);
    }

    let installed = install.output().expect("nomos runs");
    assert!(installed.status.success(), "install: {}", String::from_utf8_lossy(&installed.stderr));

    Git_Ok(&repository, &["init", "-q", "-b", "main"]);
    Git_Ok(&repository, &["config", "user.name", "Me"]);
    Git_Ok(&repository, &["config", "user.email", "me@home.example.invalid"]);
    Git_Ok(&repository, &["config", "commit.gpgsign", "false"]);
    Git_Ok(&repository, &["config", "core.hooksPath", &hooks.to_string_lossy().replace('\\', "/")]);
    let marker = scratch.join("own-hook-ran");
    let own_hook = repository.join(".git").join("hooks").join("post-commit");
    std::fs::write(&own_hook, format!("#!/bin/sh\necho ran > '{}'\n", marker.to_string_lossy().replace('\\', "/"))).expect("write own hook");
    return Setup { repository, marker };
}

#[test]
fn Test_The_Installed_Hooks_Should_Refuse_Admit_And_Hand_Over_Through_Git()
{
    let setup = Guarded_Repository("commit", &[POLICY.to_owned()]);
    let repository = &setup.repository;

    std::fs::write(repository.join("clean.md"), "fine\n").expect("write");
    Git_Ok(repository, &["add", "."]);
    Git_Ok(repository, &["commit", "-q", "-m", "clean"]);
    assert!(setup.marker.exists(), "the repository's own post-commit hook still ran");

    std::fs::write(repository.join("leak.md"), "notes from Northwind\n").expect("write");
    Git_Ok(repository, &["add", "."]);
    let refused = Git(repository, &["commit", "-q", "-m", "leak"]);
    assert!(!refused.status.success(), "a commit adding the party's name was admitted");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("[party-name] leak.md:1"), "{}", String::from_utf8_lossy(&refused.stderr));

    Git_Ok(repository, &["reset", "-q", "leak.md"]);
    std::fs::write(repository.join("ok.md"), "fine too\n").expect("write");
    Git_Ok(repository, &["add", "ok.md"]);
    let message = Git(repository, &["commit", "-q", "-m", "fixes NW-12"]);
    assert!(!message.status.success(), "a message carrying a ticket key was admitted");
}

#[test]
fn Test_The_Installed_Push_Hook_Should_Refuse_What_Commit_Hooks_Were_Skipped_For()
{
    let setup = Guarded_Repository("push", &[POLICY.to_owned()]);
    let repository = &setup.repository;
    let remote = repository.with_file_name("remote.git");
    Git_Ok(repository.parent().expect("has a parent"), &["init", "-q", "--bare", &remote.to_string_lossy()]);
    Git_Ok(repository, &["remote", "add", "origin", &remote.to_string_lossy()]);

    std::fs::write(repository.join("clean.md"), "fine\n").expect("write");
    Git_Ok(repository, &["add", "."]);
    Git_Ok(repository, &["commit", "-q", "-m", "clean"]);
    Git_Ok(repository, &["push", "-q", "origin", "main"]);

    std::fs::write(repository.join("leak.md"), "Northwind\n").expect("write");
    Git_Ok(repository, &["add", "."]);
    Git_Ok(repository, &["commit", "-q", "--no-verify", "-m", "skipped the commit hooks"]);
    let pushed = Git(repository, &["push", "-q", "origin", "main"]);
    assert!(!pushed.status.success(), "a push carrying the party's name was admitted");
}

#[test]
fn Test_Hooks_Installed_With_Two_Policies_Should_Refuse_Each_Partys_Material()
{
    let setup = Guarded_Repository("two", &[POLICY.to_owned(), SECOND_POLICY.to_owned()]);
    let repository = &setup.repository;

    std::fs::write(repository.join("clean.md"), "fine\n").expect("write");
    Git_Ok(repository, &["add", "."]);
    Git_Ok(repository, &["commit", "-q", "-m", "clean"]);

    for (file, text, rule) in [("first.md", "notes from Northwind\n", "[party-name] first.md:1"), ("second.md", "notes from Fabrikam\n", "[second-party-name] second.md:1")]
    {
        std::fs::write(repository.join(file), text).expect("write");
        Git_Ok(repository, &["add", file]);
        let refused = Git(repository, &["commit", "-q", "-m", file]);
        assert!(!refused.status.success(), "a commit adding {file} was admitted");
        assert!(String::from_utf8_lossy(&refused.stderr).contains(rule), "{}", String::from_utf8_lossy(&refused.stderr));
        Git_Ok(repository, &["reset", "-q", file]);
    }
}

#[test]
fn Test_A_Policy_Standing_Aside_For_Its_Own_Repository_Should_Silence_No_Other()
{
    let repository = Scratch("aside").join("repository");
    let setup = Guarded_Repository("aside", &[Policy_Owning(&repository), SECOND_POLICY.to_owned()]);
    let repository = &setup.repository;

    std::fs::write(repository.join("own.md"), "notes from Northwind\n").expect("write");
    Git_Ok(repository, &["add", "own.md"]);
    Git_Ok(repository, &["commit", "-q", "-m", "the first party's own repository may name it"]);

    std::fs::write(repository.join("second.md"), "notes from Fabrikam\n").expect("write");
    Git_Ok(repository, &["add", "second.md"]);
    let refused = Git(repository, &["commit", "-q", "-m", "second"]);
    assert!(!refused.status.success(), "the first policy standing aside admitted the second party's material");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("[second-party-name] second.md:1"), "{}", String::from_utf8_lossy(&refused.stderr));
}
