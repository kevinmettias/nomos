//! What this module promises, exercised.

use super::*;

const CORE: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                    | Model | Responsibility |\n| --- | --- |\n\
                    | WorkspaceContext | Repository, branch, configuration. |\n\
                    | ModelUsageObservation and CostObservation | Tokens against money. |\n\n\
                    ## 6. Systems and subsystem responsibilities\n\n\
                    ### 6.1 Change reasoning\n\n\
                    #### Counterfactual Analysis Service\n\nEvaluates proposals.\n";

const DECLARED: &str = "This section groups related specification material for the domain.";

fn Documents(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
{
    return pairs
        .iter()
        .map(|(path, text)| return ((*path).to_owned(), (*text).to_owned()))
        .collect();
}

fn Earlier() -> Revision
{
    return Revision {
        label: "v14.36".to_owned(),
        documents: Documents(&[("01_authoring/domain_volumes/02-core-architecture.md", CORE)]),
    };
}

fn Earlier_With(path: &str, text: &str) -> Revision
{
    let mut revision = Earlier();
    revision.documents.insert(path.to_owned(), text.to_owned());

    return revision;
}

fn Later_Than(documents: &[(&str, &str)]) -> Revision
{
    return Revision {
        label: "v15.0".to_owned(),
        documents: Documents(documents),
    };
}

fn Fate_Of(report: &RegressionReport, name: &str) -> Fate
{
    return report
        .Named(name)
        // Callers name a member the `Earlier()` fixture puts in the corpus, so the lookup
        // failing means extraction stopped recognising it — a different failure from the
        // fate being wrong, and one the `assert_eq!` on a fate could not tell apart.
        .unwrap_or_else(|| panic!("{name} is not a member"))
        .fate
        .clone();
}

fn Reported(later: &[(&str, &str)]) -> RegressionReport
{
    return Regression(&Earlier(), &Later_Than(later)).expect("reports");
}

#[test]
fn Test_A_Heading_Over_A_Repeated_Paragraph_Should_Be_Hollowed()
{
    let report = Reported(&[
        ("a.md", "# Counterfactual Analysis Service\n\nRefer to the owning domain.\n"),
        ("b.md", "# Something else\n\nRefer to the owning domain.\n"),
        ("c.md", "# A third\n\nRefer to the owning domain.\n"),
    ]);

    assert_eq!(
        Fate_Of(&report, "Counterfactual Analysis Service"),
        Fate::Hollowed {
            document: "a.md".to_owned(),
            evidence: Hollow::Template {
                shared_with: 3,
                declared: None,
            },
        }
    );
}

#[test]
fn Test_A_Paragraph_Two_Sections_Share_Should_Not_Be_A_Template()
{
    let report = Reported(&[
        ("a.md", "# Counterfactual Analysis Service\n\nRefer to the owning domain.\n"),
        ("b.md", "# Something else\n\nRefer to the owning domain.\n"),
    ]);

    assert_eq!(
        Fate_Of(&report, "Counterfactual Analysis Service"),
        Fate::Preserved {
            document: "a.md".to_owned(),
        }
    );
}

#[test]
fn Test_A_Heading_Over_Nothing_Should_Be_Hollowed_With_No_Body()
{
    let report = Reported(&[("a.md", "# Counterfactual Analysis Service\n")]);

    assert_eq!(
        Fate_Of(&report, "Counterfactual Analysis Service"),
        Fate::Hollowed {
            document: "a.md".to_owned(),
            evidence: Hollow::NoBody,
        }
    );
}

#[test]
fn Test_A_Heading_Over_Its_Own_Link_List_Should_Not_Count_As_A_Body()
{
    let report = Reported(&[(
        "a.md",
        "# Counterfactual Analysis Service\n\n- [One](one.md)\n- [Two](two.md)\n",
    )]);

    assert_eq!(
        Fate_Of(&report, "Counterfactual Analysis Service"),
        Fate::Hollowed {
            document: "a.md".to_owned(),
            evidence: Hollow::NoBody,
        }
    );
}

#[test]
fn Test_A_Name_In_Prose_Should_Be_Mentioned_Rather_Than_Preserved()
{
    let report = Reported(&[("a.md", "# Elsewhere\n\nThe WorkspaceContext is discussed.\n")]);

    assert_eq!(
        Fate_Of(&report, "WorkspaceContext"),
        Fate::Mentioned {
            documents: vec!["a.md".to_owned()],
        }
    );
}

#[test]
fn Test_A_Name_Occurring_Nowhere_Should_Be_Gone()
{
    let report = Reported(&[("a.md", "# Elsewhere\n\nNothing of the kind.\n")]);

    assert_eq!(Fate_Of(&report, "WorkspaceContext"), Fate::Gone);
    assert_eq!(report.Tally(Restored::CanonicalDomainModel).gone, 3);
}

#[test]
fn Test_A_Model_Sharing_A_Row_Should_Be_Found_In_That_Row()
{
    let report = Reported(&[(
        "a.md",
        "# Models\n\n| Model | Responsibility |\n| --- | --- |\n\
         | ModelUsageObservation and CostObservation | Tokens against money. |\n",
    )]);

    assert_eq!(
        Fate_Of(&report, "CostObservation"),
        Fate::Preserved {
            document: "a.md".to_owned(),
        },
        "a model reads as absent from a table it is in, because the cell names two"
    );
}

#[test]
fn Test_A_Moved_Document_Should_Be_Relocated_Rather_Than_Both_Sets()
{
    let earlier = Earlier_With("old/record.md", "# Record\n\nA decision.\n");
    let later = Later_Than(&[("new/record.md", "# Record\n\nA decision.\n")]);

    let report = Regression(&earlier, &later).expect("reports");

    assert_eq!(report.documents.relocated.len(), 1);
    assert_eq!(
        report.documents.relocated.first().map(|moved| return moved.to.clone()),
        Some("new/record.md".to_owned())
    );
    assert_eq!(
        report.documents.relocated.first().map(|moved| return moved.from.clone()),
        Some(vec!["old/record.md".to_owned()])
    );
    assert!(report.documents.appeared.is_empty());
    assert!(!report.documents.disappeared.iter().any(|path| return path == "old/record.md"));
}

#[test]
fn Test_A_Relocation_With_Two_Origins_Should_Name_Both()
{
    let mut earlier = Earlier_With("old/one.md", "# Record\n\nA decision.\n");
    earlier
        .documents
        .insert("old/two.md".to_owned(), "# Record\n\nA decision.\n".to_owned());
    let later = Later_Than(&[("new/record.md", "# Record\n\nA decision.\n")]);

    let report = Regression(&earlier, &later).expect("reports");

    assert_eq!(
        report.documents.relocated.first().map(|moved| return moved.from.clone()),
        Some(vec!["old/one.md".to_owned(), "old/two.md".to_owned()])
    );
}

#[test]
fn Test_An_Edited_Move_Should_Not_Be_A_Relocation()
{
    let earlier = Earlier_With("old/record.md", "# Record\n\nA decision.\n");
    let later = Later_Than(&[("new/record.md", "# Record\n\nA different decision.\n")]);

    let report = Regression(&earlier, &later).expect("reports");

    assert!(report.documents.relocated.is_empty());
    assert_eq!(report.documents.appeared, vec!["new/record.md".to_owned()]);
    assert!(report.documents.disappeared.contains(&"old/record.md".to_owned()));
}

#[test]
fn Test_A_Revision_Without_The_Volumes_Should_Be_Refused()
{
    let earlier = Revision {
        label: "v15.0".to_owned(),
        documents: Documents(&[("records/one.md", "# Record\n\nA decision.\n")]),
    };

    let refusal = Regression(&earlier, &Later_Than(&[("a.md", "# A\n\nText.\n")]))
        .expect_err("must refuse");

    assert!(format!("{refusal}").contains("no family to ask after"), "{refusal}");
}

#[test]
fn Test_A_Revision_With_No_Markdown_Should_Be_Refused()
{
    let empty: BTreeMap<String, String> = BTreeMap::new();

    assert!(Fingerprint_Of("v15.0", &empty).is_err());
}

#[test]
fn Test_Declared_Filler_Should_Be_Named_As_Declared()
{
    let first = format!("# Counterfactual Analysis Service\n\n{DECLARED}\n");
    let second = format!("# Something else\n\n{DECLARED}\n");
    let third = format!("# A third\n\n{DECLARED}\n");
    let report = Reported(&[("a.md", &first), ("b.md", &second), ("c.md", &third)]);

    assert!(
        matches!(
            Fate_Of(&report, "Counterfactual Analysis Service"),
            Fate::Hollowed {
                evidence: Hollow::Template {
                    declared: Some(_), ..
                },
                ..
            }
        ),
        "{:?}",
        Fate_Of(&report, "Counterfactual Analysis Service")
    );
    assert_eq!(report.filler.declared.len(), 3);
    assert!(report.filler.Widest_Undeclared().is_none());
    assert_eq!(report.filler.stubs.len(), 3);
}

#[test]
fn Test_Declared_Filler_Should_Not_Need_The_Threshold()
{
    let only = format!("# Counterfactual Analysis Service\n\n{DECLARED}\n");
    let report = Reported(&[("a.md", &only)]);

    assert!(matches!(
        Fate_Of(&report, "Counterfactual Analysis Service"),
        Fate::Hollowed { .. }
    ));
}

#[test]
fn Test_A_Template_Naming_Its_Own_Section_Should_Read_As_One_Template()
{
    let report = Reported(&[
        (
            "a.md",
            "# Counterfactual Analysis Service\n\nRead Counterfactual Analysis Service \
             within the owning contract.\n",
        ),
        ("b.md", "# Second\n\nRead Second within the owning contract.\n"),
        ("c.md", "# Third\n\nRead Third within the owning contract.\n"),
    ]);

    assert!(
        matches!(Fate_Of(&report, "Counterfactual Analysis Service"), Fate::Hollowed { .. }),
        "three sections of one form letter read as three distinct paragraphs"
    );
    assert_eq!(
        report.filler.Widest_Undeclared().map(|template| return template.sections),
        Some(3)
    );
}

#[test]
fn Test_The_Summary_Should_Name_Members_Rather_Than_Only_Count_Them()
{
    let report = Reported(&[("a.md", "# A\n\nText.\n")]);

    assert!(report.Summary().contains("WorkspaceContext"), "{}", report.Summary());
}
