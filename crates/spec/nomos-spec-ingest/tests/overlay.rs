//! P3-OVERLAY. I4: the v15.0 overlay, merged block by block.
//!
//! Merge is block-level because 132 v15 files mix real prose with filler, so preferring
//! v14 whole-file would discard real v15 improvements along with the stubs. Filler is
//! ingested rather than dropped: a stub is the evidence of what was lost, and discarding
//! it leaves the regression report with nothing to point at.
//!
//! Every count here is measured against the archives. Where the plan states a different
//! number, the file wins and the disagreement is named — the plan's figures are the
//! estimates that motivated the work, not readings.

use nomos_spec_ingest::{
    Archive, Disposition, Family, Ingest_Overlay_Document, Ingest_V15_Record, Get_Filler_Pattern, Overlaid,
    OverlayReport, Parse_Artifact, Reconcile_Artifacts, ReconciliationReport, Statements_In,
};
use nomos_spec_store::{SpecificationStore, Table};
use std::collections::BTreeMap;
use std::path::PathBuf;

const V15: &str = "nomos-spec-v15.0.zip";

fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    assert!(root.is_dir(), "NOMOS_V14_CORPUS is not a directory: {}", root.display());
    return Some(root);
}

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

/// Both are needed, and having one without the other is a misconfiguration rather than a
/// reason to skip: it would silently test half of a comparison.
fn Both() -> Option<(PathBuf, Archive)>
{
    let (Some(corpus), Some(archives)) = (Corpus(), Archives())
    else
    {
        assert!(
            Corpus().is_none() && Archives().is_none(),
            "one of NOMOS_V14_CORPUS and NOMOS_SPEC_ARCHIVES is set without the other, so \
             the overlay would compare against nothing"
        );
        return None;
    };

    // `None` from here means the deliberate no-corpus case and nothing else. Turning a v15.0 that
    // will not open into that same `None` is the half comparison this function's doc refuses,
    // arriving by another route and looking like a skip.
    let archive = Archive::Open(&archives.join(V15)).unwrap_or_else(|error| panic!("{error}"));
    return Some((corpus, archive));
}

fn V14_Artifacts(corpus: &std::path::Path) -> Vec<nomos_spec_ingest::Artifact>
{
    let mut artifacts = Vec::new();

    for family in Family::All()
    {
        let directory = corpus.join("01_authoring/artifacts").join(family.Directory());
        let entries = std::fs::read_dir(&directory)
            // A family directory that will not read contributes no artifacts, and the reconciler
            // then reports that whole family as absent in v15 — the loss this file exists to
            // measure, manufactured by the reader instead of found in the archive.
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

        for entry in entries.flatten()
        {
            let read = Artifact_At(&entry.path(), *family);

            artifacts.extend(read);
        }
    }

    artifacts.sort_by(|left, right| return left.id.cmp(&right.id));
    return artifacts;
}

/// One artifact file, parsed under its family, or `None` for anything that is not one.
fn Artifact_At(path: &std::path::Path, family: Family) -> Option<nomos_spec_ingest::Artifact>
{
    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
    {
        return None;
    }
    let text = std::fs::read_to_string(path)
        // One unreadable file is one v14 identifier missing from the comparison, and the 689
        // count would report that as the artifact count having changed.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    return Some(
        Parse_Artifact(&text, family)
            // `None` from this function already means "not an artifact file", so returning it on
            // a parse failure would file malformed front matter under nothing-to-see. The path is
            // the only part that makes it fixable.
            .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    );
}

fn V15_Statements(archive: &mut Archive) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for entry in archive.Listing().Ending_With(".md")
    {
        let text = archive
            .Read_Text(&entry)
            // The entry came out of this archive's own listing, so a failed read means the two
            // disagree. Skipping it would delete v15 statements, and the reconciler would then
            // report every identifier they carry as lost.
            .unwrap_or_else(|error| panic!("{error}"));
        found.extend(Statements_In(&text));
    }
    return found;
}

