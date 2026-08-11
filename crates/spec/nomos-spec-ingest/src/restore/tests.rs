//! What this module promises, exercised.

use super::*;
use crate::phases::Ingest_Source_Document;
use super::extract::{Milestone, Numbering};

const CORE: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                    | Model | Responsibility |\n| --- | --- |\n\
                    | WorkspaceContext | Repository, branch, configuration. |\n\
                    | MetricTradeoffProjection | Cost against benefit. |\n\n\
                    ## 6. Systems and subsystem responsibilities\n\n\
                    ### 6.1 Change reasoning\n\n\
                    #### Counterfactual Analysis Service\n\nEvaluates proposals.\n\n\
                    #### CodeStewardshipAssignment\n\nA model, not a service.\n";

const REFERENCE: &str = "# Reference\n\n## Glossary\n\n\
                         | Term | Definition |\n| --- | --- |\n\
                         | Applicability | Whether a rule can run. |\n\n\
                         ### Extended operational terms\n\n#### SavedView\n\nA saved query.\n\n\
                         ## Scenarios\n\n\
                         ### G.1 Scenario catalog and coverage\n\nA catalog.\n\n\
                         ### G.2 End-to-end scenario: add a strategy\n\nA scenario.\n";

/// A store and the markdown it was built from.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct Built
{
    store: SpecificationStore,
    documents: BTreeMap<String, String>,
}

fn Corpus(documents: &[(&str, &str)]) -> Built
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut set = BTreeMap::new();

    for (document, markdown) in documents
    {
        Ingest_Source_Document(&mut store, document, "v14.36", markdown).expect("ingests");
        set.insert((*document).to_owned(), (*markdown).to_owned());
    }

    return Built {
        store,
        documents: set,
    };
}

fn Core() -> Built
{
    return Corpus(&[("02-core.md", CORE)]);
}

#[test]
fn Test_A_Domain_Model_Row_Should_Become_A_Concept_Named_After_Itself()
{
    let members = Extract("02-core.md", CORE).expect("extracts");
    let models: Vec<&Member> = members
        .iter()
        .filter(|member| return member.family == Restored::CanonicalDomainModel)
        .collect();

    assert_eq!(models.len(), 2, "the header or the delimiter was minted as a model");
    assert_eq!(models.first().map(|member| member.id.as_str()), Some("CDM-WORKSPACECONTEXT"));
    assert_eq!(models.first().map(|member| member.name.as_str()), Some("WorkspaceContext"));
    assert!(matches!(models.first().map(|member| member.origin), Some(Origin::Row { .. })));
}

/// The reason the header kind exists. Without it the column titles mint a concept
/// named after the column names.
#[test]
fn Test_The_Column_Titles_Should_Not_Become_A_Concept()
{
    let members = Extract("02-core.md", CORE).expect("extracts");

    assert!(
        !members.iter().any(|member| return member.name == "Model"),
        "the header row was restored as a domain model"
    );
}

#[test]
fn Test_Only_A_Leaf_Naming_A_Service_Should_Become_One()
{
    let members = Extract("02-core.md", CORE).expect("extracts");
    let services: Vec<&str> = members
        .iter()
        .filter(|member| return member.family == Restored::Service)
        .map(|member| return member.name.as_str())
        .collect();

    assert_eq!(services, vec!["Counterfactual Analysis Service"]);
}

/// A section that catalogs the scenarios is not a scenario.
#[test]
fn Test_A_Catalog_Section_Should_Not_Become_A_Scenario()
{
    let members = Extract("09-reference.md", REFERENCE).expect("extracts");
    let scenarios: Vec<&str> = members
        .iter()
        .filter(|member| return member.family == Restored::Scenario)
        .map(|member| return member.id.as_str())
        .collect();

    assert_eq!(scenarios, vec!["SCEN-G-2"]);
}

#[test]
fn Test_The_Glossary_Should_Restore_Both_Of_Its_Shapes()
{
    let members = Extract("09-reference.md", REFERENCE).expect("extracts");
    let terms: Vec<&str> = members
        .iter()
        .filter(|member| return member.family == Restored::GlossaryTerm)
        .map(|member| return member.id.as_str())
        .collect();

    assert_eq!(terms, vec!["GLS-APPLICABILITY", "GLS-SAVEDVIEW"]);
}

/// A family recognised in the wrong volume would count whatever happened to be shaped
/// like it.
#[test]
fn Test_A_Family_Should_Not_Be_Recognised_Outside_Its_Volume()
{
    assert!(Extract("07-clients.md", CORE).expect("extracts").is_empty());
}

#[test]
fn Test_Two_Members_Taking_One_Identifier_Should_Be_Refused()
{
    let doubled = "# X\n\n## Glossary\n\n| Term | Definition |\n| --- | --- |\n\
                   | Applicability | One. |\n| applicability | Two. |\n";

    let refusal = Extract("09-reference.md", doubled).expect_err("must refuse");

    assert!(format!("{refusal}").contains("GLS-APPLICABILITY"), "{refusal}");
}

#[test]
fn Test_Milestones_Should_Number_Themselves_As_The_Roadmap_Does()
{
    assert_eq!(Milestone("Foundation 0 — Protocol"), Some("F.0".to_owned()));
    assert_eq!(Milestone("Release 7 — Advanced"), Some("R.7".to_owned()));
    assert_eq!(Milestone("Release notes"), None);
    assert_eq!(Milestone("11.1 First usable product boundary"), None);
}

