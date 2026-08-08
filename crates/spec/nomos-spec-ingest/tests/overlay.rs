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
    Archive, Disposition, Family, Ingest_Overlay_Document, Ingest_v15_Record, Is_Filler,
    OverlayReport, Parse_Artifact, Reconcile, ReconciliationReport, Statements_In,
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
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

        for entry in entries.flatten()
        {
            let path = entry.path();
            if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            artifacts.push(
                Parse_Artifact(&text, *family)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
            );
        }
    }

    artifacts.sort_by(|left, right| return left.id.cmp(&right.id));
    return artifacts;
}

fn V15_Statements(archive: &mut Archive) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for entry in archive.Ending_With(".md")
    {
        let text = archive
            .Read_Text(&entry)
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
    assert_eq!(v14.len(), 689, "the v14 artifact count changed");

    let report: ReconciliationReport = Reconcile(&v14, &V15_Statements(&mut archive));

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
    let absent = report.Absent_In(Family::Acceptance);
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

    let report = Reconcile(&V14_Artifacts(&corpus), &V15_Statements(&mut archive));

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

    for entry in archive.Ending_With(".md")
    {
        if entry.contains("/records/")
        {
            continue;
        }
        let text = archive.Read_Text(&entry).unwrap_or_else(|error| panic!("{error}"));
        Ingest_Overlay_Document(&mut store, &entry, &text, &headings, &mut report)
            .unwrap_or_else(|error| panic!("{entry}: {error}"));
    }

    assert_eq!(report.documents, 206, "the v15 non-record document count changed");
    assert!(!report.filler.is_empty(), "no filler was found, so this test examined nothing");

    let rows: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM lineage WHERE disposition LIKE 'regression-filler:%'",
            [],
            |row| row.get(0),
        )
        .expect("queries");
    assert_eq!(
        rows,
        u32::try_from(report.filler.len()).unwrap_or(u32::MAX),
        "a filler block was reported without a lineage row"
    );

    // Every row names the pattern that judged it, so the judgement is checkable.
    for block in &report.filler
    {
        assert!(
            Is_Filler(block.pattern).is_some(),
            "{} block {} was judged by a pattern that is not one",
            block.document,
            block.ordinal
        );
    }

    // And most of them name the v14 heading they displaced.
    let displaced = report.filler.iter().filter(|block| block.displaced.is_some()).count();
    assert!(
        displaced > 0,
        "no filler block names what it displaced, so the lineage points nowhere"
    );
}

/// The v15-only records, as authored nodes.
#[test]
fn Test_The_v15_Records_Should_Be_Ingested_As_Authored_Nodes()
{
    let Some((_, mut archive)) = Both()
    else
    {
        return;
    };

    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut ingested = Vec::new();

    for entry in archive.Ending_With(".md")
    {
        if !entry.contains("/records/")
        {
            continue;
        }
        let text = archive.Read_Text(&entry).unwrap_or_else(|error| panic!("{error}"));
        ingested.push(
            Ingest_v15_Record(&mut store, &entry, &text)
                .unwrap_or_else(|error| panic!("{entry}: {error}")),
        );
    }

    // The plan says 64. The archive holds 66 — 54 under records/ and 12 under
    // spec-governance/records/ — and the file wins, as it did for the catalog's 2,619
    // against the plan's 1,632.
    assert_eq!(ingested.len(), 66, "the v15 record count changed");
    assert_eq!(
        ingested.iter().collect::<std::collections::BTreeSet<_>>().len(),
        66,
        "two records share an identifier"
    );
    assert_eq!(store.Count(Table::Nodes).expect("counts"), 66);

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
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut headings = BTreeMap::new();
    for entry in entries.flatten()
    {
        let path = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some("md")
        {
            continue;
        }
        let text = std::fs::read_to_string(&path)
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

    assert!(!headings.is_empty(), "no v14 headings, so nothing can be displaced");
    return headings;
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

    let absent = Reconcile(std::slice::from_ref(&artifact), &BTreeMap::new());
    assert_eq!(absent.Absent().len(), 1);
    assert!(absent.Reworded().is_empty());

    let mut changed = BTreeMap::new();
    changed.insert("X-001".to_owned(), "X-001 A different thing shall hold.".to_owned());
    let reworded = Reconcile(std::slice::from_ref(&artifact), &changed);
    assert!(reworded.Absent().is_empty());
    assert_eq!(reworded.Reworded().len(), 1);
    assert!(matches!(
        reworded.Reworded().first().map(|outcome| &outcome.disposition),
        Some(Disposition::Reworded { .. })
    ));
}
