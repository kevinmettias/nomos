//! The I1 gate over a real slice of the v14 corpus, vendored so it always runs.
//!
//! `real_corpus.rs` covers the whole 2533-block manifest and is opt-in by path. This
//! file exists because an opt-in test reports `ok` when the path is unset, and `nomos
//! work finish` would accept that as the gate having passed. A vendored slice is real
//! recorded data: the hashes below were produced by v14's Python toolchain, not by us.

use nomos_spec_ingest::{Check_Against_Manifest, Ingest_Statements, Parse_Block_Lineage, Parse_Statements};
use std::collections::BTreeMap;

fn Fixture(name: &str) -> String
{
    use std::path::Path;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus").join(name);
    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the gate needs {}: {error}", path.display()));
}

fn Documents() -> BTreeMap<String, String>
{
    return BTreeMap::from([("00-suite-index.md".to_owned(), Fixture("00-suite-index.md"))]);
}

#[test]
fn Test_The_Gate_Should_Pass_Against_Real_Recorded_Blocks()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("parses");

    let report = Check_Against_Manifest(&lineage, &Documents());

    assert!(
        report.mismatches.is_empty(),
        "{}\n  {}",
        report.Summary(),
        report
            .mismatches
            .iter()
            .take(5)
            .map(nomos_spec_ingest::BlockMismatch::Describe)
            .collect::<Vec<String>>()
            .join("\n  ")
    );
    assert!(report.Passed());
    assert_eq!(report.blocks_checked, 46, "{}", report.Summary());
    assert_eq!(report.documents_checked, 1);
}

/// This slice contains no table, so it verifies hashing and segmentation and *not*
/// normalization. The report has to say so rather than reporting an unqualified pass —
/// which is the whole reason `Exercised_The_Normalizer` is separate from `Passed`.
#[test]
fn Test_This_Slice_Should_Not_Claim_To_Exercise_The_Normalizer()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("parses");

    let report = Check_Against_Manifest(&lineage, &Documents());

    assert!(report.Passed());
    assert!(
        !report.Exercised_The_Normalizer(),
        "no block in this slice discriminates, so the run must not claim normalization \
         was verified"
    );
}

/// The negative control. One altered byte of body text and the gate must name the block.
#[test]
fn Test_An_Altered_Source_Should_Fail_The_Gate()
{
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("parses");
    let source = Fixture("00-suite-index.md");
    let altered = source.replacen("Focused", "Focussed", 1);
    assert_ne!(altered, source, "the control must actually alter the document");

    let report = Check_Against_Manifest(
        &lineage,
        &BTreeMap::from([("00-suite-index.md".to_owned(), altered)]),
    );

    assert!(!report.Passed(), "an altered source must not pass the gate");
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
    let lineage = Parse_Block_Lineage(&Fixture("block-lineage-slice.yaml")).expect("parses");
    let source = Fixture("00-suite-index.md");
    let altered = source.replacen("status: accepted", "status: withdrawn", 1);
    assert_ne!(altered, source, "the fixture must contain the front-matter key");

    let report = Check_Against_Manifest(
        &lineage,
        &BTreeMap::from([("00-suite-index.md".to_owned(), altered)]),
    );

    assert!(
        report.Passed(),
        "front matter is outside the block manifest by construction"
    );
}

#[test]
fn Test_Real_Statements_Should_Ingest_Without_Divergence()
{
    use nomos_spec_store::SpecificationStore;

    let file = Parse_Statements(&Fixture("statements-slice.yaml")).expect("parses");
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let report = Ingest_Statements(&mut store, &file).expect("ingests");

    assert!(report.Passed(), "{report:?}");
    assert_eq!(report.ingested, 16);
}

/// Non-ASCII statements pin the encoding. Under latin-1 or NFD these hash differently.
#[test]
fn Test_The_Statement_Slice_Should_Contain_Non_Ascii()
{
    let file = Parse_Statements(&Fixture("statements-slice.yaml")).expect("parses");

    let non_ascii = file
        .statements
        .iter()
        .filter(|statement| !statement.canonical_text.is_ascii())
        .count();

    assert!(non_ascii >= 5, "only {non_ascii} non-ascii statements in the slice");
}
