//! Every claim this suite makes against the revision archives.
//!
//! All nine corpus-gated tests are here with [`Archives`], the root that gates them. That is
//! forced rather than chosen: `tests/contract/src/gates.rs` follows calls within one file's
//! text and none of these tests names `NOMOS_SPEC_ARCHIVES` itself, so a gated test in a
//! sibling module would be counted by nobody and the declared size of the hole would drop
//! without an assertion being removed.

use crate::rows::{Entry, Family, Fates, Register, V14_LAST, V14_PREVIOUS, V15};
use crate::measure::{Is_Record, Measure};
use nomos_spec_ingest::{
    Archive, Fate, Regression, RegressionReport, Restored, Revision, Revisions_In, Tally,
};
use std::path::{Path, PathBuf};

const NEW_RECORDS: &[&str] = &[
    "records/architecture/ARC-ARCHWORKBENCH-001-bidirectional-architecture-design-generation-and-round-trip.md",
    "records/architecture/ARC-FEATOVERLAY-001-feature-overlays-and-topology.md",
];

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

fn Read(root: &Path, label: &str) -> Revision
{
    let revisions = Revisions_In(root).unwrap_or_else(|error| panic!("{error}"));
    let (_, path) = revisions
        .iter()
        .find(|(found, _)| return found == label)
        .unwrap_or_else(|| panic!("{label} is not among the revision archives"));
    let mut archive = Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));

    return Revision::Read(&mut archive, label).unwrap_or_else(|error| panic!("{error}"));
}

fn Headline(root: &Path) -> RegressionReport
{
    let before = Read(root, V14_LAST);
    let after = Read(root, V15);

    return Regression(&before, &after).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn Test_The_Headline_Should_Reproduce_From_The_Archives()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let mut checked = 0_u32;

    assert_eq!(report.from, V14_LAST);
    assert_eq!(report.to, V15);
    for entry in Register()
    {
        let (Some(label), Some(fates)) = (entry.family.as_deref(), entry.fates.as_ref())
        else
        {
            continue;
        };

        Assert_The_Archives_Agree(&report, &entry, Family(label), fates);
        checked = checked.saturating_add(1);
    }
    assert_eq!(checked, 9, "a family went unmeasured");
}

#[test]
fn Test_Every_Measured_Figure_Should_Reproduce()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let mut checked = 0_u32;
    for entry in Register()
    {
        for (key, stated) in &entry.measured
        {
            assert_eq!(
                Measure(key, &report),
                *stated,
                "{}: {key} is registered as {stated} and measures {}",
                entry.id,
                Measure(key, &report)
            );
            checked = checked.saturating_add(1);
        }
    }

    assert!(checked > 0, "the register states no figure, so this measured nothing");
}

#[test]
fn Test_The_Families_The_Plan_Calls_Gone_Should_Be_Hollowed_Rather_Than_Absent()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let mut hollowed = 0_u32;
    let heading_shaped = [
        Restored::RoadmapMilestone,
        Restored::Scenario,
        Restored::Service,
        Restored::AppendixD,
        Restored::AppendixH,
        Restored::HeadlessInventory,
        Restored::IdeProfile,
    ];

    for family in heading_shaped
    {
        let tally = report.Tally(family);

        Assert_Every_Member_Is_Hollowed(&report, family, &tally);
        hollowed = hollowed.saturating_add(tally.hollowed);
    }
    assert_eq!(hollowed, 92, "members whose heading survives and whose content does not");
}

#[test]
fn Test_The_Content_That_Really_Went_Should_Be_Named()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let gone: Vec<&str> = report
        .members
        .iter()
        .filter(|member| return member.fate == Fate::Gone)
        .map(|member| return member.name.as_str())
        .collect();
    let Some(lost) = report.Named("ModelUsageObservation")
    else
    {
        panic!("the member does not resolve by the name the corpus gives it");
    };

    assert_eq!(gone.len(), 19, "{gone:?}");
    for name in [
        "WorkspaceContext",
        "BuildVariant",
        "EvidenceClassification",
        "WorkflowStepContract",
        "TelemetryJunction",
        "ModelUsageObservation",
        "Task envelope",
        "Protocol binding",
    ]
    {
        assert!(gone.contains(&name), "{name} is not among the members v15.0 lost: {gone:?}");
    }
    assert_eq!(lost.id, "CDM-MODELUSAGEOBSERVATION");
    assert!(lost.was.starts_with("02-core"), "{}", lost.was);
    assert_eq!(lost.family, Restored::CanonicalDomainModel);
}

