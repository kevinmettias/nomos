//! P3-ARCHIVE. The versioned archives, read where they lie.
//!
//! Nothing is unpacked. Keeping every revision inside its archive is the same barrier
//! that stops generated output becoming source: an extracted tree in the working
//! directory is a thing a later ingest can mistake for authored input.
//!
//! The failure this file exists to prevent is an archive that reads as empty. A zip that
//! cannot be opened, or that holds nothing, is refused by name — because at every later
//! call site "this revision contained no files" and "this revision was never read" are
//! the same answer, and one of them is a silent loss of a whole revision.

use nomos_spec_ingest::{Archive, Archives_In, ErrorKind as ArchiveErrorKind};
use std::path::PathBuf;

/// The fixture each always-on test writes and reads.
///
/// Four names rather than one, for the reason [`Fixture`] states: these tests run
/// concurrently, and a shared name would have one reading a file another was still
/// writing. They are four facts that happen to be spelled alike, not one fact.
const LISTING_FIXTURE: &str = "listing";
const ADDRESSING_FIXTURE: &str = "addressing";
const MISSING_ENTRY_FIXTURE: &str = "missing-entry";
const BINARY_FIXTURE: &str = "binary";

/// The markdown entries the fixture writes: `suite/00-index.md` and `suite/nested/01-core.md`.
const FIXTURE_MARKDOWN_ENTRIES: usize = 2;

/// The bytes of the fixture's third entry. They are not text, which is the point: a reader
/// that decoded them to something would pass a lossy replacement off as the file's content.
const BINARY_BYTES: &[u8] = &[0xFF, 0xFE, 0x00];

/// The real archive set: two series that do not overlap, and what belongs to neither.
const ARCHIVES: usize = 32;
const ARCHIVES_WITH_MARKDOWN: usize = 16;
const REVISION_TREES: usize = 12;
const DOCX_DELIVERIES: usize = 16;

/// The archives outside both series: v15.0 and the three seed suites.
const OTHER_ARCHIVES: usize = 4;

/// How many markdown files a v14 revision tree carries at least. A floor rather than a count,
/// so it separates a revision tree from a DOCX delivery without pinning a second figure here.
const REVISION_TREE_MARKDOWN_FLOOR: usize = 2000;

/// v15.0's file count, and the floor under the records it carries.
const V15_FILES: usize = 273;
const V15_RECORDS_FLOOR: usize = 60;

/// How many entries the unpacking test reads before it compares the directory to itself.
const UNPACK_SAMPLE: usize = 20;

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(
        root.is_dir(),
        "NOMOS_SPEC_ARCHIVES is set to {}, which is not a directory",
        root.display()
    );
    return Some(root);
}

/// A zip written for one test, so the always-on cases do not depend on the corpus.
///
/// Named per caller: these tests run concurrently, and one shared path would have them
/// reading a file another was still writing.
fn Fixture(name: &str) -> PathBuf
{
    use std::io::Write as _;

    let path = std::env::temp_dir().join(format!("nomos-p3-archive-{name}.zip"));
    let file = std::fs::File::create(&path).expect("creates the fixture");
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();

    writer.start_file("suite/00-index.md", options).expect("the writer holds no open entry");
    writer.write_all(b"---\nid: V0\n---\n\n# Index\n").expect("the open entry takes the bytes");
    writer.start_file("suite/nested/01-core.md", options).expect("the writer holds no open entry");
    writer
        .write_all("# Core\n\nUnknown is not pass \u{2014} ni\u{f1}o.\n".as_bytes())
        .expect("the open entry takes the bytes");
    writer.start_file("suite/binary.bin", options).expect("the writer holds no open entry");
    writer.write_all(BINARY_BYTES).expect("the open entry takes the bytes");
    writer.finish().expect("the fixture's entries are all written, so it closes");

    return path;
}

#[test]
fn Test_An_Archive_Should_List_Its_Files_Sorted()
{
    let mut archive = Archive::Open(&Fixture(LISTING_FIXTURE))
        .expect("the fixture is a zip this reader wrote, so it opens");

    assert_eq!(
        archive.Listing().Paths(),
        [
            "suite/00-index.md".to_owned(),
            "suite/binary.bin".to_owned(),
            "suite/nested/01-core.md".to_owned()
        ]
    );
    assert!(archive.Listing().Has_Path("suite/nested/01-core.md"));
    assert!(!archive.Listing().Has_Path("suite/absent.md"));
    assert_eq!(archive.Listing().Ending_With(".md").len(), FIXTURE_MARKDOWN_ENTRIES);
    assert_eq!(
        archive.Read_Text("suite/00-index.md").expect("reads"),
        "---\nid: V0\n---\n\n# Index\n"
    );
}

