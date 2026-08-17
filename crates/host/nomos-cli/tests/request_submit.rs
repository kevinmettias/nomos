//! `nomos request submit` as a user meets it: the real binary, a real store, a real exit code.
//!
//! `OD-SPEC-009` says a transport constructs a submission, calls the one accept function, and
//! renders the verdict — nothing else. Every other test of `Accept_Submission` and `Validate`
//! calls a Rust function directly; this is the one that can fail if the verb is never wired
//! into `main`, if an exit code does not survive `u8::try_from(code).unwrap_or(1)`, or if the
//! group is spelled differently in the dispatcher than in its own usage text. `check_command.rs`
//! and `freshness.rs` are the precedent for testing a verb this way rather than only at the
//! library that backs it.
//!
//! # Why this cannot chain into a second invocation
//!
//! Nothing in this repository persists a specification database — `ARC-SPECDB-002` and
//! `OD-SPEC-006` are why. Each `Nomos(...)` call below assembles and discards its own store, so
//! a submission accepted by one call is gone before the next one starts. `--into` is tested
//! within the one invocation that created the submission, because that is the only invocation
//! in which the submission exists to be projected.
//!
//! The corpus variable is removed from every child so a machine holding the v14 corpus behaves
//! like one that does not — `nomos-spec-store`'s embedded governing records are all any of
//! these commands need, and citation resolution reads only the `submissions` table this run's
//! own store just wrote.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const V14: &str = concat!("NOMOS_", "V14_CORPUS");

fn Nomos(arguments: &[&str]) -> Output
{
    return Command::new(env!("CARGO_BIN_EXE_nomos"))
        .args(arguments)
        .env_remove(V14)
        .output()
        .expect("the binary runs");
}

fn Code(output: &Output) -> i32
{
    return output.status.code().unwrap_or(-1);
}

fn Out_Text(output: &Output) -> String
{
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

fn Err_Text(output: &Output) -> String
{
    return String::from_utf8_lossy(&output.stderr).into_owned();
}

/// A build root of this test's own, emptied first so a rerun cannot pass on a leftover.
fn Scratch(name: &str) -> PathBuf
{
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if path.exists()
    {
        std::fs::remove_dir_all(&path).expect("clears the build root");
    }
    std::fs::create_dir_all(&path).expect("creates the build root");

    return path;
}

/// A minimal, complete feature request: every universal field and every field
/// `SubmissionKind::FeatureRequest` requires, `OD-SPEC-010`'s tables applied.
fn Complete_Request_Fields(id: &str) -> Vec<&str>
{
    return vec![
        "request",
        "submit",
        "--kind",
        "feature-request",
        "--id",
        id,
        "--by",
        "kevin",
        "--field",
        "title=Items can be declined",
        "--field",
        "goal=close superseded work",
        "--field",
        "behaviour=a verb writes Declined",
        "--field",
        "acceptance=it stops being claimable",
        "--field",
        "invariants=none",
    ];
}

#[test]
fn Test_A_Complete_Request_Should_Be_Accepted()
{
    let output = Nomos(&Complete_Request_Fields("FR-CLI-001"));

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("accepted FR-CLI-001"), "{said}");
    assert!(said.contains("feature-request"), "{said}");
    assert!(said.contains("draft"), "a submission with no --state defaults to draft: {said}");
}

#[test]
fn Test_An_Incomplete_Submission_Should_Be_Refused_With_Exit_Nine()
{
    let mut arguments = Complete_Request_Fields("FR-CLI-002");
    // Drop the last two `--field` flags (`invariants` and its value), leaving the rest intact.
    arguments.truncate(arguments.len() - 2);

    let output = Nomos(&arguments);

    assert_eq!(Code(&output), 9, "{}", Out_Text(&output));
    let said = Err_Text(&output);
    assert!(said.contains("invariants"), "{said}");
    assert!(said.contains("nothing was stored"), "{said}");
}

#[test]
fn Test_A_Refusal_Should_Name_Every_Failing_Field_Rather_Than_The_First()
{
    let output = Nomos(&[
        "request",
        "submit",
        "--kind",
        "feature-request",
        "--id",
        "FR-CLI-003",
        "--by",
        "kevin",
        "--field",
        "title=only a title",
    ]);

    assert_eq!(Code(&output), 9, "{}", Out_Text(&output));
    let said = Err_Text(&output);
    for field in ["goal", "behaviour", "acceptance", "invariants"]
    {
        assert!(said.contains(field), "refusal did not name {field}: {said}");
    }
}

#[test]
fn Test_A_Missing_Required_Argument_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["request", "submit", "--id", "FR-CLI-004", "--by", "kevin"]);

    assert_eq!(Code(&output), 2, "{}", Out_Text(&output));
    assert!(Err_Text(&output).contains("--kind"), "{}", Err_Text(&output));
}

#[test]
fn Test_An_Unrecognised_Command_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["request", "nonsense"]);

    assert_eq!(Code(&output), 2, "{}", Out_Text(&output));
}

/// `OD-SPEC-013` made a submission a row in `nodes`; this is that row surviving the one
/// projection pass this run can take before its store is gone. `ARC-SPECDB-002` charges a
/// born-structured object with a freshness obligation on every committed projection — this is
/// the freshness stamp being produced at all, for an object that did not exist a moment before
/// this process started.
#[test]
fn Test_Into_Should_Render_The_Accepted_Submission_With_A_Freshness_Stamp()
{
    let into = Scratch("request-submit-into");
    let mut arguments = Complete_Request_Fields("FR-CLI-005");
    let destination = into.display().to_string();
    arguments.push("--into");
    arguments.push(&destination);

    let output = Nomos(&arguments);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("subject-dossier ->"), "{said}");
    assert!(said.contains("sidecar (.nomos-projection.json) ->"), "{said}");

    let body_path = into.join("subjects/FR-CLI-005/dossier.md");
    let sidecar_path = into.join("subjects/FR-CLI-005/dossier.md.nomos-projection.json");
    let body = std::fs::read_to_string(&body_path)
        .unwrap_or_else(|error| panic!("reads {}: {error}", body_path.display()));
    let sidecar = std::fs::read_to_string(&sidecar_path)
        .unwrap_or_else(|error| panic!("reads {}: {error}", sidecar_path.display()));

    assert!(body.contains("FR-CLI-005"), "{body}");
    assert!(body.contains("feature-request"), "{body}");

    // Not parsed as JSON: a dependency on `serde_json` in this crate's tests would be a second
    // reason to touch `Cargo.toml`, and the stamp's own round trip is already
    // `nomos-spec-project`'s to prove. What this run has to show is that the stamp names the
    // right profile and carries a digest, which a substring check answers just as honestly.
    assert!(sidecar.contains("\"profile\": \"subject-dossier\""), "{sidecar}");
    assert!(sidecar.contains("\"content_digest\":"), "{sidecar}");
}

/// A refused submission has nothing to project, so `--into` is never reached.
#[test]
fn Test_Into_Should_Write_Nothing_When_The_Submission_Is_Refused()
{
    let into = Scratch("request-submit-into-refused");
    let destination = into.display().to_string();

    let output = Nomos(&[
        "request",
        "submit",
        "--kind",
        "feature-request",
        "--id",
        "FR-CLI-006",
        "--by",
        "kevin",
        "--field",
        "title=only a title",
        "--into",
        &destination,
    ]);

    assert_eq!(Code(&output), 9, "{}", Out_Text(&output));
    assert!(
        std::fs::read_dir(&into).expect("the directory itself exists").next().is_none(),
        "a refused submission wrote something into {}",
        into.display()
    );
}
