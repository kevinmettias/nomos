//! P3-BOM. How v14 segmented a document carrying a byte order mark.
//!
//! Settled against the real archives rather than by preference. v14's two shipped
//! readers — `build_spec.py::split_markdown_front_matter` and
//! `validate_bundle.py::markdown_document_records` — both match front matter as
//! `FRONT_RE = ^\ufeff?---\n(.*?)\n---\n`, so a mark is part of the opening fence and is
//! consumed with it. Run over the corpus, that pattern matches all 2187 authored
//! documents, 1637 of which carry a mark, and no body retains one. Had the mark not been
//! tolerated, `markdown_document_records` would have reported 1637 documents as missing
//! front matter, and v14 validated clean.
//!
//! What the archives do **not** settle: `source-block-lineage.yaml` records 2533 blocks
//! drawn from exactly ten documents, the domain volumes, and none of the ten carries a
//! mark. So no recorded block hash corroborates any of this, and the segmenter that
//! produced those hashes does not ship. The evidence here is the readers' agreement, not
//! a manifest, and the residual is stated rather than closed.

use nomos_spec_model::{BlockKind, Segment};
use std::path::{Path, PathBuf};

const BYTE_ORDER_MARK: char = '\u{feff}';

/// A real v14 decision record, byte for byte, mark included.
///
/// Committed because the corpus is not in this repository. `include_str!` would strip
/// nothing, but the bytes are read here so the mark's presence is asserted rather than
/// assumed — a fixture that quietly lost it would make this whole file vacuous.
fn Fixture() -> String
{
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/bom-decision-record.md");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("the gate needs {}: {error}", path.display()));

    assert_eq!(
        bytes.get(..3),
        Some([0xEF, 0xBB, 0xBF].as_slice()),
        "the fixture no longer carries a byte order mark and pins nothing"
    );

    return String::from_utf8(bytes).expect("the fixture is utf-8");
}

/// The behaviour under test, over a real marked document.
#[test]
fn Test_A_Marked_Record_Should_Have_Its_Front_Matter_Skipped()
{
    let text = Fixture();
    let blocks = Segment(&text);

    assert!(!blocks.is_empty(), "the fixture segmented to nothing");
    assert_eq!(
        blocks.first().map(|block| block.kind),
        Some(BlockKind::Heading),
        "the first block is not the heading, so the front matter became content"
    );
    assert_eq!(
        blocks.first().map(|block| block.text.as_str()),
        Some("# Runtime capture boundary")
    );

    for block in &blocks
    {
        assert!(
            !block.text.contains(BYTE_ORDER_MARK),
            "block {} carries the mark: {:?}",
            block.ordinal,
            block.text
        );
        assert!(
            !block.text.starts_with("---"),
            "block {} is front matter read as content",
            block.ordinal
        );
    }
}

/// The mark must make no difference at all, not merely be tolerated.
#[test]
fn Test_The_Mark_Should_Not_Change_A_Single_Block()
{
    let text = Fixture();
    let unmarked = text.strip_prefix(BYTE_ORDER_MARK).expect("the fixture is marked");

    assert_ne!(unmarked, text, "the negative control removed nothing");
    assert_eq!(Segment(&text), Segment(unmarked));
}

/// The whole authored tree, when it is reachable. Opt-in by path, and loud rather than
/// silent: a configured corpus that cannot be read fails, because a gate that skips its
/// own subject is worse than one that fails.
#[test]
fn Test_Every_Authored_Document_Should_Segment_Past_Its_Front_Matter()
{
    let Some(root) = std::env::var_os("NOMOS_V14_CORPUS")
    else
    {
        return;
    };

    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "NOMOS_V14_CORPUS is set to {}, which is not a directory",
        root.display()
    );

    let authoring = root.join("01_authoring");
    let mut documents = Vec::new();
    Collect(&authoring, &mut documents);

    let mut marked = 0_u32;
    for path in &documents
    {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let blocks = Segment(&text);

        assert_eq!(
            blocks.first().map(|block| block.kind),
            Some(BlockKind::Heading),
            "{} does not open with a heading, so its front matter became content",
            path.display()
        );

        if let Some(unmarked) = text.strip_prefix(BYTE_ORDER_MARK)
        {
            marked = marked.saturating_add(1);
            assert_eq!(
                blocks,
                Segment(unmarked),
                "{} reads differently with its mark",
                path.display()
            );
        }

        for block in &blocks
        {
            assert!(
                !block.text.contains(BYTE_ORDER_MARK),
                "{} block {} carries the mark",
                path.display(),
                block.ordinal
            );
        }
    }

    assert!(
        documents.len() >= 2000,
        "expected the authored tree, found {} documents",
        documents.len()
    );
    assert!(
        marked >= 1600,
        "only {marked} marked documents: this no longer measures the case it exists for"
    );
}

fn Collect(directory: &Path, into: &mut Vec<PathBuf>)
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    for entry in entries.flatten()
    {
        let path = entry.path();
        if path.is_dir()
        {
            Collect(&path, into);
        }
        else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("md")
        {
            into.push(path);
        }
    }
}