#[test]
fn Test_The_Records_The_Plan_Calls_New_Should_Be_Relocations()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let moved: Vec<&str> = report
        .documents
        .relocated
        .iter()
        .filter(|relocation| return Is_Record(&relocation.to))
        .map(|relocation| return relocation.to.as_str())
        .collect();
    let written: Vec<&str> = report
        .documents
        .appeared
        .iter()
        .filter(|path| return Is_Record(path))
        .map(String::as_str)
        .collect();

    assert_eq!(moved.len(), 64, "record documents v15.0 carries unchanged from v14.36");
    assert_eq!(written, NEW_RECORDS, "the records v15.0 actually wrote");
    for relocation in &report.documents.relocated
    {
        assert_eq!(
            relocation.from.len(),
            1,
            "{} could have come from any of {:?}",
            relocation.to,
            relocation.from
        );
    }
}

#[test]
fn Test_The_Filler_The_Blocklist_Does_Not_See_Should_Be_Named()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline(&root);
    let Some(widest) = report.filler.Widest_Undeclared()
    else
    {
        panic!("no undeclared template, so the blocklist saw everything");
    };

    assert!(
        widest
            .text
            .contains("This section preserves the governed reference or explanatory material for"),
        "{}",
        widest.text
    );
    assert_eq!(widest.sections, 196);
    assert_eq!(widest.documents.len(), 132, "the plan's 132, reproduced");
    assert_eq!(report.filler.declared.len(), 99, "documents the blocklist does match");
    assert!(
        report.filler.templates.iter().any(|template| return template.declared.is_some()),
        "no declared pattern matched any template, so the comparison is between one \
         mechanism and nothing"
    );
}

#[test]
fn Test_The_Summary_Should_Name_What_It_Counted()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let summary = Headline(&root).Summary();

    assert!(summary.contains(V14_LAST) && summary.contains(V15), "{summary}");
    assert!(summary.contains("hollowed"), "{summary}");
    assert!(
        summary.contains("Counterfactual Analysis Service"),
        "the summary counted without naming a member: {summary}"
    );
}

#[test]
fn Test_An_Ordinary_Pair_Should_Preserve_Every_Member()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let before = Read(&root, V14_PREVIOUS);
    let after = Read(&root, V14_LAST);
    let report = Regression(&before, &after).expect("reports");
    for family in Restored::All()
    {
        let tally = report.Tally(*family);
        assert_eq!(
            tally.preserved,
            tally.Total(),
            "{}: {tally:?} over a pair where nothing was reorganised\n{}",
            family.Label(),
            report.Summary()
        );
    }

    assert!(
        report.members.len() > 100,
        "the control measured {} members",
        report.members.len()
    );
}

#[test]
fn Test_A_Revision_Without_The_Volumes_Should_Be_Refused()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let before = Read(&root, V15);
    let after = Read(&root, V14_LAST);
    let refusal = Regression(&before, &after)
        .expect_err("v15.0 has no domain volumes and must not report every family gone");

    assert!(format!("{refusal}").contains("no family to ask after"), "{refusal}");
}

/// The register's fates for a family and the archives' tally of it are the same four numbers.
fn Assert_The_Archives_Agree(
    report: &RegressionReport,
    entry: &Entry,
    family: Restored,
    fates: &Fates,
)
{
    let tally = report.Tally(family);

    assert_eq!(
        (tally.preserved, tally.hollowed, tally.mentioned, tally.gone),
        (fates.preserved, fates.hollowed, fates.mentioned, fates.gone),
        "{}: the register says {fates:?} and the archives say {tally:?}\n{}",
        entry.id,
        report.Summary()
    );
}

/// The plan calls this family gone, so no member may be absent and every one must be hollow.
fn Assert_Every_Member_Is_Hollowed(report: &RegressionReport, family: Restored, tally: &Tally)
{
    assert_eq!(
        tally.gone, 0,
        "{}: the plan calls this family gone and {} members are absent from v15.0",
        family.Label(),
        tally.gone
    );
    assert_eq!(
        tally.hollowed,
        tally.Total(),
        "{}: not every member is hollowed\n{}",
        family.Label(),
        report.Summary()
    );
}
