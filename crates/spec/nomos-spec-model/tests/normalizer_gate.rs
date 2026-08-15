//! P1-GATE. The hashes here are v14's, recorded by a Python toolchain this workspace
//! is replacing. Reproducing them is what makes every preservation rule downstream
//! measure something.
//!
//! The algorithm was recovered from the corpus rather than chosen, because the tool
//! that produced these values does not ship with it.

use nomos_spec_model::{BlockKind, ContentHash, Is_Normalized, Normalize, Segment, SourceBlock};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct BlockRecord
{
    ordinal: u32,
    kind: String,
    content_hash: String,
    normalized_hash: String,
}

#[derive(Deserialize)]
struct DiscriminatingBlock
{
    source_document: String,
    ordinal: u32,
    text: String,
    content_hash: String,
    normalized_hash: String,
}

#[derive(Deserialize)]
struct StatementRecord
{
    id: String,
    canonical_text: String,
    canonical_hash: String,
}

fn Corpus(name: &str) -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus").join(name);
}

fn Read(name: &str) -> String
{
    let path = Corpus(name);
    return std::fs::read_to_string(&path)
        // These fixtures are committed under `tests/corpus` and carry the recorded hashes this
        // gate reproduces. A missing one leaves nothing to reproduce against, and `Load` would
        // otherwise have to hand back an empty vector — over which every assertion in this
        // file iterates and therefore passes.
        .unwrap_or_else(|error| panic!("the gate needs {}: {error}", path.display()));
}

fn Load<T: serde::de::DeserializeOwned>(name: &str) -> Vec<T>
{
    return serde_json::from_str(&Read(name)).expect("fixture is valid json");
}

fn Kind_Label(kind: BlockKind) -> &'static str
{
    return match kind
    {
        BlockKind::Heading => "heading",
        BlockKind::Prose => "prose",
        BlockKind::Code => "code",
    };
}

/// Segmentation and content hashing over a real authored document.
#[test]
fn Test_Every_Block_Of_A_Real_Document_Should_Reproduce_Its_Hashes()
{
    let expected: Vec<BlockRecord> = Load("suite-index-blocks.json");
    let blocks = Segment(&Read("suite-index.md"));

    assert!(!expected.is_empty(), "the fixture must not be empty");
    assert_eq!(
        blocks.len(),
        expected.len(),
        "segmentation produced a different number of blocks than v14 recorded"
    );

    for (block, want) in blocks.iter().zip(&expected)
    {
        Assert_The_Block_Matches(block, want);
    }
}

/// One block against what v14 recorded for it: its kind and both of its hashes.
fn Assert_The_Block_Matches(block: &SourceBlock, want: &BlockRecord)
{
    assert_eq!(block.ordinal, want.ordinal);
    assert_eq!(Kind_Label(block.kind), want.kind, "block {} kind", want.ordinal);
    assert_eq!(
        block.Content_Hash().As_Str(),
        want.content_hash,
        "block {} content_hash",
        want.ordinal
    );
    assert_eq!(
        block.Normalized_Hash().As_Str(),
        want.normalized_hash,
        "block {} normalized_hash",
        want.ordinal
    );
}

/// The only blocks in the entire v14 corpus where normalization changes the hash.
///
/// Everything else is already a fixed point, so a normalizer that did nothing at all
/// would pass every other check in this file. These thirty are what tell the two apart.
#[test]
fn Test_The_Normalizer_Should_Reproduce_The_Discriminating_Hashes()
{
    let blocks: Vec<DiscriminatingBlock> = Load("discriminating-blocks.json");

    assert_eq!(
        blocks.len(),
        30,
        "the discriminating set is fixed; a change here means the corpus changed"
    );

    for block in &blocks
    {
        Assert_It_Discriminates(block);
    }
}

/// A block that hashes the same either way tells the normalizer apart from nothing, so it
/// does not belong in this fixture.
fn Assert_It_Discriminates(block: &DiscriminatingBlock)
{
    assert_ne!(
        block.content_hash, block.normalized_hash,
        "{}#{} does not discriminate and does not belong in this fixture",
        block.source_document, block.ordinal
    );
    assert_eq!(
        ContentHash::Of(&block.text).As_Str(),
        block.content_hash,
        "{}#{} content_hash",
        block.source_document,
        block.ordinal
    );
    assert_eq!(
        ContentHash::Of_Normalized(&block.text).As_Str(),
        block.normalized_hash,
        "{}#{} normalized_hash",
        block.source_document,
        block.ordinal
    );
}

