//! The `nomos spec` group, driven the way a person drives it.
//!
//! Every assertion here runs the built binary. That is the point of the file: the library
//! under it has been covered since Phase 1, and the part that had never run once was
//! argument parsing — the dispatch from `argv` to a command, the flags, the exit codes and
//! which stream each answer goes to. A test that calls `spec::Run` directly proves none of
//! that, because it is the caller that assembled the command.
//!
//! # Why this file is not a corpus gate
//!
//! It names the corpus variable, and it is not one of the sixty-eight tests
//! `OD-GATE-001` counts. Nothing here reads the v14 corpus: the variable is *removed*
//! from the child's environment so that a machine which has one behaves like a machine
//! which does not, and where a corpus is wanted this file builds a small one of its own
//! under `CARGO_TARGET_TMPDIR`. So these tests run everywhere and skip nothing.
//!
//! The name is therefore assembled with `concat!`, exactly as
//! `tests/contract/tests/corpus_gates.rs` assembles it for the same reason — the census
//! resolves a gate by finding a variable named in a test's reachable source, and a file
//! that names one without reading a corpus would inflate the hole rather than measure it.
//! Spelling it out here and adding an exemption to the census was the alternative, and an
//! exemption is the easiest place in a check to hide something.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const V14: &str = concat!("NOMOS_", "V14_CORPUS");

/// The binary, with a corpus environment this test decides rather than inherits.
///
/// Without the removal these assertions would pass or fail according to whether the
/// machine running them happens to hold a corpus, which is the one thing a test about
/// absence must not depend on.
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

fn Err_Text(output: &Output) -> String
{
    return String::from_utf8_lossy(&output.stderr).into_owned();
}

