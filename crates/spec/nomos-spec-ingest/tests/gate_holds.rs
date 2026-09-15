//! The I1 gate over a real slice of the v14 corpus, vendored so it always runs.
//!
//! `real_corpus.rs` covers the whole 2533-block manifest and is opt-in by path. This
//! file exists because an opt-in test reports `ok` when the path is unset, and `nomos
//! work finish` would accept that as the gate having passed. A vendored slice is real
//! recorded data: the hashes below were produced by v14's Python toolchain, not by us.

use nomos_spec_ingest::{Check_Against_Manifest, Ingest_Statements, Parse_Block_Lineage, Parse_Statements};
use std::collections::BTreeMap;

/// How many mismatches the failure message names before it stops listing them.
const MISMATCHES_LISTED: usize = 5;

/// The blocks the vendored lineage slice covers, as v14's own manifest counted them.
const SLICE_BLOCKS: u32 = 46;

/// The statements the vendored statement slice holds.
const SLICE_STATEMENTS: u32 = 16;

/// A floor rather than a measurement: the slice carries non-ascii statements, and this many
/// pins the encoding rather than merely showing one of them.
const NON_ASCII_FLOOR: usize = 5;

fn Fixture(name: &str) -> String
{
    use std::path::Path;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus").join(name);
    return std::fs::read_to_string(&path)
        // These fixtures are vendored into the repository so this gate always runs. A missing
        // one has to fail rather than skip: `nomos work finish` reads a skipped test as the
        // gate having passed, which is the failure mode this whole file exists to close.
        .unwrap_or_else(|error| panic!("the gate needs {}: {error}", path.display()));
}

fn Documents() -> BTreeMap<String, String>
{
    return BTreeMap::from([("00-suite-index.md".to_owned(), Fixture("00-suite-index.md"))]);
}

#[test]
fn Test_The_Gate_Should_Pass_Against_Real_Recorded_Blocks()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");

    let report = Check_Against_Manifest(&lineage, &Documents());

    assert!(
        report.mismatches.is_empty(),
        "{}\n  {}",
        report.Summary(),
        report
            .mismatches
            .iter()
            .take(MISMATCHES_LISTED)
            .map(nomos_spec_ingest::Mismatch::Describe)
            .collect::<Vec<String>>()
            .join("\n  ")
    );
    assert!(report.Is_Passing());
    assert_eq!(report.blocks_checked, SLICE_BLOCKS, "{}", report.Summary());
    assert_eq!(report.documents_checked, 1);
}

/// This slice contains no table, so it verifies hashing and segmentation and *not*
/// normalization. The report has to say so rather than reporting an unqualified pass —
/// which is the whole reason `Has_Exercised_The_Normalizer` is separate from `Is_Passing`.
#[test]
fn Test_This_Slice_Should_Not_Claim_To_Exercise_The_Normalizer()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");

    let report = Check_Against_Manifest(&lineage, &Documents());

    assert!(report.Is_Passing());
    assert!(
        !report.Has_Exercised_The_Normalizer(),
        "no block in this slice discriminates, so the run must not claim normalization \
         was verified"
    );
}

/// The negative control. One altered byte of body text and the gate must name the block.
#[test]
fn Test_An_Altered_Source_Should_Fail_The_Gate()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");
    let source = Fixture("00-suite-index.md");
    let altered = source.replacen("Focused", "Focussed", 1);
    assert_ne!(altered, source, "the control must actually alter the document");

    let report = Check_Against_Manifest(
        &lineage,
        &BTreeMap::from([("00-suite-index.md".to_owned(), altered)]),
    );

    assert!(!report.Is_Passing(), "an altered source must not pass the gate");
    assert!(
        report
            .mismatches
            .iter()
            .any(|mismatch| mismatch.Describe().contains("00-suite-index.md#")),
        "the failure must name the block: {:?}",
        report.mismatches
    );
}

/// Front matter is not hashed, so editing it does not move any block hash.
///
/// Correct — v14's manifest covers blocks, and front matter is not one — but it means
/// the preservation ledger cannot see a change to a document's `title`, `status` or
/// `authority`. Recorded here so the limit is a known one rather than a surprise when
/// something important is edited there and nothing notices.
#[test]
fn Test_Front_Matter_Should_Be_Outside_The_Gate()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");
    let source = Fixture("00-suite-index.md");
    let altered = source.replacen("status: accepted", "status: withdrawn", 1);
    assert_ne!(altered, source, "the fixture must contain the front-matter key");

    let report = Check_Against_Manifest(
        &lineage,
        &BTreeMap::from([("00-suite-index.md".to_owned(), altered)]),
    );

    assert!(
        report.Is_Passing(),
        "front matter is outside the block manifest by construction"
    );
}

#[test]
fn Test_Real_Statements_Should_Ingest_Without_Divergence()
{
    use nomos_spec_store::SpecificationStore;

    let file = Parse_Statements(&Fixture("statements-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");
    let mut store = SpecificationStore::In_Memory()
        .expect("an in-memory store opens over no file, so this construction has no failure path");

    let report = Ingest_Statements(&mut store, &file)
        .expect("the vendored statements are well formed, so ingestion reports rather than refuses");

    assert!(report.Is_Passing(), "{report:?}");
    assert_eq!(report.ingested, SLICE_STATEMENTS);
}

/// Non-ASCII statements pin the encoding. Under latin-1 or NFD these hash differently.
#[test]
fn Test_The_Statement_Slice_Should_Contain_Non_Ascii()
{
    let file = Parse_Statements(&Fixture("statements-slice.yaml")).expect("the vendored slice is the YAML v14 wrote, so the parser reads it");

    let non_ascii = file
        .statements
        .iter()
        .filter(|statement| !statement.canonical_text.is_ascii())
        .count();

    assert!(non_ascii >= NON_ASCII_FLOOR, "only {non_ascii} non-ascii statements in the slice");
}