/// Addressable by path, including one nested in a directory.
#[test]
fn Test_An_Entry_Should_Be_Addressable_By_Its_Path()
{
    let mut archive = Archive::Open(&Fixture(ADDRESSING_FIXTURE))
        .expect("the fixture is a zip this reader wrote, so it opens");

    let text = archive.Read_Text("suite/nested/01-core.md").expect("the fixture wrote this entry");

    assert!(text.contains("Unknown is not pass"));
    assert!(text.contains('\u{f1}'), "the bytes did not survive as UTF-8");
}

#[test]
fn Test_A_Missing_Entry_Should_Name_The_Archive_And_The_Entry()
{
    let mut archive = Archive::Open(&Fixture(MISSING_ENTRY_FIXTURE))
        .expect("the fixture is a zip this reader wrote, so it opens");

    let refusal = archive.Read("suite/absent.md").expect_err("must refuse");

    assert!(matches!(refusal.kind, ArchiveErrorKind::NoSuchEntry { .. }), "{refusal}");
    let spelled = refusal.to_string();
    assert!(spelled.contains("suite/absent.md"), "{spelled}");
    assert!(spelled.contains(MISSING_ENTRY_FIXTURE), "{spelled}");
}

/// Bytes that are not text are an error, not a lossy replacement.
#[test]
fn Test_A_Binary_Entry_Should_Refuse_To_Be_Read_As_Text()
{
    let mut archive = Archive::Open(&Fixture(BINARY_FIXTURE))
        .expect("the fixture is a zip this reader wrote, so it opens");

    assert_eq!(
        archive.Read("suite/binary.bin").expect("the fixture wrote this entry"),
        BINARY_BYTES.to_vec()
    );

    let refusal = archive.Read_Text("suite/binary.bin").expect_err("must refuse");
    assert!(matches!(refusal.kind, ArchiveErrorKind::NotText { .. }), "{refusal}");
}

/// The refusal this item exists for.
#[test]
fn Test_An_Archive_Holding_Nothing_Should_Be_Refused_Not_Reported_Empty()
{
    let path = std::env::temp_dir().join("nomos-p3-archive-empty.zip");
    let file = std::fs::File::create(&path).expect("the temp directory is writable, so it creates the file");
    zip::ZipWriter::new(file).finish().expect("finishes an empty archive");

    let Err(refusal) = Archive::Open(&path)
    else
    {
        // This arm is reachable only when `Open` accepted a zip holding nothing, which is the one
        // confusion this file exists to stop. There is no refusal left to assert against, so
        // arriving here is itself the assertion failing.
        panic!("an empty archive must be refused, not opened");
    };

    assert!(matches!(refusal.kind, ArchiveErrorKind::Empty), "{refusal}");
    assert!(
        refusal.to_string().contains("nomos-p3-archive-empty.zip"),
        "the error does not say which archive: {refusal}"
    );
}

/// Every real archive opens, none reads as empty, and the two families are distinct.
///
/// The counts here are measured, not estimated. The plan says "about twenty archives";
/// there are 32, and they are not one series: 16 carry markdown spec trees and 16 are
/// DOCX deliveries carrying no markdown at all. Asserting "most hold markdown" passed
/// for the wrong reason until it was measured, which is exactly the drift these counts
/// exist to catch.
#[test]
fn Test_Every_Real_Archive_Should_Open_And_Hold_Files()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    // `Archives()` has already established that the root is a directory, so a listing that refuses
    // is a fact about this machine and not about the archive set. The counts below would spell it
    // "the archive count changed" and send the reader to the archives instead of to the disk.
    let archives = Archives_In(&root).unwrap_or_else(|error| panic!("{error}"));
    let mut counted = Series::default();
    for path in &archives
    {
        Count_One_Archive(path, &mut counted);
    }
    let Series {
        with_markdown,
        revision_trees,
        docx_deliveries,
    } = counted;

    assert_eq!(archives.len(), ARCHIVES, "the archive count changed");
    assert_eq!(with_markdown, ARCHIVES_WITH_MARKDOWN, "the number of archives carrying markdown changed");
    assert_eq!(revision_trees, REVISION_TREES, "the v14 revision series changed length");
    assert_eq!(docx_deliveries, DOCX_DELIVERIES, "the DOCX delivery series changed length");
    assert_eq!(
        revision_trees.saturating_add(docx_deliveries).saturating_add(OTHER_ARCHIVES),
        ARCHIVES,
        "the two series plus v15.0 and the three seed suites no longer account for every archive"
    );
}