/// The done-when's first clause, reported per identifier.
#[test]
fn Test_Every_Identifier_Should_Reconcile_By_Name()
{
    let Some((corpus, mut archive)) = Both()
    else
    {
        return;
    };
    let v14 = V14_Artifacts(&corpus);
    let v15 = V15_Statements(&mut archive);
    let report: ReconciliationReport = Reconcile_Artifacts(&v14, &v15);
    let absent = report.Absent_In(Family::Acceptance);

    assert_eq!(v14.len(), 689, "the v14 artifact count changed");
    assert_eq!(report.Declared_In(Family::Requirement), 363);
    assert_eq!(report.Declared_In(Family::Story), 169);
    assert_eq!(report.Declared_In(Family::Acceptance), 157);
    // Requirements and stories came through whole, and their wording did not drift.
    assert_eq!(report.Preserved_In(Family::Requirement), 363, "{}", report.Summary());
    assert_eq!(report.Preserved_In(Family::Story), 169, "{}", report.Summary());
    assert!(
        report.Reworded().is_empty(),
        "wording drifted: {:?}",
        report.Reworded().iter().take(5).collect::<Vec<_>>()
    );
    // The whole acceptance family is gone. Reported as absent, not as reworded, and named
    // rather than counted.
    assert_eq!(report.Preserved_In(Family::Acceptance), 0, "{}", report.Summary());
    assert_eq!(absent.len(), 157, "{}", report.Summary());
    assert!(
        absent.contains(&"US-AGT-001-AC"),
        "the report does not name the identifiers it lost"
    );
    assert_eq!(
        report.Absent().len(),
        157,
        "something outside the acceptance family also disappeared: {}",
        report.Summary()
    );
}

/// Named, never counted. A report that can only say "157" cannot answer "which".
#[test]
fn Test_The_Report_Should_Name_What_It_Lost()
{
    let Some((corpus, mut archive)) = Both()
    else
    {
        return;
    };
    let report = Reconcile_Artifacts(&V14_Artifacts(&corpus), &V15_Statements(&mut archive));

    let spelled = report.Summary();
    assert!(spelled.contains("acceptance"), "{spelled}");
    assert!(spelled.contains("-AC"), "the summary gives a count with no identifier: {spelled}");
    assert_eq!(
        report.outcomes.len(),
        689,
        "the report holds fewer outcomes than there are identifiers, so some were summarised away"
    );
}

/// Filler is stored and dispositioned, not discarded.
#[test]
fn Test_Every_Filler_Block_Should_Carry_A_Lineage_Row()
{
    let Some((corpus, mut archive)) = Both()
    else
    {
        return;
    };
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let headings = V14_Headings(&corpus);
    let mut report = OverlayReport::default();
    for entry in archive.Listing().Ending_With(".md")
    {
        if !entry.contains("/records/")
        {
            Overlay_One(&mut store, &mut archive, &entry, (&headings, &mut report));
        }
    }
    let rows = Filler_Lineage_Rows(&store);
    let displaced = report.filler.iter().filter(|block| block.displaced.is_some()).count();

    assert_eq!(report.documents, 206, "the v15 non-record document count changed");
    assert!(!report.filler.is_empty(), "no filler was found, so this test examined nothing");
    assert_eq!(
        rows,
        u32::try_from(report.filler.len()).unwrap_or(u32::MAX),
        "a filler block was reported without a lineage row"
    );
    Assert_Every_Judgement_Names_Its_Pattern(&report);
    // And most of them name the v14 heading they displaced.
    assert!(
        displaced > 0,
        "no filler block names what it displaced, so the lineage points nowhere"
    );
}

/// Every filler row names the pattern that judged it, so the judgement is checkable.
fn Assert_Every_Judgement_Names_Its_Pattern(report: &OverlayReport)
{
    for block in &report.filler
    {
        assert!(
            Get_Filler_Pattern(block.pattern).is_some(),
            "{} block {} was judged by a pattern that is not one",
            block.document,
            block.ordinal
        );
    }
}

/// One overlaid document, read out of the archive and ingested against the v14 headings.
fn Overlay_One(
    store: &mut SpecificationStore,
    archive: &mut Archive,
    entry: &str,
    against: (&BTreeMap<String, i64>, &mut OverlayReport),
)
{
    let (headings, report) = against;
    // A document that will not read is one the overlay never sees, and `report.documents` would
    // present the shortfall as v15's non-record document count having changed.
    let text = archive.Read_Text(entry).unwrap_or_else(|error| panic!("{error}"));
    let document = Overlaid {
        path: entry,
        markdown: &text,
    };

    Ingest_Overlay_Document(store, &document, headings, report)
        // An ingest that refused wrote neither the filler judgement nor its lineage row, so the
        // two would still agree in count while this document went unexamined. The entry says
        // which of the 206 it was.
        .unwrap_or_else(|error| panic!("{entry}: {error}"));
}

/// How many lineage rows a filler judgement wrote.
fn Filler_Lineage_Rows(store: &SpecificationStore) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM lineage WHERE disposition LIKE 'regression-filler:%'",
            [],
            |row| return row.get(0),
        )
        .expect("queries");
}

