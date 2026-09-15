//! I1's hard stop, against the real v14 corpus.
//!
//! Opt-in by `NOMOS_V14_CORPUS`, and loud rather than silent: a configured corpus that
//! cannot be read fails. A gate that quietly skips its own subject is worse than one
//! that fails.

use nomos_spec_ingest::{
    BlockLineage, Check_Against_Manifest, GateReport, Ingest_Catalog, Ingest_Source_Document,
    Ingest_Statements, Parse_Block_Lineage, Parse_Catalog, Parse_Statements, RecordedBlock,
};
use nomos_spec_store::{SpecificationStore, Table};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Every block the v14 lineage records, and how many a full ingest must produce.
const RECORDED_BLOCK_COUNT: u32 = 2535;

/// The recorded blocks whose content and normalized hashes differ, so they exercise the normalizer.
const DISCRIMINATING_BLOCK_COUNT: u32 = 31;

/// The statements the normative statement file records.
const STATEMENT_COUNT: u32 = 493;

/// The entities `catalog.json` resolves, which every ingested statement must attach to.
const CATALOG_ENTITY_COUNT: u32 = 2619;

/// The domain volumes a full ingest writes, one `SourceDocuments` row each.
const SOURCE_DOCUMENT_COUNT: u32 = 10;

/// The recorded blocks the five volumes `cace351ac` left untouched hold between them.
const UNTOUCHED_VOLUME_BLOCK_COUNT: usize = 929;

/// How many ingest passes prove the second is a no-op.
const RE_INGEST_PASSES: u32 = 2;

/// How many disagreements a failing report prints, enough to locate the first divergence.
const DISAGREEMENTS_SHOWN: usize = 10;

/// How many sample entries a divergence report prints before its count.
const SAMPLES_SHOWN: usize = 5;

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
        // Every caller names a file the v14 corpus is defined to carry: the block manifest,
        // the statement file, the catalog. Missing one means NOMOS_V14_CORPUS points at
        // something that is not that corpus, and this file is loud rather than silent by
        // design — a gate that quietly skips its own subject is worse than one that fails.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

fn Domain_Volumes(root: &Path) -> BTreeMap<String, String>
{
    let directory = root.join("01_authoring/domain_volumes");
    let entries = std::fs::read_dir(&directory)
        // The gate is defined over every recorded block, so it cannot run against whatever
        // subset of volumes happened to list. An empty listing would bring `blocks_checked`
        // in low against a manifest that still records 2535 and read as a hash mismatch.
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
        // The extension and the name have already been accepted, so this entry is a volume.
        // Returning `None` here is the one case the function's `None` must not cover: it
        // would drop a document the manifest still holds blocks for, and the gate would
        // report those blocks as changed rather than as a file it never opened.
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
        First_Disagreements(&report)
    );
    assert!(report.Is_Passing(), "{}", report.Summary());
    assert!(
        report.Has_Exercised_The_Normalizer(),
        "no block in this run discriminates the normalizer, so the run verified hashing \
         and not normalization: {}",
        report.Summary()
    );
    // Re-measured 2026-09-14 against the live corpus at code-standards `6e5eb14e2`,
    // from 2533. `cace351ac` split one service heading and its prose into two of each in
    // volume 02; `P102` refreshed `source-block-lineage.yaml` at its source before this
    // number moved, and the v14.36 archive still answers 2533 (`revisions.rs`).
    assert_eq!(report.blocks_checked, RECORDED_BLOCK_COUNT, "{}", report.Summary());
    // Re-measured 2026-09-14 from 30, alongside `blocks_checked` above and for the same
    // reason: one of the two blocks `cace351ac` added to volume 02 normalizes to different
    // bytes than it hashes, so it discriminates the normalizer where its predecessors did
    // not. The count rising is what keeps this assertion meaningful -- a refresh that left
    // it at 30 while adding blocks would be evidence the normalizer stopped being exercised.
    assert_eq!(report.discriminating_blocks, DISCRIMINATING_BLOCK_COUNT, "{}", report.Summary());
}