/// How many archives of each kind the directory holds.
#[derive(Default)]
struct Series
{
    with_markdown: usize,
    revision_trees: usize,
    docx_deliveries: usize,
}

/// One archive: it opens, it lists something, and it is one of the two series or neither.
fn Count_One_Archive(path: &std::path::Path, counted: &mut Series)
{
    // An archive that will not open belongs to neither series, and counting it as neither leaves
    // both totals short — which the caller reads as "the DOCX delivery series changed length"
    // rather than as one file on disk that would not open.
    let archive = Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));
    let name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_owned();
    let markdown = archive.Listing().Ending_With(".md").len();

    assert!(
        !archive.Listing().Paths().is_empty(),
        "{} opened but lists nothing",
        path.display()
    );
    if markdown > 0
    {
        counted.with_markdown = counted.with_markdown.saturating_add(1);
    }
    if name.starts_with("nomos-spec-internal-artifacts-v14.")
    {
        counted.revision_trees = counted.revision_trees.saturating_add(1);
        assert!(
            markdown > REVISION_TREE_MARKDOWN_FLOOR,
            "{name} is a revision tree holding only {markdown} markdown file(s)"
        );
    }
    else if name.starts_with("nomos_v14_")
    {
        counted.docx_deliveries = counted.docx_deliveries.saturating_add(1);
        assert_eq!(markdown, 0, "{name} was classed a DOCX delivery but holds markdown");
        assert!(
            !archive.Listing().Ending_With(".docx").is_empty(),
            "{name} holds neither markdown nor DOCX, so what it is was never established"
        );
    }
}

/// v15.0 in particular, since it is what P3-OVERLAY reads.
#[test]
fn Test_The_V15_Archive_Should_Yield_Its_Records()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let path = root.join("nomos-spec-v15.0.zip");
    // v15.0 is the single archive P3-OVERLAY reads, so there is no reduced version of this test to
    // run without it — every assertion below would be about an archive that was never opened.
    let mut archive = Archive::Open(&path).unwrap_or_else(|error| panic!("{error}"));
    let records = archive
        .Listing()
        .Paths()
        .iter()
        .filter(|name| name.contains("/records/") && name.to_lowercase().ends_with(".md"))
        .count();
    let decision = "nomos-spec-v15.0/records/decisions/D-045-runtime-capture-boundary.md";

    assert_eq!(archive.Listing().Paths().len(), V15_FILES, "the v15.0 file count changed");
    assert!(records >= V15_RECORDS_FLOOR, "only {records} records under records/");
    assert!(archive.Listing().Has_Path(decision), "the v15 decision records are not where I4 expects");
    assert!(
        archive.Read_Text(decision).expect("reads").starts_with("---\nid: D-045"),
        "the record did not read as its own front matter"
    );
}

/// Reading must not put anything on disk beside the archive.
#[test]
fn Test_Reading_Should_Not_Unpack()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let before = Listing(&root);
    let path = root.join("nomos-spec-v15.0.zip");
    // An open that failed quietly would leave the two listings identical because nothing was ever
    // read, and the assertion below would report "reading did not unpack" having never read.
    let mut archive = Archive::Open(&path).unwrap_or_else(|error| panic!("{error}"));
    for entry in archive.Listing().Ending_With(".md").iter().take(UNPACK_SAMPLE)
    {
        // The same vacuity one entry at a time: a read that errored cannot have put anything on
        // disk, so swallowing it buys the comparison below for free.
        archive.Read(entry).unwrap_or_else(|error| panic!("{error}"));
    }

    assert_eq!(before, Listing(&root), "reading the archive changed the directory");
}

fn Listing(directory: &std::path::Path) -> Vec<String>
{
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("the caller established this directory exists")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    return names;
}