/// The v15-only records, as authored nodes.
#[test]
fn Test_The_V15_Records_Should_Be_Ingested_As_Authored_Nodes()
{
    let Some((_, mut archive)) = Both()
    else
    {
        return;
    };
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let ingested = Ingest_Every_V15_Record(&mut store, &mut archive);

    // The plan says 64. The archive holds 66 — 54 under records/ and 12 under
    // spec-governance/records/ — and the file wins, as it did for the catalog's 2,619
    // against the plan's 1,632.
    Assert_The_Record_Set_Is_Complete(&ingested, &store);
    Assert_Governing_Records_Are_Present(&store);
}

/// Every record under `records/` in the archive, ingested as an authored node.
fn Ingest_Every_V15_Record(store: &mut SpecificationStore, archive: &mut Archive) -> Vec<String>
{
    let mut ingested = Vec::new();
    for entry in archive.Listing().Ending_With(".md")
    {
        if entry.contains("/records/")
        {
            // A record that will not read is a node the store never receives, and the count the
            // caller checks would read that as the v15 record set having changed size.
            let text = archive.Read_Text(&entry).unwrap_or_else(|error| panic!("{error}"));
            let record = Ingest_V15_Record(store, &entry, &text)
                // Named per entry, because the check at the end of this test can only say that a
                // governing record is not in the store — never that its ingest refused.
                .unwrap_or_else(|error| panic!("{entry}: {error}"));

            ingested.push(record);
        }
    }

    return ingested;
}

/// The record set ingested is exactly 66 records, each with a distinct identifier, and the
/// store received one node per record.
fn Assert_The_Record_Set_Is_Complete(ingested: &[String], store: &SpecificationStore)
{
    assert_eq!(ingested.len(), 66, "the v15 record count changed");
    assert_eq!(
        ingested.iter().collect::<std::collections::BTreeSet<_>>().len(),
        66,
        "two records share an identifier"
    );
    assert_eq!(store.Count(Table::Nodes).expect("counts"), 66);
}

/// The records this system's own governance depends on are among the ones ingested.
fn Assert_Governing_Records_Are_Present(store: &SpecificationStore)
{
    for required in ["ADR-ARTIFACT-GRAPH-002", "D-117", "D-120", "D-127", "D-128"]
    {
        assert!(
            store.Node_Uid(required).expect("queries").is_some(),
            "{required} governs this system and is not in the store"
        );
    }
}

fn V14_Headings(corpus: &std::path::Path) -> BTreeMap<String, i64> {
    let directory = corpus.join("01_authoring/domain_volumes");
    let entries = std::fs::read_dir(&directory)
        // The assertion at the end of this function catches an empty heading set, but it would
        // blame the corpus for holding no headings when the directory simply would not read.
        // Those are two different repairs.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut headings = BTreeMap::new();
    for entry in entries.flatten()
    {
        Note_Headings_In(&entry.path(), &mut headings);
    }

    assert!(!headings.is_empty(), "no v14 headings, so nothing can be displaced");
    return headings;
}

/// Every heading one volume declares, filed under the order it was first seen in.
fn Note_Headings_In(path: &std::path::Path, headings: &mut BTreeMap<String, i64>)
{
    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
    {
        return;
    }
    let text = std::fs::read_to_string(path)
        // One volume's headings quietly absent still leaves `displaced > 0` true on the other
        // nine, so every filler block that displaced this volume would read as displacing nothing.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    for line in text.split('\n')
    {
        if line.starts_with('#')
        {
            let count = i64::try_from(headings.len()).unwrap_or(0);
            headings.entry(line.trim_end().to_owned()).or_insert(count);
        }
    }
}

/// The always-on half: the reconciler's three arms, without the corpus.
#[test]
fn Test_Absent_And_Reworded_Should_Not_Collapse()
{
    let artifact = Parse_Artifact(
        "---\nid: X-001\nstatement: X-001 A thing shall hold.\n---\n\n# X-001 - A thing\n",
        Family::Requirement,
    )
    .expect("reads");

    let absent = Reconcile_Artifacts(std::slice::from_ref(&artifact), &BTreeMap::new());
    assert_eq!(absent.Absent().len(), 1);
    assert!(absent.Reworded().is_empty());

    let mut changed = BTreeMap::new();
    changed.insert("X-001".to_owned(), "X-001 A different thing shall hold.".to_owned());
    let reworded = Reconcile_Artifacts(std::slice::from_ref(&artifact), &changed);
    assert!(reworded.Absent().is_empty());
    assert_eq!(reworded.Reworded().len(), 1);
    assert!(matches!(
        reworded.Reworded().first().map(|outcome| &outcome.disposition),
        Some(Disposition::Reworded { .. })
    ));
}