/// The plan's stated gate, over more than the twenty it asked for.
#[test]
fn Test_Sampled_Statements_Should_Reproduce_Their_Canonical_Hash()
{
    let statements: Vec<StatementRecord> = Load("statements.json");

    assert!(
        statements.len() >= 20,
        "the gate requires at least twenty sampled statements, found {}",
        statements.len()
    );

    for statement in &statements
    {
        assert_eq!(
            ContentHash::Of(&statement.canonical_text).As_Str(),
            statement.canonical_hash,
            "{} canonical_hash",
            statement.id
        );
    }
}

/// Non-ASCII statements are what pin the encoding. Under latin-1, or under NFD, these
/// hash differently; under UTF-8 with the bytes left alone they do not.
#[test]
fn Test_The_Sample_Should_Contain_Non_Ascii_Statements()
{
    let statements: Vec<StatementRecord> = Load("statements.json");
    let non_ascii = statements
        .iter()
        .filter(|statement| !statement.canonical_text.is_ascii())
        .count();

    assert!(
        non_ascii >= 10,
        "only {non_ascii} non-ascii statements: the sample no longer pins the encoding"
    );
}

/// Every recorded `canonical_text` is already what the normalizer produces.
///
/// This is why the statement file alone cannot validate a normalizer — every candidate
/// is the identity on it — and it is also the invariant that lets `canonical_hash` hash
/// the text verbatim without two spellings becoming two statements.
#[test]
fn Test_Every_Canonical_Text_Should_Be_A_Fixed_Point()
{
    let statements: Vec<StatementRecord> = Load("statements.json");

    for statement in &statements
    {
        assert!(
            Is_Normalized(&statement.canonical_text),
            "{} is not a fixed point of the normalizer",
            statement.id
        );
        assert_eq!(Normalize(&statement.canonical_text), statement.canonical_text);
    }
}

/// The whole corpus, when it is reachable. Opt-in by path, and loud rather than silent:
/// a configured corpus that cannot be read fails, because a gate that quietly skips its
/// own subject is worse than one that fails.
#[test]
fn Test_The_Whole_Corpus_Should_Reproduce_When_Available()
{
    let Some(root) = Corpus_Root()
    else
    {
        return;
    };
    let volumes = root.join("01_authoring/domain_volumes");
    let entries = std::fs::read_dir(&volumes)
        // `Corpus_Root` has already established that the configured root is a directory, so a
        // failure here is a corpus whose `01_authoring/domain_volumes` is gone. Treating that
        // as no entries would leave the ten-volume assertion below to report "found 0" and
        // send the reader looking for volumes rather than for the directory.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", volumes.display()));

    let mut documents = 0_u32;
    for entry in entries.flatten()
    {
        documents = documents.saturating_add(Assert_It_Segments(&entry.path()));
    }

    assert!(documents >= 10, "expected the ten domain volumes, found {documents}");
}

/// Opt-in by path, and loud rather than silent: a configured corpus that cannot be read fails,
/// because a gate that quietly skips its own subject is worse than one that fails.
fn Corpus_Root() -> Option<PathBuf>
{
    let named = std::env::var_os("NOMOS_V14_CORPUS")?;
    let root = PathBuf::from(named);

    assert!(
        root.is_dir(),
        "NOMOS_V14_CORPUS is set to {}, which is not a directory",
        root.display()
    );

    return Some(root);
}

/// One volume, if the entry is one. Answers 1 where a document was read.
fn Assert_It_Segments(path: &Path) -> u32
{
    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
    {
        return 0;
    }

    let text = std::fs::read_to_string(path)
        // The extension check two lines up has already accepted this entry as a volume, so an
        // unreadable one is a file that moved mid-run. Answering 0 instead would quietly drop
        // it from the count and make the ten-volume assertion blame the corpus.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    assert!(
        !Segment(&text).is_empty(),
        "{} segmented to nothing",
        path.display()
    );

    return 1;
}
