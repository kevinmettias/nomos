//! Band 91 — `nomos-surface-provenance`.
//!
//! A report, run by a human on demand, over exactly the join `docs/records/
//! OD-STORE-002-a-derived-identity-already-excludes-revision-and-that-is-the-key-a-series-
//! needs.md` names in its Worked Case: for a crate `C` and a commit range, does the blob
//! at `tests/contract/surface/C.txt` differ between the range's endpoints, and does any
//! commit in that same range touch `docs/records/`? A yes to the first and no to the
//! second is printed as a finding — a candidate for a reader to judge, never a verdict
//! this tool renders on its own.
//!
//! # Why this is a report and never a gate
//!
//! `OD-STORE-002`'s own "What A Temporal Claim Is Evidence Of, And What It Is Not"
//! section states the rule this binary is built to obey: "a temporal claim must never
//! gate the change that produced the observation the trend depends on." A finding here
//! is computed from commits already accepted into history — never from the change a
//! caller is proposing right now — so it is `EvidenceClass::Derived` at best, and
//! `Derived` evidence is read "alongside a live check, never substituted for one, and
//! never wired to a mechanism — a gate, a lint, an auto-block." Concretely: this process
//! exits `0` whenever it finished running, whether or not [`report::Render`] printed a
//! finding, and nothing in `.github/workflows/gate.yml` or any other crate's `cargo test`
//! invokes it.
//!
//! # Why the range is always given, never inferred
//!
//! A survey of this repository's own history found that read at single-commit
//! granularity — the range from a commit's parent to itself — 21 of the 59 commits that
//! ever touched `tests/contract/surface/` (36%) touch zero paths under `docs/records/` in
//! that same commit, including a rebless-only commit
//! (`27b3ed94794dd68cf4b19de2209ee1d985bb3fe8`) that was correct precisely because it
//! carried no decision to record. `OD-STORE-002` leaves the choice of range to the
//! caller rather than deciding a default that would paper over that class — so `--since`
//! and `--until` are required here, and a reader who wants a small, legible range (a
//! feature branch, a session's commits) supplies one rather than this tool guessing one
//! on their behalf.
//!
//! # Why the queries never touch this repository's own checkout during `cargo test`
//!
//! `.github/workflows/gate.yml`'s `actions/checkout` step sets no `fetch-depth`, so a CI
//! run's clone is not guaranteed to hold this repository's full history — the one input
//! every query here depends on. `evaluate.rs`'s unit tests exercise the join logic
//! against [`fake_launcher::Scripted`], which answers by matching a fragment of the argv
//! it was asked to run rather than by running `git` at all, so `cargo test -p
//! nomos-surface-provenance` proves the join correct without depending on how much
//! history the checkout that ran it happens to hold.

#![forbid(unsafe_code)]

mod arguments;
mod discovery;
mod evaluate;
#[cfg(test)]
mod fake_launcher;
mod git;
mod report;

mod exit_code;

use exit_code::ExitCode;
use nomos_platform::ProcessLauncher;
use nomos_platform_std::StdProcessLauncher;

fn main() -> std::process::ExitCode
{
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let code = Run(&arguments, &StdProcessLauncher, &mut stdout, &mut stderr);

    return std::process::ExitCode::from(u8::try_from(code.Value()).unwrap_or(1));
}

