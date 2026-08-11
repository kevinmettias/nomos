//! P3-FAMILIES. I5: the families v15.0 destroyed, restored from v14.36.
//!
//! Every count this test asserts is read out of `tests/corpus/families/counts.json` rather
//! than written here. Per D-132 the plan's figures are lineage: 30 canonical domain models
//! is the pipe-line count of the table that lists 28 of them, and a restoration asserting
//! 30 would have to mint the header row as a concept to reach it. The register carries the
//! measurement and the extraction that produced it; this asserts the restoration agrees
//! with the register, so the two cannot drift apart without one of them going red.
//!
//! The second half is the preservation run. A restoration that added nodes and left blocks
//! undisposed would satisfy every assertion about identifiers while quietly breaking the
//! ledger the whole system exists for.

use nomos_spec_ingest::{
    BlockLineage, Ingest_Block_Dispositions, Ingest_Section_Lineage, Ingest_Source_Document,
    Parse_Block_Lineage, Parse_Section_Lineage, Resolve, RestorationReport, Restore, Restored,
};
use nomos_spec_store::{SpecificationStore, Table};
use nomos_spec_validate::{RuleOutcome, Validate, ValidationRun};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const REGISTER: &str = include_str!("../../../../tests/corpus/families/counts.json");

const REVISION: &str = "v14.36";

/// The register entry that states each family's membership.
///
/// Named per family rather than derived, because "how many members does this family have"
/// is a different question from "how many pipe lines does its table hold" and the register
/// carries both. Picking the wrong one is exactly the confusion D-132 exists to stop, so
/// the choice is written down where it can be read.
const MEMBERSHIP: &[(Restored, &str)] = &[
    (Restored::RoadmapMilestone, "roadmap.milestones"),
    (Restored::Scenario, "scenario.end_to_end"),
    (Restored::Service, "service.service_headings"),
    (Restored::AppendixD, "appendix_d.members"),
    (Restored::AppendixH, "appendix_h.sections"),
    (Restored::HeadlessInventory, "headless_inventory.sections"),
    (Restored::IdeProfile, "ide_profiles.sections"),
    (Restored::GlossaryTerm, "glossary.definitions"),
    (Restored::CanonicalDomainModel, "domain_model.named_models"),
];

/// The models the plan names by hand, and the two ends of the range it writes as
/// "through". If the restoration stops naming these, the plan's own examples no longer
/// resolve and the restoration is not the one the plan asked for.
const NAMED_MODELS: &[&str] = &[
    "WorkspaceContext",
    "FindingGeometry",
    "MetricTradeoffProjection",
    "ModelUsageObservation",
];

#[derive(Debug, Deserialize)]
struct Entry
{
    id: String,
    measured: u32,
}

fn Declared(id: &str) -> u32
{
    let entries: Vec<Entry> = serde_json::from_str(REGISTER).expect("the register parses");

    return entries
        .iter()
        .find(|entry| return entry.id == id)
        .map_or_else(|| panic!("{id} is not in the register"), |entry| return entry.measured);
}

fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    assert!(root.is_dir(), "NOMOS_V14_CORPUS is not a directory: {}", root.display());
    return Some(root);
}

fn Read(root: &Path, relative: &str) -> String
{
    let path = root.join(relative);
    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

fn Volumes(root: &Path) -> BTreeMap<String, String>
{
    let directory = root.join("01_authoring/domain_volumes");
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut documents = BTreeMap::new();
    for entry in entries.flatten()
    {
        let Some(name) = Volume_Name(&entry.path())
        else
        {
            continue;
        };
        let text = Read(&directory, &name);

        documents.insert(name, text);
    }

    assert_eq!(documents.len(), 10, "the ten domain volumes are the corpus this restores from");
    return documents;
}

/// A restored store and the report the restoration produced.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct RestoredStore
{
    store: SpecificationStore,
    report: RestorationReport,
}

/// A domain volume's file name, or `None` for anything else in the directory.
fn Volume_Name(path: &Path) -> Option<String>
{
    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
    {
        return None;
    }

    return path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_owned);
}

/// Source truth, then the restoration on top of it.
fn Restored_Store(root: &Path) -> RestoredStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let documents = Volumes(root);

    for (name, markdown) in &documents
    {
        Ingest_Source_Document(&mut store, name, REVISION, markdown).expect("ingests");
    }

    let report =
        Restore(&mut store, REVISION, &documents).unwrap_or_else(|error| panic!("{error}"));

    Assert_The_Restoration_Owns_Its_Names(&report);

    return RestoredStore { store, report };
}

