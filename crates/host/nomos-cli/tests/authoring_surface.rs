//! The authoring round trip, driven the way a person drives it.
//!
//! The library assertions live in `nomos-spec-store`'s `round_trip_holds.rs`; what runs only
//! here is the part `P9-AUTHORING` is about at the surface — that a person can read a record
//! out of the store, edit the file, see what their edit changes, and commit it, and that the
//! commit prints the preview rather than trusting the caller to have asked for one.
//!
//! # Why this file is not a corpus gate
//!
//! It names the corpus variable and removes it from the child's environment, for the same
//! reason `read_surface.rs` does: these assertions must behave identically on a machine that
//! holds a v14 corpus and one that does not. The name is assembled with `concat!` so the
//! census that resolves a gate by finding the variable in reachable source does not count a
//! test that reads no corpus.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const V14: &str = concat!("NOMOS_", "V14_CORPUS");

/// The record these tests read and edit.
///
/// `D-131` rather than `D-129`: the tests below stage edited copies of it, and D-129 is the
/// record this whole item amends, so a fixture built from it would change shape underneath
/// them for reasons that have nothing to do with the surface.
const RECORD: &str = "D-131";
const RECORD_PATH: &str = "docs/records/D-131-a-byte-order-mark-belongs-to-the-front-matter-fence.md";

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

/// The record as it is on disk, which is what the store was seeded from.
fn On_Disk() -> String
{
    return std::fs::read_to_string(Repository_Root().join(RECORD_PATH)).expect("reads the record");
}

/// Writes a text to stage, whatever it is.
fn Staged(name: &str, text: &str) -> PathBuf
{
    let path = Scratch(name).join("staged.md");
    std::fs::write(&path, text).expect("writes the staged record");

    return path;
}

/// Writes an *edited* copy, and refuses to build a fixture that changes nothing.
///
/// Separate from [`Staged`] because two tests below stage the record unchanged on purpose — a
/// rename, and a run whose only subject is what the command says about itself — and a guard
/// that cannot tell those from a `replace` that silently matched nothing is a guard that fires
/// on the wrong cases.
fn Edited(name: &str, edit: impl Fn(&str) -> String) -> (PathBuf, String)
{
    let text = edit(&On_Disk());
    assert_ne!(text, On_Disk(), "the {name} fixture changed nothing");

    return (Staged(name, &text), text);
}

