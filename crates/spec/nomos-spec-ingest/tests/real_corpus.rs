//! I1's hard stop, against the real v14 corpus.
//!
//! Opt-in by `NOMOS_V14_CORPUS`, and loud rather than silent: a configured corpus that
//! cannot be read fails. A gate that quietly skips its own subject is worse than one
//! that fails.

use nomos_spec_ingest::{
    Check_Against_Manifest, Ingest_Catalog, Ingest_Source_Document, Ingest_Statements,
    Parse_Block_Lineage, Parse_Catalog, Parse_Statements,
};
use nomos_spec_store::{SpecificationStore, Table};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    assert!(
        root.is_dir(),
        "NOMOS_V14_CORPUS is set to {}, which is not a directory",
        root.display()
    );
    return Some(root);
}

fn Read(root: &Path, relative: &str) -> String
{
    let path = root.join(relative);
    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

fn Domain_Volumes(root: &Path) -> BTreeMap<String, String>
{
    let directory = root.join("01_authoring/domain_volumes");
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut documents = BTreeMap::new();
    for entry in entries.flatten()
    {
        let path = entry.path();
        let Some(named) = Volume_At(&path)
        else
        {
            continue;
        };

        documents.insert(named.0, named.1);
    }

    assert!(!documents.is_empty(), "no domain volumes under {}", directory.display());
    return documents;
}

/// The gate the plan calls a hard stop: recompute every block hash and compare against
/// A domain volume's name and text, or `None` for anything else in the directory.
fn Volume_At(path: &Path) -> Option<(String, String)>
{
    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
    {
        return None;
    }
    let name = path.file_name().and_then(std::ffi::OsStr::to_str)?.to_owned();
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    return Some((name, text));
}

/// the real 1.5 MB manifest.
#[test]
fn Test_I1_Should_Reproduce_Every_Recorded_Block()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let manifest = Read(&root, "01_authoring/source_lineage/source-block-lineage.yaml");
    let lineage = Parse_Block_Lineage(&manifest).expect("the manifest parses");
    let documents = Domain_Volumes(&root);
    let report = Check_Against_Manifest(&lineage, &documents);

    assert!(
        report.mismatches.is_empty(),
        "{}\nfirst disagreements:\n  {}",
        report.Summary(),
        report
            .mismatches
            .iter()
            .take(10)
            .map(nomos_spec_ingest::BlockMismatch::Describe)
            .collect::<Vec<String>>()
            .join("\n  ")
    );
    assert!(report.Passed(), "{}", report.Summary());
    assert!(
        report.Exercised_The_Normalizer(),
        "no block in this run discriminates the normalizer, so the run verified hashing \
         and not normalization: {}",
        report.Summary()
    );
    assert_eq!(report.blocks_checked, 2533, "{}", report.Summary());
    assert_eq!(report.discriminating_blocks, 30, "{}", report.Summary());
}

/// I2 over the real statement file. Every recorded hash must match its recorded text,
/// and every text must be a fixed point.
#[test]
fn Test_I2_Should_Ingest_Every_Statement_Without_Divergence()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let source = Read(&root, "01_authoring/source_lineage/normative-source-statements.yaml");
    let file = Parse_Statements(&source).expect("the statement file parses");
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let report = Ingest_Statements(&mut store, &file).expect("ingests");

    assert!(
        report.divergences.is_empty(),
        "{} statement(s) disagree with their own recorded hash: {:?}",
        report.divergences.len(),
        report.divergences.iter().take(5).collect::<Vec<_>>()
    );
    assert!(
        report.non_canonical_text.is_empty(),
        "{} statement(s) are not fixed points: {:?}",
        report.non_canonical_text.len(),
        report.non_canonical_text.iter().take(5).collect::<Vec<_>>()
    );
    assert_eq!(report.ingested, 493);
    assert_eq!(
        store.Count(Table::NormativeStatements).expect("counts"),
        493
    );
}

/// I0 through I3 end to end, into one store.
#[test]
fn Test_The_Whole_Corpus_Should_Ingest_Into_One_Store()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let blocks = Ingest_The_Volumes(&mut store, &root);
    let source = Read(&root, "01_authoring/source_lineage/normative-source-statements.yaml");
    let statements = Parse_Statements(&source).expect("parses");
    Ingest_Statements(&mut store, &statements).expect("ingests");
    let catalog_json = Read(&root, "02_machine/catalog/catalog.json");
    let catalog = Parse_Catalog(&catalog_json).expect("parses");
    let report = Ingest_Catalog(&mut store, &catalog).expect("ingests");
    // Every statement resolves to a node, so nothing was ingested orphaned.
    let orphans = Orphaned_Statements(&store);

    assert_eq!(blocks, 2533);
    assert_eq!(report.nodes, 2619, "the catalog entity count changed");
    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 10);
    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), 2533);
    assert_eq!(orphans, 0);
}

/// Every domain volume, ingested, and how many blocks they came to.
fn Ingest_The_Volumes(store: &mut SpecificationStore, root: &Path) -> u32
{
    let documents = Domain_Volumes(root);
    let mut blocks = 0_u32;

    for (name, markdown) in &documents
    {
        let ingested =
            Ingest_Source_Document(store, name, "v14.36", markdown).expect("ingests");

        blocks = blocks.saturating_add(ingested);
    }

    return blocks;
}

/// How many statements resolve to no node at all.
fn Orphaned_Statements(store: &SpecificationStore) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM normative_statements s
             LEFT JOIN nodes n ON n.uid = s.node_uid WHERE n.uid IS NULL",
            [],
            |row| return row.get(0),
        )
        .expect("queries");
}

/// Re-ingesting the whole corpus must change nothing.
#[test]
fn Test_Re_Ingesting_The_Corpus_Should_Be_A_No_Op()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let documents = Domain_Volumes(&root);

    for _ in 0..2
    {
        for (name, markdown) in &documents
        {
            Ingest_Source_Document(&mut store, name, "v14.36", markdown).expect("ingests");
        }
    }

    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 10);
    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), 2533);
}
