//! `nomos guard install`'s scripts, run by real `git` with the built binary: they refuse a
//! commit and a push that carry an invented party's material, admit clean work, and still run
//! the repository's own hooks (`OD-POLICY-002` decision 6).

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

/// A repository whose `core.hooksPath` is the installed scripts, holding one hook of its own.
fn Guarded_Repository(name: &str) -> Setup
{
    let scratch = std::env::temp_dir().join(format!("nomos-guard-hooks-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let repository = scratch.join("repository");
    let hooks = scratch.join("hooks");
    std::fs::create_dir_all(&repository).expect("create repository");
    std::fs::create_dir_all(&hooks).expect("create hooks");
    let policy = scratch.join("party.json");
    std::fs::write(&policy, POLICY).expect("write policy");

    let installed = Command::new(NOMOS).args(["guard", "install", "--into"]).arg(&hooks).arg("--policy").arg(&policy).output().expect("nomos runs");
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
    let setup = Guarded_Repository("commit");
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
    let setup = Guarded_Repository("push");
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