/// The first few disagreements, one per line, for a failing run's message.
///
/// Bounded because a manifest off by one block disagrees on every block after it; the
/// first handful is what locates the divergence, and `report.mismatches` still holds all
/// of them.
fn First_Disagreements(report: &GateReport) -> String
{
    return report
        .mismatches
        .iter()
        .take(DISAGREEMENTS_SHOWN)
        .map(nomos_spec_ingest::Mismatch::Describe)
        .collect::<Vec<String>>()
        .join("\n  ");
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
    let mut store = SpecificationStore::In_Memory().expect("an in-memory store needs no file to open");
    let report = Ingest_Statements(&mut store, &file).expect("the statement file came from the parser above");

    assert!(
        report.divergences.is_empty(),
        "{} statement(s) disagree with their own recorded hash: {:?}",
        report.divergences.len(),
        report.divergences.iter().take(SAMPLES_SHOWN).collect::<Vec<_>>()
    );
    assert!(
        report.non_canonical_text.is_empty(),
        "{} statement(s) are not fixed points: {:?}",
        report.non_canonical_text.len(),
        report.non_canonical_text.iter().take(SAMPLES_SHOWN).collect::<Vec<_>>()
    );
    assert_eq!(report.ingested, STATEMENT_COUNT);
    assert_eq!(
        store.Count(Table::NormativeStatements).expect("counts"),
        STATEMENT_COUNT
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
    let mut store = SpecificationStore::In_Memory().expect("an in-memory store needs no file to open");
    let blocks = Ingest_The_Volumes(&mut store, &root);
    let source = Read(&root, "01_authoring/source_lineage/normative-source-statements.yaml");
    let statements = Parse_Statements(&source).expect("the statement file is the corpus artifact this test read");
    Ingest_Statements(&mut store, &statements).expect("the statements came from the parser above");
    let catalog_json = Read(&root, "02_machine/catalog/catalog.json");
    let catalog = Parse_Catalog(&catalog_json).expect("the catalog is the corpus artifact this test read");
    let report = Ingest_Catalog(&mut store, &catalog).expect("the catalog came from the parser above");
    // Every statement resolves to a node, so nothing was ingested orphaned.
    let orphans = Orphaned_Statements(&store);

    assert_eq!(blocks, RECORDED_BLOCK_COUNT);
    assert_eq!(report.nodes, CATALOG_ENTITY_COUNT, "the catalog entity count changed");
    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), SOURCE_DOCUMENT_COUNT);
    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), RECORDED_BLOCK_COUNT);
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
            Ingest_Source_Document(store, name, "v14.36", markdown).expect("a domain volume of the v14 corpus");

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
        .expect("the store applied this schema, so the query resolves over it");
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
    let mut store = SpecificationStore::In_Memory().expect("an in-memory store needs no file to open");
    let documents = Domain_Volumes(&root);

    for _ in 0..RE_INGEST_PASSES
    {
        for (name, markdown) in &documents
        {
            Ingest_Source_Document(&mut store, name, "v14.36", markdown).expect("a domain volume of the v14 corpus");
        }
    }

    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), SOURCE_DOCUMENT_COUNT);
    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), RECORDED_BLOCK_COUNT);
}

/// The five volumes `cace351ac` did not touch, and why that list is the one worth naming.
///
/// `cace351ac` (2026-08-17) edited five of the ten domain volumes and nothing else. The
/// other five have not moved since the lineage was recorded, so for them the recorded
/// mechanical fields and a fresh segmentation must agree *exactly* -- and that agreement is
/// the only thing separating an emitter compatible with the segmenter
/// [`Test_I1_Should_Reproduce_Every_Recorded_Block`] judges against from a second segmenter
/// that merely agrees with itself. A refresh built on the second kind would make every
/// failing test pass by redefining what a block is, which is the outcome this file exists to
/// prevent rather than to produce.
const VOLUMES_THE_EDIT_DID_NOT_TOUCH: &[&str] = &[
    "00-suite-index.md",
    "01-product-definition-and-principles.md",
    "03-packages-providers-rules-and-applicability.md",
    "04-checks-gates-corrections-and-governance.md",
    "05-atlas-architecture-features-tests-runtime.md",
];

/// One block's mechanical fields, as a refreshed lineage would have to record them.
///
/// Mechanical is the whole point of the split: these five are derived from the text by
/// [`nomos_spec_model::Segment`] and nothing else, so a refresh may regenerate them. A
/// block's `disposition`, `authority`, `target_volumes` and `stable_ids` are authored
/// judgments that no segmentation can recover, and this function deliberately does not
/// produce them.
fn Emitted_Mechanical_Fields(markdown: &str) -> Vec<(u32, String, String, String)>
{
    return nomos_spec_model::Segment(markdown)
        .iter()
        .map(|block| {
            return (
                block.ordinal,
                nomos_spec_store::Kind_Label(block.kind).to_owned(),
                block.Content_Hash().As_String_Slice().to_owned(),
                block.Normalized_Hash().As_String_Slice().to_owned(),
            );
        })
        .collect();
}

