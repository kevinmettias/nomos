//! Every claim this suite makes against the revision archives.
//!
//! All nine corpus-gated tests are here with [`Archives`], the root that gates them. That is
//! forced rather than chosen: `tests/contract/src/gates.rs` follows calls within one file's
//! text and none of these tests names `NOMOS_SPEC_ARCHIVES` itself, so a gated test in a
//! sibling module would be counted by nobody and the declared size of the hole would drop
//! without an assertion being removed.

use crate::rows::{Entry, Fates, Register, Restored_Family_For_Label, V14_LAST, V14_PREVIOUS, V15};
use crate::measure::{Figure_For_Register_Key, Is_Record};
use nomos_spec_ingest::{
    Archive, Fate, Regression_Between_Revisions, RegressionReport, Restored, Revision, Revisions_In, Tally,
};
use std::path::{Path, PathBuf};

const NEW_RECORDS: &[&str] = &[
    "records/architecture/ARC-ARCHWORKBENCH-001-bidirectional-architecture-design-generation-and-round-trip.md",
    "records/architecture/ARC-FEATOVERLAY-001-feature-overlays-and-topology.md",
];

/// The families the register states a fate for, each one measured by the headline pair.
const REGISTERED_FAMILIES: u32 = 9;

/// Members whose heading survives into v15.0 and whose content does not.
const HOLLOWED_MEMBERS: u32 = 92;

/// The members the headline pair really lost, over every family.
const GONE_MEMBERS: usize = 19;

/// Record documents v15.0 carries unchanged from v14.36, at a new path.
const MOVED_RECORDS: usize = 64;

/// The widest undeclared template the filler census found: its sections, and the documents
/// carrying it.
const WIDEST_SECTIONS: u32 = 196;
const WIDEST_DOCUMENTS: usize = 132;

/// Documents the declared blocklist does match, which is the other half of the comparison.
const DECLARED_FILLERS: usize = 99;

/// A floor rather than a measurement: an ordinary pair keeps more members than this, so a
/// walk that saw almost nothing fails here.
const ORDINARY_PAIR_FLOOR: usize = 100;

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

fn Revision_From_Label(root: &Path, label: &str) -> Revision
{
    // NOMOS_SPEC_ARCHIVES was asserted to be a directory, so a root that yields no revisions
    // at all is the wrong directory rather than an empty one. Every test below asks for two
    // labels that would then both be reported absent, which reads as a corpus that lost them.
    let revisions = Revisions_In(root).unwrap_or_else(|error| panic!("{error}"));
    let (_, path) = revisions
        .iter()
        .find(|(found, _)| return found == label)
        // The label is one of the three revisions this suite pins by constant. One of them
        // missing means the pair under test is not the pair the register was measured over,
        // so every fate it goes on to compare would be a comparison of something else.
        .unwrap_or_else(|| panic!("{label} is not among the revision archives"));
    // The listing named this path, so a zip that will not open is a damaged archive rather
    // than a revision this suite does not have.
    let mut archive = Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));

    // `Revision::Read` refuses an archive carrying no markdown, and that refusal exists
    // precisely so an unread revision cannot be reported as one where every family is gone.
    // Recovering from it here would put back the reading it refuses to make.
    return Revision::Read(&mut archive, label).unwrap_or_else(|error| panic!("{error}"));
}

fn Headline_Report(root: &Path) -> RegressionReport
{
    let before = Revision_From_Label(root, V14_LAST);
    let after = Revision_From_Label(root, V15);

    // `Regression_Between_Revisions` refuses a pair whose earlier revision carries no domain volumes, and
    // `Test_A_Revision_Without_The_Volumes_Should_Be_Refused` asserts that refusal on
    // purpose. Reaching it from the headline pair means v14.36 is not the v14.36 the
    // register was measured over, so every figure taken from this report would be wrong.
    return Regression_Between_Revisions(&before, &after).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn Test_The_Headline_Should_Reproduce_From_The_Archives()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline_Report(&root);
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

        Assert_The_Archives_Agree(&report, &entry, Restored_Family_For_Label(label), fates);
        checked = checked.saturating_add(1);
    }
    assert_eq!(checked, REGISTERED_FAMILIES, "a family went unmeasured");
}