fn Out_Text(output: &Output) -> String
{
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// A directory of this test's own, emptied first so a rerun cannot pass on a leftover.
fn Scratch(name: &str) -> PathBuf
{
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if path.exists()
    {
        std::fs::remove_dir_all(&path).expect("clears the scratch directory");
    }
    std::fs::create_dir_all(&path).expect("creates the scratch directory");

    return path;
}

/// A corpus with one volume, one table in it, and the two machine files empty.
///
/// Small on purpose. The real corpus's numbers are asserted by the ingest tests that hold
/// it; what is asserted here is that the command line reaches it at all.
fn Fixture_Corpus(name: &str) -> PathBuf
{
    let root = Scratch(name);
    let volumes = root.join("01_authoring/domain_volumes");
    let lineage = root.join("01_authoring/source_lineage");
    let catalog = root.join("02_machine/catalog");

    for directory in [&volumes, &lineage, &catalog]
    {
        std::fs::create_dir_all(directory).expect("creates a corpus directory");
    }

    std::fs::write(
        volumes.join("05_domain_model.md"),
        "# Canonical domain model\n\
         \n\
         | Model | Responsibility |\n\
         | --- | --- |\n\
         | WorkspaceContext | Repository, branch, snapshot, trust context. |\n\
         | Capability | A named operation with guarantees and a provider. |\n",
    )
    .expect("writes a volume");
    std::fs::write(lineage.join("normative-source-statements.yaml"), "statements: []\n")
        .expect("writes the statement file");
    std::fs::write(catalog.join("catalog.json"), "[]\n").expect("writes the catalog");

    return root;
}

/// Phase 2's payoff, from a terminal: what did this record say?
///
/// Byte for byte against the file the store was seeded from. Not "contains the title" —
/// the whole claim of a content-addressed store is that what comes out is what went in,
/// and an assertion over a substring would hold just as well if the middle of the record
/// were missing.
#[test]
fn Test_A_Record_Should_Come_Out_Of_The_Binary_Byte_For_Byte()
{
    let path = Repository_Root().join("docs/records/OD-GATE-001-a-skipped-test-reports-ok.md");
    let expected = std::fs::read(&path).expect("reads the record from disk");

    let output = Nomos(&["spec", "record", "--id", "OD-GATE-001"]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert_eq!(
        output.stdout,
        expected,
        "stdout is not the record's bytes; {} bytes out, {} bytes on disk",
        output.stdout.len(),
        expected.len()
    );
    assert!(
        Err_Text(&output).contains("sha256:"),
        "the content address belongs on stderr, beside the answer rather than inside it"
    );
}

/// Everything about the answer goes to the other stream, so a redirect is exact.
#[test]
fn Test_Nothing_But_The_Record_Should_Reach_Standard_Output()
{
    let output = Nomos(&["spec", "record", "--id", "D-132"]);

    let text = Out_Text(&output);
    assert!(text.starts_with("---\nid: D-132\n"), "{:?}", text.get(..60));
    assert!(
        !text.contains("absent:"),
        "the absence note reached stdout, so a redirect would write it into the record"
    );
    assert!(Err_Text(&output).contains("absent:"), "and it must still be reported");
}

/// The other half of Phase 2's question: the real rows.
#[test]
fn Test_A_Table_Should_Come_Out_As_The_Rows_That_Were_Authored()
{
    let corpus = Fixture_Corpus("table-rows");

    let output = Nomos(&[
        "spec",
        "table",
        "--document",
        "05_domain_model.md",
        "--corpus",
        &corpus.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert_eq!(
        Out_Text(&output),
        "| Model | Responsibility |\n\
         | --- | --- |\n\
         | WorkspaceContext | Repository, branch, snapshot, trust context. |\n\
         | Capability | A named operation with guarantees and a provider. |\n"
    );
    assert!(
        Err_Text(&output).contains("4 pipe line(s)"),
        "the census belongs beside the rows: {}",
        Err_Text(&output)
    );
}

/// A corpus that is not there is an absence naming what was expected — never an empty
/// answer. This is `OD-GATE-001`'s finding one level up, and it is the reason the exit
/// code is its own: a caller told the identifier is unknown corrects the identifier, and
/// a caller told the corpus is absent configures a corpus.
#[test]
fn Test_An_Absent_Corpus_Should_Be_Named_Rather_Than_Answered_Empty()
{
    let output = Nomos(&["spec", "table", "--document", "05_domain_model.md"]);

    assert_eq!(Code(&output), 6, "{}", Err_Text(&output));
    assert!(
        Out_Text(&output).is_empty(),
        "an absent corpus produced content: {}",
        Out_Text(&output)
    );

    let notes = Err_Text(&output);
    assert!(notes.contains(V14), "the absence must name how to supply one: {notes}");
    assert!(notes.contains("01_authoring/domain_volumes"), "{notes}");
    assert!(notes.contains("02_machine/catalog/catalog.json"), "{notes}");
    assert!(notes.contains("not in this store"), "{notes}");
}

/// A corpus pointed somewhere wrong is the same absence, and must name the path it was
/// pointed at. Reporting only "no corpus" would send somebody to set a variable that is
/// already set.
#[test]
fn Test_An_Unreadable_Corpus_Should_Name_The_Path_It_Was_Given()
{
    let missing = Scratch("unreadable").join("no-corpus-here");

    let output = Nomos(&[
        "spec",
        "sources",
        "--corpus",
        &missing.display().to_string(),
    ]);

    assert_eq!(Code(&output), 6, "{}", Err_Text(&output));
    assert!(
        Out_Text(&output).contains("no-corpus-here"),
        "{}",
        Out_Text(&output)
    );
}

/// A store nothing is missing from says so, and says it with a zero exit — so `sources`
/// is a check a script can run rather than a paragraph a person reads.
#[test]
fn Test_A_Whole_Store_Should_Report_Nothing_Missing()
{
    let corpus = Fixture_Corpus("whole-store");

    let output = Nomos(&["spec", "sources", "--corpus", &corpus.display().to_string()]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let text = Out_Text(&output);
    assert!(text.contains("nothing this store expects is missing"), "{text}");
    assert!(text.contains("governing record(s)"), "{text}");
    assert!(text.contains("1 domain volume(s)"), "{text}");
}

/// Phase 4 shipped fourteen profiles and no command that runs one.
#[test]
fn Test_A_Profile_Should_Render_To_A_File()
{
    let into = Scratch("render");

    let output = Nomos(&[
        "spec",
        "render",
        "--profile",
        "domain-specification",
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));

    let body = into.join("spec/domain-specification.md");
    let sidecar = into.join("spec/domain-specification.md.nomos-projection.json");
    let rendered = std::fs::read_to_string(&body)
        .unwrap_or_else(|error| panic!("{} was not written: {error}", body.display()));
    let stamp = std::fs::read_to_string(&sidecar)
        .unwrap_or_else(|error| panic!("{} was not written: {error}", sidecar.display()));

    assert!(rendered.starts_with("---\nnomos_generated: true\n"), "{:?}", rendered.get(..60));
    assert!(
        rendered.contains("OD-GATE-001-a-skipped-test-reports-ok.md"),
        "the projection does not carry what the store holds"
    );
    assert!(stamp.contains("\"profile\": \"domain-specification\""), "{stamp:.200}");
    assert!(stamp.contains("\"content_digest\""), "{stamp:.200}");
}

/// A profile whose sections need the corpus fails as an absence, not as the profile's own
/// "declare `may_be_empty`" advice — which would send a reader to change a profile because
/// of a variable that is not set.
#[test]
fn Test_A_Profile_That_Needs_The_Corpus_Should_Fail_As_An_Absence()
{
    let into = Scratch("render-absent");

    let output = Nomos(&[
        "spec",
        "render",
        "--profile",
        "architecture-document",
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 6, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(notes.contains("this store is not whole"), "{notes}");
    assert!(!notes.contains("may_be_empty"), "the profile's advice is the wrong advice here");
    assert!(
        !into.join("spec/architecture.md").exists(),
        "a refused render wrote its output anyway"
    );
}

#[test]
fn Test_Every_Shipped_Profile_Should_Be_Listed()
{
    let output = Nomos(&["spec", "profiles"]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    let listed = Out_Text(&output);
    assert_eq!(listed.lines().count(), 14, "{listed}");
    for named in ["domain-specification", "github-markdown", "offline-bundle"]
    {
        assert!(listed.contains(named), "{named} is not listed:\n{listed}");
    }
}

/// The identifier is unknown and the store is whole, so this is the caller's mistake and
/// not a missing corpus. The two must not print the same, which is the whole point.
#[test]
fn Test_An_Unknown_Identifier_Over_A_Whole_Store_Should_Not_Be_An_Absence()
{
    let corpus = Fixture_Corpus("unknown-id");

    let output = Nomos(&[
        "spec",
        "record",
        "--id",
        "D-99999",
        "--corpus",
        &corpus.display().to_string(),
    ]);

    assert_eq!(Code(&output), 1, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(notes.contains("no node in this store is identified D-99999"), "{notes}");
    assert!(!notes.contains("absent:"), "{notes}");
}

/// A node the graph holds with no document behind it is a third answer, and the one most
/// easily reported as either of the other two.
#[test]
fn Test_A_Node_With_No_Document_Should_Say_So_Rather_Than_Report_It_Missing()
{
    let output = Nomos(&["spec", "record", "--id", "ADR-DOC-001"]);

    assert_eq!(Code(&output), 6, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(
        notes.contains("no source document is recorded against it"),
        "{notes}"
    );
    assert!(notes.contains("ADR-DOC-001 is in the store as a"), "{notes}");
}

/// Argument parsing, which is the part that had never run.
#[test]
fn Test_A_Missing_Required_Flag_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["spec", "record"]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    let notes = Err_Text(&output);
    assert!(notes.contains("--id is required"), "{notes}");
    assert!(notes.contains("usage: nomos spec"), "{notes}");
}

#[test]
fn Test_A_Non_Numeric_Ordinal_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["spec", "table", "--document", "x.md", "--table", "second"]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("second"), "{}", Err_Text(&output));
}

#[test]
fn Test_An_Unknown_Spec_Command_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["spec", "frobnicate"]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("frobnicate"), "{}", Err_Text(&output));
}

/// The binary has two groups now, and the top-level usage has to say so — a group nobody
/// can discover is a group nobody runs, which is how fourteen renderers shipped unused.
#[test]
fn Test_The_Binary_Should_Name_Both_Groups()
{
    let output = Nomos(&[]);

    assert_eq!(Code(&output), 2);
    let notes = Err_Text(&output);
    assert!(notes.contains("work"), "{notes}");
    assert!(notes.contains("spec"), "{notes}");
}

/// The `work` group's own parsing, through the binary, because both groups now share one
/// argument reader and a change to it must not be provable only by the group that changed.
#[test]
fn Test_The_Work_Group_Should_Still_Parse_Its_Arguments()
{
    let output = Nomos(&["work", "claim", "--holder", "somebody"]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("--item is required"), "{}", Err_Text(&output));
}