/// The restoration walked into no name another authority owns, and the names two members
/// share are the seven this build knows about.
///
/// Seven names are carried by two members each: the canonical domain model and the glossary
/// term of the same name. Both are real nodes and neither takes the bare name. Pinned rather
/// than tolerated — an eighth appearing is something to look at, not something to absorb.
fn Assert_The_Restoration_Owns_Its_Names(report: &RestorationReport)
{
    assert!(
        report.contested_aliases.is_empty(),
        "the restoration walked into names it does not own: {:?}",
        report.contested_aliases
    );
    assert_eq!(
        report.ambiguous_names,
        SHARED_NAMES.map(str::to_owned).to_vec(),
        "the set of names carried by two members changed"
    );
}

/// The names a canonical domain model and a glossary term both carry.
const SHARED_NAMES: [&str; 7] = [
    "Applicability",
    "Artifact",
    "Capability",
    "Gate",
    "Phase",
    "Snapshot",
    "Workflow",
];

#[test]
fn Test_Every_Family_Should_Restore_The_Count_The_Register_Declares()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore { report, .. } = Restored_Store(&root);
    for (family, id) in MEMBERSHIP
    {
        let restored = u32::try_from(report.In(*family).len()).unwrap_or(u32::MAX);
        assert_eq!(
            restored,
            Declared(id),
            "{}: the register's {id} declares {} and the restoration produced {restored}\n{}",
            family.Label(),
            Declared(id),
            report.Summary()
        );
    }

    assert_eq!(
        MEMBERSHIP.len(),
        Restored::All().len(),
        "a family has no declared membership, so its count would go unasserted"
    );
}

#[test]
fn Test_Every_Restored_Member_Should_Resolve_By_Its_Identifier()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore { store, report } = Restored_Store(&root);

    assert!(!report.members.is_empty(), "nothing was restored, so nothing was checked");

    let unresolved: Vec<&str> = report
        .members
        .iter()
        .filter(|member| return Resolve(&store, &member.id).expect("resolves").is_none())
        .map(|member| return member.id.as_str())
        .collect();

    assert!(unresolved.is_empty(), "restored and unresolvable: {unresolved:?}");
}

/// The done-when's own list. Each must resolve by the name the corpus uses, not only by
/// the identifier this build minted, and each must trace to the row it came from.
#[test]
fn Test_The_Canonical_Domain_Models_Should_Resolve_By_Name_And_Trace_To_Their_Rows()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore { store, report } = Restored_Store(&root);
    for name in NAMED_MODELS
    {
        Assert_Resolves_By_Its_Own_Name(&store, &report, name);
    }

    // Every model, not only the four the plan names: each traces to exactly one row, and
    // that row says the model's name. A lineage to the containing block would satisfy
    // "traces to something" and answer the wrong question.
    for member in report.In(Restored::CanonicalDomainModel)
    {
        let traced = Rows_Traced_To(&store, &member.id);

        assert_eq!(traced.len(), 1, "{} traces to {} rows", member.id, traced.len());
        assert!(
            traced.first().is_some_and(|text| return text.contains(&member.name)),
            "{} traces to a row that does not name it: {traced:?}",
            member.id
        );
    }
}

/// One named model is a canonical domain model, resolves by the name the corpus gives it,
/// and is not one of the names two members share.
fn Assert_Resolves_By_Its_Own_Name(
    store: &SpecificationStore,
    report: &RestorationReport,
    name: &str,
)
{
    let member = report
        .Named(name)
        .unwrap_or_else(|| panic!("{name} was not restored\n{}", report.Summary()));

    assert_eq!(member.family, Restored::CanonicalDomainModel, "{name}");
    assert!(
        Resolve(store, name).expect("resolves").is_some(),
        "{name} resolves only by the identifier this build minted"
    );
    assert!(
        !report.ambiguous_names.contains(&name.to_owned()),
        "{name} became ambiguous, so resolving it by name is answering one of two"
    );
}

/// The text of every table row a node's lineage points at.
fn Rows_Traced_To(store: &SpecificationStore, node_id: &str) -> Vec<String>
{
    return store
        .Connection()
        .prepare(
            "SELECT r.text FROM lineage l
             JOIN source_table_rows r ON r.uid = l.source_table_row_uid
             JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = ?1",
        )
        .and_then(|mut statement| {
            return statement
                .query_map(rusqlite::params![node_id], |row| return row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("queries");
}

/// Every family member traces to a source, and the two shapes stay distinct: a heading
/// member to its block, a row member to its row.
#[test]
fn Test_Every_Restored_Member_Should_Carry_A_Lineage_To_What_Produced_It()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore { store, report } = Restored_Store(&root);

    let untraced: Vec<&str> = report
        .members
        .iter()
        .filter(|member| return Sources_Behind(&store, &member.id) == 0)
        .map(|member| return member.id.as_str())
        .collect();

    assert!(untraced.is_empty(), "restored with no lineage: {untraced:?}");
}

/// How many blocks or table rows a node's lineage points back at.
fn Sources_Behind(store: &SpecificationStore, node_id: &str) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM lineage l JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = ?1
               AND (l.source_block_uid IS NOT NULL OR l.source_table_row_uid IS NOT NULL)",
            rusqlite::params![node_id],
            |row| return row.get(0),
        )
        .expect("queries");
}

