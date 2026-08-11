
use nomos_spec_ingest::{
    Archive, Fate, Hollow, Regression, RegressionReport, Restored, Revision, Revisions_In,
    DOMAIN_VOLUMES,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const HEADLINE: &str = include_str!("../../../../tests/corpus/regression/headline.json");

const COUNTS: &str = include_str!("../../../../tests/corpus/families/counts.json");

const V14_LAST: &str = "v14.36";

const V15: &str = "v15.0";

const V14_PREVIOUS: &str = "v14.35";

const PLAN_HEADLINE: &[&str] = &[
    "282 table_row",
    "6 code_block",
    "8 roadmap_milestone",
    "8 scenario",
    "54 service",
    "N glossary_term",
    "all appendix-D schema",
    "all appendix-H section",
    "132 sections replaced by filler",
    "64 records (records/ + spec-governance/records/)",
];

const NEW_RECORDS: &[&str] = &[
    "records/architecture/ARC-ARCHWORKBENCH-001-bidirectional-architecture-design-generation-and-round-trip.md",
    "records/architecture/ARC-FEATOVERLAY-001-feature-overlays-and-topology.md",
];

#[derive(Debug, Deserialize)]
struct Entry
{
    id: String,
    row: String,
    clause: String,
    family: Option<String>,
    #[serde(rename = "counts_entry")]
    quoted: Option<String>,
    fates: Option<Fates>,
    measured: BTreeMap<String, u32>,
    resolution: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Fates
{
    preserved: u32,
    hollowed: u32,
    mentioned: u32,
    gone: u32,
}

impl Fates
{
    const fn Total(&self) -> u32
    {
        return self
            .preserved
            .saturating_add(self.hollowed)
            .saturating_add(self.mentioned)
            .saturating_add(self.gone);
    }
}

#[derive(Debug, Deserialize)]
struct Count
{
    id: String,
    measured: u32,
}

fn Register() -> Vec<Entry>
{
    return serde_json::from_str(HEADLINE).expect("the headline register does not parse");
}

fn Counts() -> Vec<Count>
{
    return serde_json::from_str(COUNTS).expect("the counts register does not parse");
}

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

fn Family(label: &str) -> Restored
{
    return Restored::All()
        .iter()
        .find(|family| return family.Label() == label)
        .copied()
        .unwrap_or_else(|| panic!("{label} is not a restored family"));
}

#[test]
fn Test_Every_Headline_Clause_Should_Be_Answered_Once()
{
    let entries = Register();

    for clause in PLAN_HEADLINE
    {
        let answering: Vec<&str> = entries
            .iter()
            .filter(|entry| return entry.clause == *clause)
            .map(|entry| return entry.id.as_str())
            .collect();
        assert_eq!(answering.len(), 1, "{clause} is answered by {answering:?}");
    }
}

#[test]
fn Test_Every_Restored_Family_Should_Carry_A_Fate()
{
    let entries = Register();
    let named: Vec<&str> = entries.iter().filter_map(|entry| return entry.family.as_deref()).collect();

    for family in Restored::All()
    {
        let carrying = named.iter().filter(|label| return **label == family.Label()).count();
        assert_eq!(carrying, 1, "{} carries {carrying} fate entries", family.Label());
    }
    for label in &named
    {
        Family(label);
    }
    for entry in &entries
    {
        assert_eq!(
            entry.family.is_some(),
            entry.fates.is_some(),
            "{}: a family entry states fates and only a family entry does",
            entry.id
        );
    }
}

#[test]
fn Test_A_Family_Should_Sum_To_The_Size_The_Counts_Register_Measured()
{
    let counts = Counts();

    for entry in Register()
    {
        let Some(fates) = &entry.fates
        else
        {
            continue;
        };
        let quoted = entry
            .quoted
            .as_deref()
            .unwrap_or_else(|| panic!("{}: states fates and quotes no measured size", entry.id));
        let measured = counts
            .iter()
            .find(|count| return count.id == quoted)
            .unwrap_or_else(|| panic!("{}: {quoted} is not in the counts register", entry.id))
            .measured;

        assert_eq!(
            fates.Total(),
            measured,
            "{}: the fates cover {} members and the register measured {measured}",
            entry.id,
            fates.Total()
        );
    }
}

#[test]
fn Test_Every_Entry_Should_Say_What_The_Clause_Resolved_To()
{
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for entry in &Register()
    {
        assert!(seen.insert(entry.id.as_str()), "{} is registered twice", entry.id);
        assert!(!entry.resolution.trim().is_empty(), "{}: resolves to nothing", entry.id);
        assert!(!entry.clause.trim().is_empty(), "{}: answers no clause", entry.id);
        assert!(
            ["disappeared", "changed in place", "appeared", "context"].contains(&entry.row.as_str()),
            "{}: {} is not a row of the headline",
            entry.id,
            entry.row
        );
    }
}

#[test]
fn Test_Every_Quoted_Count_Should_Exist_In_The_Counts_Register()
{
    let counts = Counts();

    for entry in Register()
    {
        let Some(quoted) = entry.quoted
        else
        {
            continue;
        };
        assert!(
            counts.iter().any(|count| return count.id == quoted),
            "{}: quotes {quoted}, which is not in the counts register",
            entry.id
        );
    }
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
    assert_eq!(report.from, V14_LAST);
    assert_eq!(report.to, V15);

    let mut checked = 0_u32;
    for entry in Register()
    {
        let (Some(label), Some(fates)) = (entry.family.as_deref(), entry.fates.as_ref())
        else
        {
            continue;
        };
        let tally = report.Tally(Family(label));

        assert_eq!(
            (tally.preserved, tally.hollowed, tally.mentioned, tally.gone),
            (fates.preserved, fates.hollowed, fates.mentioned, fates.gone),
            "{}: the register says {fates:?} and the archives say {tally:?}\n{}",
            entry.id,
            report.Summary()
        );
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

fn Measure(key: &str, report: &RegressionReport) -> u32
{
    let widest = || {
        return report
            .filler
            .Widest_Undeclared()
            .unwrap_or_else(|| panic!("v15.0 carries no undeclared template"));
    };
    let records = |paths: &[String]| {
        return Count(
            paths
                .iter()
                .filter(|path| return Is_Record(path))
                .count(),
        );
    };

    return match key
    {
        "volumes.absent" => Count(
            report
                .documents
                .disappeared
                .iter()
                .filter(|path| return path.contains(DOMAIN_VOLUMES))
                .count(),
        ),

        "documents.appeared" => Count(report.documents.appeared.len()),
        "documents.disappeared" => Count(report.documents.disappeared.len()),
        "documents.changed" => Count(report.documents.changed.len()),
        "documents.relocated" => Count(report.documents.relocated.len()),

        "records.relocated" => Count(
            report
                .documents
                .relocated
                .iter()
                .filter(|moved| return Is_Record(&moved.to))
                .count(),
        ),
        "records.new" => records(&report.documents.appeared),

        "filler.declared_documents" => Count(report.filler.declared.len()),
        "filler.stub_documents" => Count(report.filler.stubs.len()),
        "filler.widest_undeclared_sections" => widest().sections,
        "filler.widest_undeclared_documents" => Count(widest().documents.len()),

        "members.gone" => Count(report.members.iter().filter(|member| return member.fate == Fate::Gone).count()),
        "members.hollowed_by_a_declared_pattern" => Count(
            report
                .members
                .iter()
                .filter(|member| return Hollowed_By_A_Declared_Pattern(&member.fate))
                .count(),
        ),
        "members.hollowed_by_an_undeclared_template" => Count(
            report
                .members
                .iter()
                .filter(|member| return Hollowed_By_An_Undeclared_Template(&member.fate))
                .count(),
        ),

        other => panic!("{other} is in the register and nothing measures it"),
    };
}

fn Hollowed_By_A_Declared_Pattern(fate: &Fate) -> bool
{
    return matches!(
        fate,
        Fate::Hollowed {
            evidence: Hollow::Template {
                declared: Some(_), ..
            },
            ..
        }
    );
}

fn Hollowed_By_An_Undeclared_Template(fate: &Fate) -> bool
{
    return matches!(
        fate,
        Fate::Hollowed {
            evidence: Hollow::Template { declared: None, .. },
            ..
        }
    );
}

fn Is_Record(path: &str) -> bool
{
    return path.starts_with("records/") || path.starts_with("spec-governance/records/");
}

fn Count(value: usize) -> u32
{
    return u32::try_from(value).unwrap_or(u32::MAX);
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
    let heading_shaped = [
        Restored::RoadmapMilestone,
        Restored::Scenario,
        Restored::Service,
        Restored::AppendixD,
        Restored::AppendixH,
        Restored::HeadlessInventory,
        Restored::IdeProfile,
    ];

    let mut hollowed = 0_u32;
    for family in heading_shaped
    {
        let tally = report.Tally(family);
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

    let Some(lost) = report.Named("ModelUsageObservation")
    else
    {
        panic!("the member does not resolve by the name the corpus gives it");
    };
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
    assert_eq!(moved.len(), 64, "record documents v15.0 carries unchanged from v14.36");

    let written: Vec<&str> = report
        .documents
        .appeared
        .iter()
        .filter(|path| return Is_Record(path))
        .map(String::as_str)
        .collect();
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