/// Parses the command line, runs every query it names through `launcher`, and renders
/// what came back.
///
/// Generic over [`ProcessLauncher`] and over `impl Write` for the same reason:
/// [`main`] is the only caller that needs a real `git` and a real `stdout`, and a test
/// that wants either replaced should not have to reach through a process boundary to do
/// it — the same shape `nomos-cli::check::Run` already uses for the `Write` half.
fn Run(
    arguments: &[String],
    launcher: &impl ProcessLauncher,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> ExitCode
{
    let parsed = match self::arguments::Parse(arguments)
    {
        Ok(parsed) => parsed,
        Err(message) =>
        {
            let _ = writeln!(stderr, "{message}");
            return ExitCode::Usage;
        }
    };

    let known = match discovery::Every_Snapshotted_Crate(&parsed.root)
    {
        Ok(known) => known,
        Err(message) =>
        {
            let _ = writeln!(stderr, "{message}");
            return ExitCode::Usage;
        }
    };

    let selected: Vec<String> = if parsed.crates.is_empty()
    {
        known
    }
    else
    {
        let unknown: Vec<&String> = parsed.crates.iter().filter(|name| !known.contains(name)).collect();
        if !unknown.is_empty()
        {
            let _ = writeln!(
                stderr,
                "these --crate names have no tests/contract/surface/<name>.txt: {unknown:?}"
            );
            return ExitCode::Usage;
        }
        parsed.crates
    };

    let records_touched = match evaluate::Records_Touched(launcher, &parsed.root, &parsed.since, &parsed.until)
    {
        Ok(touched) => touched,
        Err(message) =>
        {
            let _ = writeln!(stderr, "{message}");
            return ExitCode::QueryFailed;
        }
    };

    let mut findings = Vec::new();
    for krate in &selected
    {
        match evaluate::Finding_For(launcher, &parsed.root, &parsed.since, &parsed.until, krate, records_touched)
        {
            Ok(finding) => findings.push(finding),
            Err(message) =>
            {
                let _ = writeln!(stderr, "{krate}: {message}");
                return ExitCode::QueryFailed;
            }
        }
    }

    let _ = write!(stdout, "{}", report::Render(&parsed.since, &parsed.until, &findings));

    return ExitCode::Ok;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use fake_launcher::Scripted;

    /// A usage error is rendered and exits `2` before any query is attempted — proven
    /// with no launcher scripted at all, since none should be asked to run anything.
    #[test]
    fn Test_A_Missing_Since_Is_A_Usage_Error_Before_Anything_Runs()
    {
        let launcher = Scripted::New();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&["--until".to_owned(), "HEAD".to_owned()], &launcher, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Usage);
        assert!(stdout.is_empty());
    }

    /// `evaluate.rs` and `report.rs` each prove their own halves of the join in
    /// isolation; this proves the seam between them and `main.rs`'s own argument and
    /// discovery handling: a finding computed from a scripted launcher's answers reaches
    /// the rendered text `Run` prints, and the exit code stays `0` regardless of a
    /// finding existing.
    #[test]
    fn Test_A_Finding_Reaches_The_Rendered_Report_And_Still_Exits_Ok()
    {
        let root = std::env::temp_dir().join(format!("nomos-surface-provenance-test-{}", std::process::id()));
        let surface = root.join("tests").join("contract").join("surface");
        std::fs::create_dir_all(&surface).expect("test needs a directory");
        std::fs::write(surface.join("nomos-model.txt"), "pub fn f();\n").expect("test needs a file");

        let launcher = Scripted::New()
            .Answer("log a..b --format=%H --", 0, "", "")
            .Answer("diff a b --name-only", 0, "tests/contract/surface/nomos-model.txt\n", "")
            .Answer("log a..b --format=%H\t%s --", 0, "deadbeef\treblessed\n", "");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let arguments = [
            "--since".to_owned(),
            "a".to_owned(),
            "--until".to_owned(),
            "b".to_owned(),
            "--root".to_owned(),
            root.to_string_lossy().into_owned(),
        ];

        let code = Run(&arguments, &launcher, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Ok, "stderr: {}", String::from_utf8_lossy(&stderr));
        let text = String::from_utf8(stdout).expect("report is text");
        assert!(text.contains("FINDING nomos-model"));
        assert!(text.contains("deadbeef"));

        let _ = std::fs::remove_dir_all(&root);
    }

    /// An unknown `--crate` name is refused rather than silently reporting nothing for
    /// it — the same "a check that cannot find its subject must fail loudly" stance
    /// `tests/contract/tests/boundaries/graph.rs` states for the workspace-wide suite.
    #[test]
    fn Test_An_Unknown_Crate_Name_Is_A_Usage_Error()
    {
        let root = std::env::temp_dir().join(format!("nomos-surface-provenance-test-unknown-{}", std::process::id()));
        let surface = root.join("tests").join("contract").join("surface");
        std::fs::create_dir_all(&surface).expect("test needs a directory");
        std::fs::write(surface.join("nomos-model.txt"), "pub fn f();\n").expect("test needs a file");

        let launcher = Scripted::New();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let arguments = [
            "--since".to_owned(),
            "a".to_owned(),
            "--until".to_owned(),
            "b".to_owned(),
            "--root".to_owned(),
            root.to_string_lossy().into_owned(),
            "--crate".to_owned(),
            "no-such-crate".to_owned(),
        ];

        let code = Run(&arguments, &launcher, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Usage);
        assert!(String::from_utf8_lossy(&stderr).contains("no-such-crate"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
