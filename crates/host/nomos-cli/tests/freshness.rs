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

/// The profile this repository commits and therefore requires.
///
/// It reads only the relations between the embedded governing records, so it builds on a
/// runner holding no corpus — which is the property that lets the gate require it at all.
const REQUIRED: &str = "diagram-set";

/// Where that profile puts its body, relative to a build root.
const REQUIRED_BODY: &str = "diagrams/relations.mmd";

/// How many profiles the catalogue ships.
///
/// Read from the catalogue rather than typed here. It was a literal `14` until four
/// subject-addressed profiles arrived and turned the census line into `of 18`, and a number
/// restated in a test is a mirror of something checked elsewhere — the failure it produces
/// says the census is wrong when what is wrong is the copy.
fn Shipped_Count() -> usize
{
    return nomos_spec_project::SHIPPED.len();
}

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
    return Rendered_As(name, EMBEDDED);
}

/// The same, for a named profile.
fn Rendered_As(name: &str, profile: &str) -> PathBuf
{
    let into = Scratch(name);
    let output = Nomos(&[
        "spec",
        "render",
        "--profile",
        profile,
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
    assert!(said.contains(&format!("checked 1 of {}", Shipped_Count())), "{said}");
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
    assert!(said.contains(&format!("checked 0 of {}", Shipped_Count())), "{said}");
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

// ---------------------------------------------------------------------------------------
// Required publications.
//
// Everything above answers "what is here, and is what is here current". A gate needs the
// other question — "is everything this repository promises to ship here and current" — and
// the two have opposite defaults. A build root holding a subset is ordinary; a *promised*
// output that is absent is a failure. `--require` is the difference, and it is a property
// of the repository doing the promising rather than of the profile, which describes how a
// projection is built and not whether anybody ships it.

/// The hole this flag closes.
///
/// Without it, deleting both halves of a governed output moves it to `not built here` and
/// the run still exits zero — so the way to silence a drifted publication is to delete it,
/// which is worse than the edit the rest of this file catches.
#[test]
fn Test_A_Required_Output_That_Was_Never_Built_Should_Fail()
{
    let into = Scratch("required-never-built");

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 8, "a promised output was absent and nobody minded: {}", Out_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains(REQUIRED_BODY), "the missing output is not named: {said}");
    assert!(said.contains("required and not current"), "{said}");
}

/// The default is unchanged, and that is the point.
///
/// A scratch build root legitimately holds three of fourteen profiles. If requiredness
/// leaked into the ordinary run, every developer rendering one profile would get a red
/// answer about thirteen they never asked for.
#[test]
fn Test_An_Unrequired_Output_That_Was_Never_Built_Should_Still_Pass()
{
    let into = Scratch("required-absent-optional");

    let output = Freshness(&into, &[]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains(REQUIRED), "{said}");
    assert!(!said.contains("required and"), "nothing was required: {said}");
}

#[test]
fn Test_A_Required_Output_Missing_Its_Sidecar_Should_Fail()
{
    let into = Rendered_As("required-unstamped", REQUIRED);
    std::fs::remove_file(into.join(format!("{REQUIRED_BODY}.nomos-projection.json")))
        .expect("removes the sidecar");

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    assert!(Out_Text(&output).contains("required and not current"), "{}", Out_Text(&output));
}

#[test]
fn Test_A_Required_Sidecar_Missing_Its_Output_Should_Fail()
{
    let into = Rendered_As("required-bodiless", REQUIRED);
    std::fs::remove_file(into.join(REQUIRED_BODY)).expect("removes the body");

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    assert!(Out_Text(&output).contains("required and not current"), "{}", Out_Text(&output));
}

/// A requirement is kept by a *current* output, not by a present one.
///
/// The summary line is what this asserts. An edited body already fails on its own line, and
/// a summary that went on calling the requirement met would be the same vacuous success one
/// level up.
#[test]
fn Test_A_Required_Output_That_Was_Edited_Should_Not_Count_As_Kept()
{
    let into = Rendered_As("required-edited", REQUIRED);
    let body = into.join(REQUIRED_BODY);
    let text = std::fs::read_to_string(&body).expect("reads the rendered body");
    std::fs::write(&body, format!("{text}\n%% somebody typed this here\n")).expect("edits it");

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("edited"), "{said}");
    assert!(said.contains("required and not current"), "{said}");
    assert!(!said.contains("required and current"), "an edited output was reported kept: {said}");
}

/// A stamp built over inputs the store no longer holds.
///
/// Simulated by rewriting the recorded inputs rather than by moving the store, because the
/// governing records are compiled into the binary under test. What is asserted is that the
/// requirement reads the store-side half of the comparison too, so a publication whose
/// inputs moved is not kept alive by a body that still matches its own digest.
#[test]
fn Test_A_Required_Output_Built_Over_Other_Inputs_Should_Not_Count_As_Kept()
{
    let into = Rendered_As("required-stale-inputs", REQUIRED);
    let sidecar = into.join(format!("{REQUIRED_BODY}.nomos-projection.json"));
    let stamp = std::fs::read_to_string(&sidecar).expect("reads the sidecar");
    let digest = stamp
        .lines()
        .find_map(|line| return line.trim().strip_prefix("\"inputs_digest\": \""))
        .and_then(|rest| return rest.strip_suffix("\","))
        .expect("the sidecar declares an inputs digest");
    std::fs::write(&sidecar, stamp.replace(digest, "sha256:0000000000000000"))
        .expect("rewrites the recorded inputs");

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 8, "{}", Out_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains("stale:"), "{said}");
    assert!(said.contains("required and not current"), "{said}");
}

/// The green case has to say what it enforced.
///
/// A gate step whose successful output does not name the outputs it guaranteed reads
/// exactly like one that guaranteed nothing, which is the defect the whole file is against.
#[test]
fn Test_A_Kept_Requirement_Should_Name_Itself()
{
    let into = Rendered_As("required-kept", REQUIRED);

    let output = Freshness(&into, &["--require", REQUIRED]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let said = Out_Text(&output);
    assert!(said.contains(&format!("required and current: {REQUIRED}")), "{said}");
}

/// An unknown requirement is a question about a profile, not an answer about a file.
///
/// Reporting `diagram-sett` as a missing output would send a reader looking for a path that
/// was never nameable. The catalogue already knows how to refuse an identifier by listing
/// the ones that exist, and resolving before reading disk is what routes it there.
#[test]
fn Test_An_Unknown_Requirement_Should_Be_Refused_Rather_Than_Reported_Missing()
{
    let into = Rendered_As("required-unknown", REQUIRED);

    let output = Freshness(&into, &["--require", "no-such-profile"]);

    assert_eq!(Code(&output), 1, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(notes.contains("no-such-profile"), "{notes}");
    assert!(notes.contains(REQUIRED), "the profiles that do exist are not named: {notes}");
    assert!(!Out_Text(&output).contains("required and not current"), "{}", Out_Text(&output));
}

/// A requirement the run would never have looked at is a contradiction, not a pass.
#[test]
fn Test_A_Requirement_Outside_The_Examined_Profile_Should_Be_Refused()
{
    let into = Rendered_As("required-unexamined", REQUIRED);

    let output = Freshness(&into, &["--profile", EMBEDDED, "--require", REQUIRED]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("never looked for it"), "{}", Err_Text(&output));
}
