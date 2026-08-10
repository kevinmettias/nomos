//! `nomos spec freshness`, driven the way a gate would drive it.
//!
//! `D-128` requires the maintained overview outputs to be freshness-validated.
//! `nomos-spec-project` has had the comparison since Phase 4 — a stamp, a sidecar and a
//! `Check` that separates *the store moved* from *somebody typed into the file* — and
//! until this command existed the only caller was that crate's own unit tests. A
//! detector nothing runs detects nothing, which is the shape `OD-GATE-001` records for
//! the corpus gates.
//!
//! Every assertion here runs the built binary against a build root of its own, because
//! the half of this that had never run is the half between `argv` and the exit code.
//!
//! # Why this file is not a corpus gate
//!
//! It names the corpus variable only to *remove* it from the child's environment, so a
//! machine holding a corpus behaves like a machine that does not. `domain-specification`
//! projects the governing records embedded in the binary, so every profile these tests
//! render builds with no corpus at all. The name is assembled with `concat!` for the
//! reason `tests/contract/tests/corpus_gates.rs` assembles it: the census resolves a gate
//! by finding a variable named in a test's reachable source, and a file that names one
//! without reading a corpus would inflate the hole rather than measure it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const V14: &str = concat!("NOMOS_", "V14_CORPUS");

/// A profile that renders from the embedded governing records and needs no corpus.
const EMBEDDED: &str = "domain-specification";

/// Where that profile puts its body, relative to a build root.
const EMBEDDED_BODY: &str = "spec/domain-specification.md";

/// The count `Catalogue::Shipped` carries, restated so the census assertions are legible.
const SHIPPED: usize = 14;

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

/// A build root holding exactly one rendered profile and its sidecar.
fn Rendered(name: &str) -> PathBuf
{
    let into = Scratch(name);
    let output = Nomos(&[
        "spec",
        "render",
        "--profile",
        EMBEDDED,
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "the fixture did not render: {}", Err_Text(&output));

    return into;
}

fn Freshness(into: &Path, extra: &[&str]) -> Output
{
    let mut arguments = vec!["spec", "freshness", "--into"];
    let destination = into.display().to_string();
    arguments.push(&destination);
    arguments.extend_from_slice(extra);

    return Nomos(&arguments);
}

#[test]
fn Test_A_Freshly_Rendered_Output_Should_Be_Current()
{
    let into = Rendered("freshness-current");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("is current"), "{said}");
    assert!(said.contains(&format!("checked 1 of {SHIPPED}")), "{said}");
}

/// The defect this command exists for.
///
/// The stamp records what the renderer produced. An edit changes the file and not the
/// stamp, so the two disagree — and until something asked them to agree, the disagreement
/// was a thing a reviewer might notice.
#[test]
fn Test_A_Hand_Edited_Output_Should_Be_Caught()
{
    let into = Rendered("freshness-edited");
    let body = into.join(EMBEDDED_BODY);
    let text = std::fs::read_to_string(&body).expect("reads the rendered body");
    std::fs::write(&body, format!("{text}\nsomebody typed this here\n")).expect("edits it");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 8, "an edited output was not caught: {}", Out_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("edited"), "{said}");
    assert!(!said.contains("stale:"), "the store did not change: {said}");
}

/// The escape hatch, closed.
///
/// A half-present pair is a failure rather than a skip. Treating a missing sidecar as
/// "nothing to check here" would make `rm` the documented way to silence this command,
/// and the first person to hit the check would learn it.
#[test]
fn Test_Deleting_The_Sidecar_Should_Not_Make_An_Edit_Invisible()
{
    let into = Rendered("freshness-unstamped");
    let body = into.join(EMBEDDED_BODY);
    let text = std::fs::read_to_string(&body).expect("reads the rendered body");
    std::fs::write(&body, format!("{text}\nsomebody typed this here\n")).expect("edits it");
    std::fs::remove_file(into.join(format!("{EMBEDDED_BODY}.nomos-projection.json")))
        .expect("removes the sidecar");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 8, "an unstamped output passed: {}", Out_Text(&output));
    assert!(Out_Text(&output).contains("nothing can say whether"), "{}", Out_Text(&output));
}

/// The other half of the pair, which is a different accident.
#[test]
fn Test_A_Sidecar_Whose_Output_Is_Gone_Should_Be_Reported()
{
    let into = Rendered("freshness-bodyless");
    std::fs::remove_file(into.join(EMBEDDED_BODY)).expect("removes the body");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    assert!(Out_Text(&output).contains("deleted or never written"), "{}", Out_Text(&output));
}

/// A run that examined nothing must not read like a run that found nothing wrong.
#[test]
fn Test_An_Empty_Build_Root_Should_Report_What_It_Did_Not_Check()
{
    let into = Scratch("freshness-empty");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains(&format!("checked 0 of {SHIPPED}")), "{said}");
    assert!(said.contains("not built here"), "{said}");
    assert!(said.contains(EMBEDDED), "the unbuilt profiles are not named: {said}");
}

/// Asking about one named output that is not there is a question with an answer.
///
/// Apart from the whole-root case above on purpose: a build root holding three of
/// fourteen profiles is ordinary, and a named profile that was never built is the caller
/// asking about a file that does not exist.
#[test]
fn Test_Asking_About_A_Profile_Nobody_Built_Should_Be_Not_Found()
{
    let into = Scratch("freshness-unasked");

    let output = Freshness(&into, &["--profile", EMBEDDED]);

    assert_eq!(Code(&output), 1, "{}", Out_Text(&output));
    assert!(Out_Text(&output).contains("nothing to compare"), "{}", Out_Text(&output));
}

#[test]
fn Test_An_Unknown_Profile_Should_Name_The_Ones_That_Exist()
{
    let into = Rendered("freshness-unknown");

    let output = Freshness(&into, &["--profile", "no-such-profile"]);

    assert_eq!(Code(&output), 1, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(notes.contains("no-such-profile"), "{notes}");
    assert!(notes.contains(EMBEDDED), "{notes}");
}

/// A single profile is checkable on its own, and the census says so.
#[test]
fn Test_One_Profile_Should_Be_Checkable_Without_The_Other_Thirteen()
{
    let into = Rendered("freshness-narrowed");

    let output = Freshness(&into, &["--profile", EMBEDDED]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("checked 1 of 1"), "{said}");
    assert!(!said.contains("not built here"), "{said}");
}

/// The negative control for the whole file.
///
/// Every assertion above would also pass against a command that answered "current" to
/// everything, so one of them has to be red for the right reason. This is that one: the
/// same rendered pair, with the *stamp* rewritten rather than the body, must come back
/// edited — proving the comparison reads the sidecar rather than trusting it.
#[test]
fn Test_A_Rewritten_Stamp_Should_Not_Excuse_The_File_It_Describes()
{
    let into = Rendered("freshness-restamped");
    let sidecar = into.join(format!("{EMBEDDED_BODY}.nomos-projection.json"));
    let stamp = std::fs::read_to_string(&sidecar).expect("reads the sidecar");
    let digest = stamp
        .lines()
        .find_map(|line| return line.trim().strip_prefix("\"content_digest\": \""))
        .and_then(|rest| return rest.strip_suffix("\","))
        .expect("the sidecar declares a content digest");
    std::fs::write(&sidecar, stamp.replace(digest, "blake3:0000000000000000"))
        .expect("rewrites the stamp");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    assert!(Out_Text(&output).contains("edited"), "{}", Out_Text(&output));
}
