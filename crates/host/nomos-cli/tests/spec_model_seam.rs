//! `nomos_cli` <-> `nomos_spec_model`, exercised through the compiled binary's public surface.
//!
//! `request.rs`'s inline `mod run_coverage`, `request/tests.rs` and `request/report.rs`'s
//! inline `mod tests` all build `nomos_spec_model::{Severity, SubmissionKind, SubmissionState,
//! Submission}` values and feed them to `nomos-cli`'s own private `request::Run`,
//! `Report_Accepted` and parsing functions -- proving `nomos_spec_model` works against those
//! private items, opened up, rather than against `nomos-cli`'s public surface. `nomos-cli` is
//! a `[[bin]]`-only crate (no `src/lib.rs`), so a file under `tests/` cannot reach any of
//! those functions at all; the only public surface left is the compiled binary itself, the
//! same surface `tests/request_submit.rs` already drives for this verb.
//!
//! `request/parsing.rs`'s `Parse_Kind` and `Parse_State` turn `--kind`/`--state` text into
//! `nomos_spec_model::SubmissionKind`/`SubmissionState` via their own `Parse`, and
//! `request/report.rs`'s `Report_Accepted` renders an accepted submission's `kind.Label()` and
//! `state.Label()` straight back out. This file constructs those same `nomos_spec_model` values
//! directly and checks the real binary's real stdout/stderr renders exactly what their own
//! public methods say -- not a hand-copied string that could drift from the type it is meant
//! to prove.

#[path = "support/mod.rs"]
mod support;

use nomos_spec_model::{SubmissionKind, SubmissionState};
use std::process::Command;

/// The same corpus variable `main.rs::Corpus_Request` reads. Removed from every child so a
/// machine holding the v14 corpus behaves like one that does not -- `tests/request_submit.rs`'s
/// own convention for this verb, kept here for the same reason: nothing this file asserts
/// should depend on whether a real corpus happens to be configured on the machine running it.
const V14_CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

/// Runs the real binary with the v14 corpus variable stripped from its environment.
fn Run_Without_Corpus(arguments: &[&str]) -> support::Ran
{
    let finished = Command::new(support::NOMOS)
        .args(arguments)
        .env_remove(V14_CORPUS_VARIABLE)
        .output()
        .expect("the binary cargo just built must be runnable");

    return support::Ran {
        code: finished.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&finished.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&finished.stderr).into_owned(),
    };
}

/// A complete feature request, submitted with `--kind` spelled from
/// `SubmissionKind::FeatureRequest.Label()` rather than a literal, must be accepted and must
/// report back exactly `kind.Label()` and `SubmissionState::Draft.Label()` -- the same pair
/// `Report_Accepted` renders from the `Submission` it was handed.
#[test]
fn Test_A_Complete_Submission_Should_Report_Its_Kind_And_State_As_Nomos_Spec_Models_Own_Labels()
{
    let kind = SubmissionKind::FeatureRequest;
    let state = SubmissionState::Draft;

    let outcome = Run_Without_Corpus(&[
        "request",
        "submit",
        "--kind",
        kind.Label(),
        "--id",
        "FR-SPEC-MODEL-SEAM-001",
        "--by",
        "seam-test",
        "--field",
        "title=Prove the nomos_spec_model seam",
        "--field",
        "goal=close a check-integration-coverage finding honestly",
        "--field",
        "behaviour=submit through the real binary and read nomos_spec_model's own labels back",
        "--field",
        "acceptance=the accepted line names exactly kind.Label() and state.Label()",
        "--field",
        "invariants=none",
    ]);

    assert_eq!(outcome.code, 0, "{}", outcome.stderr);
    assert!(
        outcome.stdout.contains(&format!("as {} ({})", kind.Label(), state.Label())),
        "{}",
        outcome.stdout
    );
}

/// A submission missing every required field but `title` must be refused, naming every field
/// `SubmissionKind::FeatureRequest.Required_Fields()` itself says this kind requires -- read
/// from the type's own public table rather than a list that could silently drift from it.
#[test]
fn Test_A_Submission_Missing_Its_Required_Fields_Should_Name_Every_One_Of_Nomos_Spec_Models_Own_Table()
{
    let kind = SubmissionKind::FeatureRequest;

    let outcome = Run_Without_Corpus(&[
        "request",
        "submit",
        "--kind",
        kind.Label(),
        "--id",
        "FR-SPEC-MODEL-SEAM-002",
        "--by",
        "seam-test",
        "--field",
        "title=only a title",
    ]);

    assert_eq!(outcome.code, 9, "{}", outcome.stdout);
    for field in kind.Required_Fields()
    {
        assert!(outcome.stderr.contains(field), "refusal did not name {field}: {}", outcome.stderr);
    }
}