/// The preservation half. A restoration that leaves the ledger broken has not restored
/// anything worth trusting.
#[test]
fn Test_The_Reconciled_Store_Should_Report_No_Preservation_Errors()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore {
        mut store, report
    } = Restored_Store(&root);
    Ingest_The_Lineage(&mut store, &root);

    let run = Validate(&store, &nomos_spec_validate::Registered());
    assert!(run.Errors().is_empty(), "{:?}", run.Errors());
    assert!(
        run.Violations().is_empty(),
        "{}\nfirst: {:?}",
        run.Summary(),
        run.Violations().iter().take(5).collect::<Vec<_>>()
    );
    assert!(run.Passed(), "{}", run.Summary());
    Assert_The_Preservation_Rules_Examined_Something(&run);
    // The two statement rules examined nothing, and this store is why: I2 is not part of
    // the restoration, so it holds no statements. Asserted rather than left to the vacuity
    // list, so the reason is recorded instead of inferred.
    assert_eq!(store.Count(Table::NormativeStatements).expect("counts"), 0);
    assert_eq!(
        run.Vacuous_Rules(),
        vec!["NSV-PRESERVE-003", "NSV-PRESERVE-006"],
        "a rule went vacuous for a reason this test does not account for"
    );
    assert!(!report.members.is_empty(), "the ledger is clean over a store with no restoration");
}

/// The section lineage and the block dispositions, on top of a restored store.
fn Ingest_The_Lineage(store: &mut SpecificationStore, root: &Path)
{
    let section_lineage = Read(root, "01_authoring/source_lineage/section-lineage.yaml");
    let sections = Parse_Section_Lineage(&section_lineage).expect("the section lineage parses");
    let headings = Ingest_Section_Lineage(store, &sections, REVISION).expect("ingests");
    let block_lineage = Read(root, "01_authoring/source_lineage/source-block-lineage.yaml");
    let manifest = Parse_Block_Lineage(&block_lineage).expect("the block manifest parses");

    assert!(headings.Passed(), "{:?}", headings.unknown_documents);
    for (document, dispositions) in Per_Document(&manifest)
    {
        Ingest_Block_Dispositions(store, &document, REVISION, &dispositions)
            .unwrap_or_else(|error| panic!("{document}: {error}"));
    }
}

/// The manifest's blocks, gathered under the document each one came from.
fn Per_Document(manifest: &BlockLineage) -> BTreeMap<String, Vec<(u32, String)>>
{
    let mut per_document: BTreeMap<String, Vec<(u32, String)>> = BTreeMap::new();

    for block in &manifest.blocks
    {
        per_document
            .entry(block.source_document.clone())
            .or_default()
            .push((block.block_ordinal, block.disposition.clone()));
    }

    return per_document;
}

/// Non-zero subject counts, per rule.
///
/// "0 violations over 0 subjects" and "0 violations over 2533" print the same and mean
/// opposite things.
fn Assert_The_Preservation_Rules_Examined_Something(run: &ValidationRun)
{
    for rule in ["NSV-PRESERVE-001", "NSV-PRESERVE-002"]
    {
        let checked = run
            .results
            .iter()
            .find(|result| return result.id == rule)
            .map(|result| return result.outcome.clone());

        assert!(
            matches!(checked, Some(RuleOutcome::Satisfied { checked }) if checked > 0),
            "{rule} concluded nothing was wrong having examined nothing: {checked:?}"
        );
    }
}

#[test]
fn Test_Restoring_The_Whole_Corpus_Twice_Should_Change_Nothing()
{
    let Some(root) = Corpus()
    else
    {
        return;
    };
    let RestoredStore {
        mut store,
        report: first,
    } = Restored_Store(&root);
    let nodes = store.Count(Table::Nodes).expect("counts");
    let lineage = store.Count(Table::Lineage).expect("counts");
    let aliases = store.Count(Table::NodeAliases).expect("counts");

    let again = Restore(&mut store, REVISION, &Volumes(&root)).expect("restores again");
    assert!(again.contested_aliases.is_empty(), "{:?}", again.contested_aliases);

    assert!(nodes > 0 && lineage > 0 && aliases > 0, "the first run wrote nothing");
    assert_eq!(store.Count(Table::Nodes).expect("counts"), nodes);
    assert_eq!(store.Count(Table::Lineage).expect("counts"), lineage);
    assert_eq!(store.Count(Table::NodeAliases).expect("counts"), aliases);
    assert_eq!(first.members.len(), Restored_Store(&root).report.members.len());
}