/// The emitter reproduces what the lineage already records, for every volume whose text has
/// not moved since it was recorded.
///
/// `P102`'s own acceptance step 2. This must pass *before* any emitted value is written back
/// into the corpus: it is what makes the refresh a regeneration of fields the segmenter
/// already owns rather than a re-blessing of whatever the current segmenter happens to say.
#[test]
fn Test_The_Emitter_Should_Reproduce_The_Recorded_Fields_For_Volumes_The_Edit_Did_Not_Touch()
{
    let Some((lineage, documents)) = Loaded_Corpus()
    else
    {
        return;
    };

    let mut compared = 0_usize;
    for name in VOLUMES_THE_EDIT_DID_NOT_TOUCH
    {
        let pairs = Reproduced_Field_Pairs_In_Volume(*name, &documents, &lineage);
        compared = compared.saturating_add(pairs);
    }
    // Measured 2026-09-14 against the live corpus, not chosen: the five volumes above hold 929
    // recorded blocks between them, and every one of them reproduced. Pinned as an equality
    // rather than a floor so that a volume dropping out of `documents` -- renamed, unreadable,
    // or quietly excluded -- fails here instead of shrinking the proof and still passing.
    assert_eq!(
        compared, UNTOUCHED_VOLUME_BLOCK_COUNT,
        "the five untouched volumes hold 929 blocks between them; comparing {compared} means \
         the proof no longer covers what it was measured over"
    );
}

/// The recorded lineage and every domain volume, or `None` when no corpus is configured.
fn Loaded_Corpus() -> Option<(BlockLineage, BTreeMap<String, String>)>
{
    let Some(root) = Corpus()
    else
    {
        return None;
    };
    let manifest = Read(&root, "01_authoring/source_lineage/source-block-lineage.yaml");
    let lineage = Parse_Block_Lineage(&manifest).expect("the manifest parses");
    let documents = Domain_Volumes(&root);
    return Some((lineage, documents));
}

/// How many block pairs one untouched volume's emitted fields reproduced against its records.
///
/// The volume must be among the documents: a name that dropped out of the corpus would
/// otherwise shrink the proof silently instead of failing, which is the outcome the caller's
/// equality over `UNTOUCHED_VOLUME_BLOCK_COUNT` exists to prevent.
fn Reproduced_Field_Pairs_In_Volume(
    name: &str,
    documents: &BTreeMap<String, String>,
    lineage: &BlockLineage,
) -> usize
{
    let markdown = documents
        .get(name)
        .unwrap_or_else(|| panic!("{name} is not among the domain volumes"));
    let recorded: Vec<&RecordedBlock> =
        lineage.blocks.iter().filter(|block| return block.source_document == name).collect();
    let emitted = Emitted_Mechanical_Fields(markdown);
    assert_eq!(
        emitted.len(),
        recorded.len(),
        "{name}: the lineage records {} blocks and segmentation produces {}, for a volume \
         cace351ac never edited -- so the emitter disagrees with the segmenter rather than \
         the corpus having moved",
        recorded.len(),
        emitted.len()
    );

    let mut compared = 0_usize;
    for (want, got) in recorded.iter().zip(&emitted)
    {
        Assert_Fields_Match(name, want, got);
        compared = compared.saturating_add(1);
    }
    return compared;
}

/// Asserts one block's emitted mechanical fields equal what the lineage records for it.
fn Assert_Fields_Match(name: &str, want: &RecordedBlock, got: &(u32, String, String, String))
{
    let (ordinal, kind, content_hash, normalized_hash) = got;
    assert_eq!(
        (want.block_ordinal, want.block_kind.as_str(), want.content_hash.as_str(), want.normalized_hash.as_str()),
        (*ordinal, kind.as_str(), content_hash.as_str(), normalized_hash.as_str()),
        "{name}#{}: a volume the edit never touched must emit exactly what it records",
        want.block_ordinal
    );
}