#[test]
fn Test_Numbering_Should_Require_Exactly_Its_Depth()
{
    assert_eq!(Numbering("D.7 Profiles", 'D', 1), Some("D.7".to_owned()));
    assert_eq!(Numbering("D.7 Profiles", 'D', 2), None);
    assert_eq!(Numbering("D.7.1 atlas", 'D', 2), Some("D.7.1".to_owned()));
    assert_eq!(Numbering("Design notes", 'D', 1), None);
}

#[test]
fn Test_A_Restored_Concept_Should_Trace_To_Its_Own_Row()
{
    let Built {
        mut store,
        documents,
    } = Core();

    let report = Restore(&mut store, "v14.36", &documents).expect("restores");

    assert!(report.contested_aliases.is_empty(), "{:?}", report.contested_aliases);
    let traced: String = store
        .Connection()
        .query_row(
            "SELECT r.text FROM lineage l
             JOIN source_table_rows r ON r.uid = l.source_table_row_uid
             JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = 'CDM-METRICTRADEOFFPROJECTION'",
            [],
            |row| row.get(0),
        )
        .expect("the concept traces to no row");

    assert!(traced.contains("MetricTradeoffProjection"), "traced to {traced}");
    assert!(!traced.contains("WorkspaceContext"), "traced to the whole table");
}

#[test]
fn Test_A_Restored_Concept_Should_Resolve_By_Its_Authored_Name()
{
    let Built {
        mut store,
        documents,
    } = Core();
    Restore(&mut store, "v14.36", &documents).expect("restores");

    assert!(Resolve(&store, "CDM-WORKSPACECONTEXT").expect("resolves").is_some());
    assert!(
        Resolve(&store, "WorkspaceContext").expect("resolves").is_some(),
        "the name the corpus uses resolves to nothing"
    );
    assert!(Resolve(&store, "NoSuchModel").expect("resolves").is_none());
}

/// A name two members carry belongs to neither. Handing it to whichever volume was
/// read first makes resolution depend on directory order and answer confidently with
/// one of two right answers.
#[test]
fn Test_A_Name_Two_Members_Carry_Should_Resolve_To_Neither()
{
    const SHARED: &str = "# Reference\n\n## Glossary\n\n\
                          | Term | Definition |\n| --- | --- |\n\
                          | WorkspaceContext | The term, not the model. |\n";

    let Built {
        mut store,
        documents,
    } = Corpus(&[("02-core.md", CORE), ("09-reference.md", SHARED)]);

    let report = Restore(&mut store, "v14.36", &documents).expect("restores");

    assert_eq!(report.ambiguous_names, vec!["WorkspaceContext".to_owned()]);
    assert!(
        Resolve(&store, "WorkspaceContext").expect("resolves").is_none(),
        "an ambiguous name answered anyway"
    );
    assert!(
        Resolve(&store, "CDM-WORKSPACECONTEXT").expect("resolves").is_some(),
        "both nodes must still resolve by identifier"
    );
    assert!(Resolve(&store, "GLS-WORKSPACECONTEXT").expect("resolves").is_some());
}

#[test]
fn Test_Restoring_Twice_Should_Change_Nothing()
{
    let Built {
        mut store,
        documents,
    } = Core();

    let first = Restore(&mut store, "v14.36", &documents).expect("restores");
    let nodes = store.Count(nomos_spec_store::Table::Nodes).expect("counts");
    let lineage = store.Count(nomos_spec_store::Table::Lineage).expect("counts");
    let second = Restore(&mut store, "v14.36", &documents).expect("restores again");

    assert_eq!(first.members, second.members);
    assert_eq!(store.Count(nomos_spec_store::Table::Nodes).expect("counts"), nodes);
    assert_eq!(store.Count(nomos_spec_store::Table::Lineage).expect("counts"), lineage);
    assert!(second.contested_aliases.is_empty(), "re-running contested its own aliases");
}

/// Restoring from text the store never saw would make the source-truth gate optional
/// for exactly the content the restoration cares about.
#[test]
fn Test_Restoring_A_Document_The_Store_Does_Not_Hold_Should_Be_Refused()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut documents = BTreeMap::new();
    documents.insert("02-core.md".to_owned(), CORE.to_owned());

    let refusal = Restore(&mut store, "v14.36", &documents).expect_err("must refuse");

    assert!(format!("{refusal}").contains("not in the store"), "{refusal}");
}

/// A restoration may not repoint a name another authority already owns.
#[test]
fn Test_A_Contested_Alias_Should_Be_Reported_Rather_Than_Silently_Repointed()
{
    let Built {
        mut store,
        documents,
    } = Core();
    let other = store
        .Upsert_Node("OTHER-001", "concept", "canonical", "record", "Something else")
        .expect("mints");
    store
        .Connection()
        .execute(
            "INSERT INTO node_aliases (alias, node_uid) VALUES ('WorkspaceContext', ?1)",
            rusqlite::params![other],
        )
        .expect("takes the alias");

    let report = Restore(&mut store, "v14.36", &documents).expect("restores");

    assert_eq!(report.contested_aliases, vec!["WorkspaceContext".to_owned()]);
    assert_eq!(
        Resolve(&store, "WorkspaceContext").expect("resolves"),
        Some(other),
        "the contested alias was silently repointed"
    );
}

#[test]
fn Test_The_Summary_Should_Name_Members_Rather_Than_Only_Count_Them()
{
    let Built {
        mut store,
        documents,
    } = Core();
    let report = Restore(&mut store, "v14.36", &documents).expect("restores");

    assert!(report.Summary().contains("WorkspaceContext"), "{}", report.Summary());
}