#[test]
fn Test_Every_Measured_Figure_Should_Reproduce()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline_Report(&root);
    let mut checked = 0_u32;
    for entry in Register()
    {
        for (key, stated) in &entry.measured
        {
            assert_eq!(
                Figure_For_Register_Key(key, &report),
                *stated,
                "{}: {key} is registered as {stated} and measures {}",
                entry.id,
                Figure_For_Register_Key(key, &report)
            );
            checked = checked.saturating_add(1);
        }
    }

    assert!(checked > 0, "the register states no figure, so this measured nothing");
}

/// Every family the plan calls entirely gone: restored as headings whose content did not
/// survive. Named for what the family looks like once hollowed, not `Cases()`.
fn Heading_Shaped_Families() -> Vec<Restored>
{
    return vec![
        Restored::RoadmapMilestone,
        Restored::Scenario,
        Restored::Service,
        Restored::AppendixD,
        Restored::AppendixH,
        Restored::HeadlessInventory,
        Restored::IdeProfile,
    ];
}

#[test]
fn Test_The_Families_The_Plan_Calls_Gone_Should_Be_Hollowed_Rather_Than_Absent()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline_Report(&root);
    let mut hollowed = 0_u32;

    for family in Heading_Shaped_Families()
    {
        let tally = report.Tally(family);

        Assert_Every_Member_Is_Hollowed(&report, family, &tally);
        hollowed = hollowed.saturating_add(tally.hollowed);
    }
    assert_eq!(hollowed, HOLLOWED_MEMBERS, "members whose heading survives and whose content does not");
}

/// The named members v15.0's headline pair actually lost, quoted rather than counted.
/// Named for what they are, not `Cases()`.
fn Members_The_Headline_Lost() -> Vec<&'static str>
{
    return vec![
        "WorkspaceContext",
        "BuildVariant",
        "EvidenceClassification",
        "WorkflowStepContract",
        "TelemetryJunction",
        "ModelUsageObservation",
        "Task envelope",
        "Protocol binding",
    ];
}

#[test]
fn Test_The_Content_That_Really_Went_Should_Be_Named()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let report = Headline_Report(&root);
    let gone: Vec<&str> = report
        .members
        .iter()
        .filter(|member| return member.fate == Fate::Gone)
        .map(|member| return member.name.as_str())
        .collect();
    let Some(lost) = report.Named("ModelUsageObservation")
    else
    {
        // This test's claim is that the lost content is *named*, and the three assertions
        // below are about this member's id, its former document and its family. If it does
        // not resolve by the spelling the corpus uses, there is nothing left to assert.
        panic!("the member does not resolve by the name the corpus gives it");
    };

    assert_eq!(gone.len(), GONE_MEMBERS, "{gone:?}");
    for name in Members_The_Headline_Lost()
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
    let report = Headline_Report(&root);
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

    assert_eq!(moved.len(), MOVED_RECORDS, "record documents v15.0 carries unchanged from v14.36");
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
    let report = Headline_Report(&root);
    let Some(widest) = report.filler.Widest_Undeclared()
    else
    {
        // The test exists to name filler the blocklist does not match. No undeclared template
        // means its subject does not exist, and the four assertions below would then hold
        // over nothing — a pass reporting that the blocklist is complete when it was never
        // measured against anything it missed.
        panic!("no undeclared template, so the blocklist saw everything");
    };

    assert!(
        widest
            .text
            .contains("This section preserves the governed reference or explanatory material for"),
        "{}",
        widest.text
    );
    assert_eq!(widest.sections, WIDEST_SECTIONS);
    assert_eq!(widest.documents.len(), WIDEST_DOCUMENTS, "the plan's 132, reproduced");
    assert_eq!(report.filler.declared.len(), DECLARED_FILLERS, "documents the blocklist does match");
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
    let summary = Headline_Report(&root).Summary();

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
    let before = Revision_From_Label(&root, V14_PREVIOUS);
    let after = Revision_From_Label(&root, V14_LAST);
    let report = Regression_Between_Revisions(&before, &after)
        .expect("an ordinary pair carries both revisions' volumes, so it reports");
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
        report.members.len() > ORDINARY_PAIR_FLOOR,
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
    let before = Revision_From_Label(&root, V15);
    let after = Revision_From_Label(&root, V14_LAST);
    let refusal = Regression_Between_Revisions(&before, &after)
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