/// The read half of the round trip: the store renders the record back, byte for byte.
#[test]
fn Test_Markdown_Should_Print_The_Record_The_Store_Can_Rebuild()
{
    let output = Nomos(&["spec", "markdown", "--id", RECORD]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert_eq!(
        Out_Text(&output),
        On_Disk(),
        "the store does not render the record back as the file it was seeded from"
    );
    assert!(
        Err_Text(&output).contains("rendered from the store's rows"),
        "{}",
        Err_Text(&output)
    );
}

/// Content on one stream and everything about it on the other, so a redirect captures a file
/// that is exactly what the store holds — the same contract `record` already keeps.
#[test]
fn Test_Markdown_Should_Keep_Content_And_Commentary_Apart()
{
    let output = Nomos(&["spec", "markdown", "--id", RECORD]);

    assert!(Out_Text(&output).starts_with("---\nid: D-131\n"), "{}", Out_Text(&output));
    assert!(!Out_Text(&output).contains("rendered from"), "commentary reached stdout");
    assert!(!Err_Text(&output).contains("id: D-131"), "content reached stderr");
}

/// A preview changes nothing, and says what it would change.
#[test]
fn Test_Preview_Should_Describe_The_Edit_And_Write_Nothing()
{
    let (staged, _) = Edited("preview-describes", |text| {
        return text.replace("## Decision", "## The decision");
    });
    let before = On_Disk();

    let output = Nomos(&[
        "spec",
        "preview",
        "--id",
        RECORD,
        "--from",
        &staged.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert!(Out_Text(&output).contains("normative wording"), "{}", Out_Text(&output));
    assert!(Err_Text(&output).contains("nothing was written"), "{}", Err_Text(&output));
    assert_eq!(On_Disk(), before, "a preview edited the tree");
}

/// The whole point of the group. A commit prints the preview it ran, writes the file where the
/// record's own path says, and says the store renders it back as what was committed.
#[test]
fn Test_Commit_Should_Preview_Then_Write_The_Record()
{
    let (staged, text) = Edited("commit-writes", |source| {
        return source.replace("belongs to the front matter fence", "belongs to the fence");
    });
    let into = Scratch("commit-writes-tree");
    let before = On_Disk();

    let output = Nomos(&[
        "spec",
        "commit",
        "--id",
        RECORD,
        "--from",
        &staged.display().to_string(),
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert!(
        Out_Text(&output).contains("normative wording"),
        "the commit did not print its preview: {}",
        Out_Text(&output)
    );
    assert!(Out_Text(&output).contains("committed D-131"), "{}", Out_Text(&output));
    assert!(
        Out_Text(&output).contains("renders it back as the same bytes"),
        "{}",
        Out_Text(&output)
    );
    assert_eq!(
        std::fs::read_to_string(into.join(RECORD_PATH)).expect("the record was written"),
        text
    );
    assert_eq!(On_Disk(), before, "the commit wrote into the tree it was not given");
}

/// A rename is an ordinary edit, and it leaves one file rather than two.
#[test]
fn Test_Commit_Should_Move_A_Renamed_Record_And_Vacate_Its_Old_Path()
{
    let staged = Staged("commit-renames", &On_Disk());
    let into = Scratch("commit-renames-tree");
    let moved = "docs/records/D-131-the-mark-belongs-to-the-fence.md";

    // The old path has to exist under the same tree for vacating it to mean anything.
    std::fs::create_dir_all(into.join("docs/records")).expect("creates the record directory");
    std::fs::write(into.join(RECORD_PATH), On_Disk()).expect("plants the record");

    let output = Nomos(&[
        "spec",
        "commit",
        "--id",
        RECORD,
        "--from",
        &staged.display().to_string(),
        "--rename",
        moved,
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
    assert!(Out_Text(&output).contains("renamed"), "{}", Out_Text(&output));
    assert!(into.join(moved).exists(), "the record was not written to its new path");
    assert!(
        !into.join(RECORD_PATH).exists(),
        "the old path still holds a file declaring this record"
    );
}

/// An edit this surface would not have written is refused with its own code, and writes
/// nothing. `9` rather than `2`: the command line was right and the content was not.
#[test]
fn Test_An_Edit_This_Surface_Would_Not_Write_Should_Exit_Refused()
{
    let (staged, _) = Edited("commit-refuses", |text| {
        return text.replace("## Decision\n\n", "## Decision\n\n\n");
    });
    let into = Scratch("commit-refuses-tree");

    let output = Nomos(&[
        "spec",
        "commit",
        "--id",
        RECORD,
        "--from",
        &staged.display().to_string(),
        "--into",
        &into.display().to_string(),
    ]);

    assert_eq!(Code(&output), 9, "{}", Err_Text(&output));
    assert!(
        Err_Text(&output).contains("would change bytes the edit did not ask to change"),
        "{}",
        Err_Text(&output)
    );
    assert!(!into.join(RECORD_PATH).exists(), "a refused commit wrote the record");
}

/// Retyping the identifier is refused, because identity is not a property of the file.
#[test]
fn Test_Restating_The_Identifier_Should_Be_Refused()
{
    let (staged, _) = Edited("commit-reidentifies", |text| {
        return text.replace("id: D-131", "id: D-131-2");
    });

    let output = Nomos(&[
        "spec",
        "preview",
        "--id",
        RECORD,
        "--from",
        &staged.display().to_string(),
    ]);

    assert_eq!(Code(&output), 9, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("Identity lives in the store"), "{}", Err_Text(&output));
}

/// A `--from` that names nothing is the command line being wrong, and reports as that.
#[test]
fn Test_A_Staged_File_That_Is_Not_There_Should_Be_A_Usage_Error()
{
    let output = Nomos(&["spec", "preview", "--id", RECORD, "--from", "no-such-file.md"]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("no-such-file.md"), "{}", Err_Text(&output));
}

#[test]
fn Test_Preview_Should_Require_A_Staged_File()
{
    let output = Nomos(&["spec", "preview", "--id", RECORD]);

    assert_eq!(Code(&output), 2, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("--from"), "{}", Err_Text(&output));
}

/// An identifier this store does not hold, over a store whose corpus is absent, is an absence
/// rather than a mistake — the distinction `D-133` built the whole read surface around, kept
/// by the authoring commands too.
#[test]
fn Test_An_Unknown_Record_Should_Report_The_Absence_It_Might_Be()
{
    let output = Nomos(&["spec", "markdown", "--id", "AGT-001"]);

    assert_eq!(Code(&output), 6, "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("AGT-001"), "{}", Err_Text(&output));
    assert!(Err_Text(&output).contains("absent"), "{}", Err_Text(&output));
}

/// Every run says what it did and did not persist. The store is assembled per invocation, so a
/// commit that said only "committed" would leave a reader believing a database somewhere now
/// holds their edit.
#[test]
fn Test_Every_Authoring_Run_Should_Say_What_Persists()
{
    let staged = Staged("says-what-persists", &On_Disk());
    let into = Scratch("says-what-persists-tree");
    let from = staged.display().to_string();
    let root = into.display().to_string();

    for arguments in [
        vec!["spec", "preview", "--id", RECORD, "--from", &from],
        vec![
            "spec", "commit", "--id", RECORD, "--from", &from, "--into", &root,
        ],
    ]
    {
        let output = Nomos(&arguments);

        assert_eq!(Code(&output), 0, "{}", Err_Text(&output));
        assert!(
            Err_Text(&output).contains("OD-SPEC-006"),
            "{:?} does not say what persists: {}",
            arguments,
            Err_Text(&output)
        );
    }
}
