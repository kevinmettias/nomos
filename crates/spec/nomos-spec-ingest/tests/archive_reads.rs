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

use nomos_spec_ingest::{Archive, ArchiveErrorKind, Archives_In};
use std::io::Write as _;
use std::path::PathBuf;

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
    let path = std::env::temp_dir().join(format!("nomos-p3-archive-{name}.zip"));
    let file = std::fs::File::create(&path).expect("creates the fixture");
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();

    writer.start_file("suite/00-index.md", options).expect("starts");
    writer.write_all(b"---\nid: V0\n---\n\n# Index\n").expect("writes");
    writer.start_file("suite/nested/01-core.md", options).expect("starts");
    writer.write_all("# Core\n\nUnknown is not pass \u{2014} ni\u{f1}o.\n".as_bytes()).expect("writes");
    writer.start_file("suite/binary.bin", options).expect("starts");
    writer.write_all(&[0xFF, 0xFE, 0x00]).expect("writes");
    writer.finish().expect("finishes");

    return path;
}

#[test]
fn Test_An_Archive_Should_List_Its_Files_Sorted()
{
    const NAME: &str = "listing";

    let mut archive = Archive::Open(&Fixture(NAME)).expect("opens");

    assert_eq!(
        archive.Listing().Paths(),
        [
            "suite/00-index.md".to_owned(),
            "suite/binary.bin".to_owned(),
            "suite/nested/01-core.md".to_owned()
        ]
    );
    assert!(archive.Listing().Contains("suite/nested/01-core.md"));
    assert!(!archive.Listing().Contains("suite/absent.md"));
    assert_eq!(archive.Listing().Ending_With(".md").len(), 2);
    assert_eq!(
        archive.Read_Text("suite/00-index.md").expect("reads"),
        "---\nid: V0\n---\n\n# Index\n"
    );
}

/// Addressable by path, including one nested in a directory.
#[test]
fn Test_An_Entry_Should_Be_Addressable_By_Its_Path()
{
    const NAME: &str = "addressing";

    let mut archive = Archive::Open(&Fixture(NAME)).expect("opens");

    let text = archive.Read_Text("suite/nested/01-core.md").expect("reads");

    assert!(text.contains("Unknown is not pass"));
    assert!(text.contains('\u{f1}'), "the bytes did not survive as UTF-8");
}

#[test]
fn Test_A_Missing_Entry_Should_Name_The_Archive_And_The_Entry()
{
    const NAME: &str = "missing-entry";

    let mut archive = Archive::Open(&Fixture(NAME)).expect("opens");

    let refusal = archive.Read("suite/absent.md").expect_err("must refuse");

    assert!(matches!(refusal.kind, ArchiveErrorKind::NoSuchEntry { .. }), "{refusal}");
    let spelled = refusal.to_string();
    assert!(spelled.contains("suite/absent.md"), "{spelled}");
    assert!(spelled.contains(NAME), "{spelled}");
}

/// Bytes that are not text are an error, not a lossy replacement.
#[test]
fn Test_A_Binary_Entry_Should_Refuse_To_Be_Read_As_Text()
{
    const NAME: &str = "binary";

    let mut archive = Archive::Open(&Fixture(NAME)).expect("opens");

    assert_eq!(archive.Read("suite/binary.bin").expect("reads"), vec![0xFF, 0xFE, 0x00]);

    let refusal = archive.Read_Text("suite/binary.bin").expect_err("must refuse");
    assert!(matches!(refusal.kind, ArchiveErrorKind::NotText { .. }), "{refusal}");
}

/// The refusal this item exists for.
#[test]
fn Test_An_Archive_Holding_Nothing_Should_Be_Refused_Not_Reported_Empty()
{
    let path = std::env::temp_dir().join("nomos-p3-archive-empty.zip");
    let file = std::fs::File::create(&path).expect("creates");
    zip::ZipWriter::new(file).finish().expect("finishes an empty archive");

    let Err(refusal) = Archive::Open(&path)
    else
    {
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
    let archives = Archives_In(&root).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(archives.len(), 32, "the archive count changed");

    let mut with_markdown = 0_u32;
    let mut revision_trees = 0_u32;
    let mut docx_deliveries = 0_u32;

    for path in &archives
    {
        let archive = Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));
        let name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default()
            .to_owned();

        assert!(
            !archive.Listing().Paths().is_empty(),
            "{} opened but lists nothing",
            path.display()
        );

        let markdown = archive.Listing().Ending_With(".md").len();
        if markdown > 0
        {
            with_markdown = with_markdown.saturating_add(1);
        }

        if name.starts_with("nomos-spec-internal-artifacts-v14.")
        {
            revision_trees = revision_trees.saturating_add(1);
            assert!(
                markdown > 2000,
                "{name} is a revision tree holding only {markdown} markdown file(s)"
            );
        }
        else if name.starts_with("nomos_v14_")
        {
            docx_deliveries = docx_deliveries.saturating_add(1);
            assert_eq!(markdown, 0, "{name} was classed a DOCX delivery but holds markdown");
            assert!(
                !archive.Listing().Ending_With(".docx").is_empty(),
                "{name} holds neither markdown nor DOCX, so what it is was never established"
            );
        }
    }

    assert_eq!(with_markdown, 16, "the number of archives carrying markdown changed");
    assert_eq!(revision_trees, 12, "the v14 revision series changed length");
    assert_eq!(docx_deliveries, 16, "the DOCX delivery series changed length");
    assert_eq!(
        revision_trees.saturating_add(docx_deliveries).saturating_add(4),
        32,
        "the two series plus v15.0 and the three seed suites no longer account for every archive"
    );
}

/// v15.0 in particular, since it is what P3-OVERLAY reads.
#[test]
fn Test_The_v15_Archive_Should_Yield_Its_Records()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let path = root.join("nomos-spec-v15.0.zip");
    let mut archive = Archive::Open(&path).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(archive.Listing().Paths().len(), 273, "the v15.0 file count changed");

    let records = archive
        .Listing()
        .Paths()
        .iter()
        .filter(|name| name.contains("/records/") && name.to_lowercase().ends_with(".md"))
        .count();
    assert!(records >= 60, "only {records} records under records/");

    let decision = "nomos-spec-v15.0/records/decisions/D-045-runtime-capture-boundary.md";
    assert!(archive.Listing().Contains(decision), "the v15 decision records are not where I4 expects");
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
    let mut archive = Archive::Open(&path).unwrap_or_else(|error| panic!("{error}"));
    for entry in archive.Listing().Ending_With(".md").iter().take(20)
    {
        archive.Read(entry).unwrap_or_else(|error| panic!("{error}"));
    }

    assert_eq!(before, Listing(&root), "reading the archive changed the directory");
}

fn Listing(directory: &std::path::Path) -> Vec<String>
{
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("lists")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    return names;
}
