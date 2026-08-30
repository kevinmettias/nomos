//! The real seam between `nomos_surface_provenance` and `nomos_platform` /
//! `nomos_platform_std`, proven from outside the crate rather than through the
//! private `fake_launcher::Scripted` every unit test in `src/` uses.
//!
//! `src/main.rs`'s own doc comment explains why those unit tests never run a real `git`:
//! `.github/workflows/gate.yml`'s checkout carries no `fetch-depth`, so this repository's
//! own history is not a fixture a CI run can promise. That is a good reason to keep the
//! join logic's own tests scripted -- it is not a reason the actual wiring should go
//! unproven. Nobody anywhere calls `nomos_platform_std::StdProcessLauncher` from a test,
//! and nobody proves `git.rs`'s commands are something a real `git` actually accepts, so
//! `main()`'s composition of the two could be broken and every `cargo test` in this crate
//! would still stay green.
//!
//! This file proves that wiring against a throwaway git repository each test creates and
//! owns, so it depends on no ambient history at all -- this repository's or otherwise.
//!
//! This crate builds only a binary (see `Cargo.toml`'s `[[bin]]`), so there is no library
//! target for an external test to link against; the compiled binary itself, run as a
//! subprocess, is the only public surface there is to test from outside. `nomos-cli`'s own
//! `tests/check_command.rs` establishes the same pattern in this workspace.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The binary under test, as cargo just built it.
const NOMOS_SURFACE_PROVENANCE: &str = env!("CARGO_BIN_EXE_nomos-surface-provenance");

/// A throwaway git repository, removed when the test ends.
struct Repository
{
    root: PathBuf,
}

impl Repository
{
    /// Initializes an empty repository under a fresh scratch directory named for the
    /// calling test, so a failure leaves an identifiable directory behind and concurrent
    /// tests never collide.
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-surface-provenance-it-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory under the temp directory");

        let repository = Self { root };
        repository.Git(&["init", "--quiet"]);
        // A commit needs an identity, and this fixture must not depend on one already
        // being configured wherever the gate happens to run.
        repository.Git(&["config", "user.email", "nomos-surface-provenance-tests@example.invalid"]);
        repository.Git(&["config", "user.name", "nomos-surface-provenance tests"]);

        return repository;
    }

    /// Writes `text` to `relative`, under the repository root, creating any parent
    /// directories it needs.
    fn Write(&self, relative: &str, text: &str) -> &Self
    {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("a fixture's parent directory");
        }
        std::fs::write(&path, text).expect("writing a fixture file");

        return self;
    }

    /// Stages everything written so far and commits it, returning the new commit's hash.
    fn Commit(&self, message: &str) -> String
    {
        self.Git(&["add", "-A"]);
        self.Git(&["commit", "--quiet", "-m", message]);

        return self.Git(&["rev-parse", "HEAD"]).trim().to_owned();
    }

    /// Runs `git` inside the repository, panicking with its stderr on failure and
    /// returning its stdout on success.
    fn Git(&self, arguments: &[&str]) -> String
    {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(&self.root)
            .output()
            .expect("git must be on PATH for this fixture to build itself");

        assert!(
            output.status.success(),
            "git {arguments:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        return String::from_utf8_lossy(&output.stdout).into_owned();
    }
}

impl Drop for Repository
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// What one run of the real binary exited with, and what it printed to each stream.
struct Ran
{
    code: i32,
    stdout: String,
    stderr: String,
}

/// Runs the real binary -- and through it, the real `StdProcessLauncher` this crate wires
/// into the real `nomos_platform::ProcessLauncher` contract -- against `repository`.
fn Run(repository: &Repository, arguments: &[&str]) -> Ran
{
    let mut full = vec!["--root".to_owned(), repository.root.display().to_string()];
    full.extend(arguments.iter().map(|argument| (*argument).to_owned()));

    let finished = Command::new(Path::new(NOMOS_SURFACE_PROVENANCE))
        .args(&full)
        .output()
        .expect("the binary cargo just built must be runnable");

    return Ran {
        code: finished.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&finished.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&finished.stderr).into_owned(),
    };
}

/// The happy-path flow across the boundary: a real git repository, a real `git diff` and
/// `git log` run through `nomos_platform_std::StdProcessLauncher`, and a real finding
/// rendered from exactly what they printed -- none of it scripted.
#[test]
fn Test_A_Real_Surface_Change_With_No_Records_Commit_Is_A_Finding()
{
    let repository = Repository::New("finding");
    repository.Write("tests/contract/surface/nomos-fixture.txt", "pub fn f();\n");
    let base = repository.Commit("base surface");

    repository.Write("tests/contract/surface/nomos-fixture.txt", "pub fn f();\npub fn g();\n");
    let changed = repository.Commit("reblessed, no design decided");

    let Ran { code, stdout, stderr } = Run(&repository, &["--since", &base, "--until", &changed]);

    assert_eq!(code, 0, "a finding is reported, never a failure: {stderr}");
    assert!(stdout.contains("FINDING nomos-fixture"), "{stdout}");
    assert!(stdout.contains("reblessed, no design decided"), "{stdout}");
}

/// The lifecycle the boundary imposes on the other side: a commit under `docs/records/`
/// in the same range is read back through the same real `git log` call, and suppresses
/// the finding the surface change alone would have produced.
#[test]
fn Test_A_Real_Records_Commit_In_Range_Suppresses_The_Finding()
{
    let repository = Repository::New("clean");
    repository.Write("tests/contract/surface/nomos-fixture.txt", "pub fn f();\n");
    let base = repository.Commit("base surface");

    repository
        .Write("tests/contract/surface/nomos-fixture.txt", "pub fn f();\npub fn g();\n")
        .Write("docs/records/OD-FIXTURE-001-a-decision-was-made.md", "# a decision\n");
    let changed = repository.Commit("real change, recorded");

    let Ran { code, stdout, stderr } = Run(&repository, &["--since", &base, "--until", &changed]);

    assert_eq!(code, 0, "{stderr}");
    assert!(!stdout.contains("FINDING"), "{stdout}");
    assert!(stdout.contains("checked and clean: nomos-fixture"), "{stdout}");
}

/// The error that crosses the boundary: a revision `git` itself refuses.
/// `src/evaluate.rs`'s own
/// `Test_Finding_For_Should_Fail_On_A_Bad_Revision_Rather_Than_Report_A_Finding` proves the
/// same refusal against a scripted launcher that never runs `git` at all; this proves it
/// reaches the real process exit code when a real `git diff` is the one that fails.
#[test]
fn Test_A_Real_Bad_Revision_Fails_The_Query_Rather_Than_Reporting_A_Finding()
{
    let repository = Repository::New("bad-revision");
    repository.Write("tests/contract/surface/nomos-fixture.txt", "pub fn f();\n");
    let base = repository.Commit("base surface");

    let Ran { code, stdout, .. } = Run(&repository, &["--since", &base, "--until", "no-such-revision-at-all"]);

    assert_eq!(code, 1, "a query `git` itself refuses must be QueryFailed, not a rendered report");
    assert!(stdout.is_empty(), "nothing is printed to stdout once a query has failed");
}
