use nomos_spec_bundle::{Bundle, Export, Import};
use nomos_spec_model::Segment;
use nomos_spec_project::{
    Build, Catalogue, Check, Content, Format, Profile, Select, Stamp, DO_NOT_EDIT, SIDECAR_SUFFIX,
};
use nomos_spec_store::SpecificationStore;
use std::collections::BTreeSet;

const CORE: &str = "# Core architecture\n\nIdentity is not a path.\n\n\
                    ## Domain model\n\n| Model | Owns |\n| --- | --- |\n\
                    | WorkspaceContext | the workspace |\n| BuildVariant | one build |\n\n\
                    ```rust\nlet quoted = \"a \\\"nested\\\" string\";\n```\n";

const CONFORMANCE: &str = "# Conformance\n\nUnknown is not pass \u{2014} ni\u{00f1}o, \
                           \u{4e2d}\u{6587}, \u{1f600}.\n";

fn Populated() -> SpecificationStore
{
    return Populated_In_Order(false);
}

fn Populated_In_Order(reversed: bool) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut documents = vec![
        ("volumes/02-core.md", CORE),
        ("volumes/03-conformance.md", CONFORMANCE),
    ];
    if reversed
    {
        documents.reverse();
    }

    for (path, text) in documents
    {
        let document = store
            .Put_Source_Document(path, "v14.36", text)
            .expect("stores the document");
        store
            .Put_Source_Blocks(document, &Segment(text))
            .expect("stores the blocks");
    }

    Populate_Graph(&store, reversed);

    return store;
}

fn Populate_Graph(store: &SpecificationStore, reversed: bool)
{
    let nodes = "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'CDM-WORKSPACECONTEXT', 'concept', 'canonical', 'record',
                        'WorkspaceContext', NULL, uid FROM suites WHERE suite_id = 'nomos';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'AGT-EXEC-001', 'requirement', 'canonical', 'record',
                        'Agent execution ancestry', NULL, uid FROM suites WHERE suite_id = 'nomos';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'D-129', 'decision', 'canonical', 'record',
                        'The store is the identity substrate', NULL, uid
                 FROM suites WHERE suite_id = 'nomos';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'SVC-COUNTERFACTUAL', 'service', 'canonical', 'record',
                        'Counterfactual Analysis Service', NULL, uid
                 FROM suites WHERE suite_id = 'nomos';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'RMAP-R-1', 'release', 'canonical', 'record',
                        'Release 1 \u{2014} deterministic check platform', NULL, uid
                 FROM suites WHERE suite_id = 'nomos';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 SELECT 'SCEN-G-2', 'scenario', 'canonical', 'record',
                        'End-to-end scenario: add a strategy', NULL, uid
                 FROM suites WHERE suite_id = 'xvpe-seed';
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                        suite_uid)
                 VALUES ('REQ-RETIRED-009', 'requirement', 'superseded', 'record', 'Retired',
                         '2026-01-14T00:00:00Z', NULL);";

    let mut statements: Vec<&str> = nodes.split(";\n").collect();
    statements.reverse();
    let reversed_nodes = statements.join(";\n");

    store
        .Connection()
        .execute_batch(
            "INSERT INTO suites (suite_id, title, authority_root)
             VALUES ('nomos', 'The Nomos specification', 1),
                    ('xvpe-seed', 'XVPE specification seed', 0);

             INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 1, 1, 'Core architecture' FROM source_documents
             WHERE path = 'volumes/02-core.md';
             INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 2, 2, 'Domain model' FROM source_documents
             WHERE path = 'volumes/02-core.md';
             INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 1, 1, 'Conformance' FROM source_documents
             WHERE path = 'volumes/03-conformance.md';",
        )
        .expect("populates the suites and headings");

    store
        .Connection()
        .execute_batch(if reversed { &reversed_nodes } else { nodes })
        .expect("populates the nodes");

    Populate_Graph_Edges(store);
}

fn Populate_Graph_Edges(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(
            "INSERT INTO node_aliases (alias, node_uid)
             SELECT 'AGT-010', uid FROM nodes WHERE node_id = 'AGT-EXEC-001';

             INSERT INTO relation_types (name, tier, inverse_of)
             VALUES ('verifies', 'core', 'verified_by'), ('verified_by', 'core', 'verifies'),
                    ('affects', 'extended', NULL);

             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'CDM-WORKSPACECONTEXT' AND t.node_id = 'AGT-EXEC-001';
             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'affects', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'D-129' AND t.node_id = 'SVC-COUNTERFACTUAL';

             INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             SELECT uid, 'AGT-EXEC-001', 'requirement', 'Nomos shall record ancestry.',
                    'sha256:aa', 'sha256:99' FROM nodes WHERE node_id = 'AGT-EXEC-001';
             INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             SELECT uid, 'D-129-01', 'principle', 'The store is the identity substrate.',
                    'sha256:bb', NULL FROM nodes WHERE node_id = 'D-129';

             INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             SELECT b.uid, NULL, 'preserved-verbatim', NULL, s.uid
             FROM source_blocks b, normative_statements s, source_documents d
             WHERE d.path = 'volumes/02-core.md' AND b.document_uid = d.uid AND b.ordinal = 2
               AND s.statement_id = 'AGT-EXEC-001';
             INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             SELECT NULL, h.uid, 'preserved-normalized', n.uid, NULL
             FROM source_headings h, nodes n, source_documents d
             WHERE d.path = 'volumes/02-core.md' AND h.document_uid = d.uid AND h.ordinal = 1
               AND n.node_id = 'CDM-WORKSPACECONTEXT';
             INSERT INTO lineage (source_table_row_uid, disposition, target_node_uid)
             SELECT r.uid, 'preserved-verbatim', n.uid
             FROM source_table_rows r, nodes n
             WHERE r.cells_json LIKE '%WorkspaceContext%'
               AND n.node_id = 'CDM-WORKSPACECONTEXT';

             INSERT INTO omissions
             (source_block_uid, source_heading_uid, reason, justification, decision_record)
             SELECT b.uid, NULL, 'superseded', 'replaced by the v15 records', 'D-129'
             FROM source_blocks b, source_documents d
             WHERE d.path = 'volumes/03-conformance.md' AND b.document_uid = d.uid
               AND b.ordinal = 1;",
        )
        .expect("populates the graph");
}

fn Shipped() -> Catalogue
{
    return Catalogue::Shipped().expect("the shipped profiles parse");
}

fn Profile_Named(id: &str) -> Profile
{
    return Shipped()
        .Named(id)
        .unwrap_or_else(|| panic!("{id} is not a shipped profile"))
        .clone();
}

fn Rendered(store: &SpecificationStore, id: &str) -> String
{
    return Build(store, &Profile_Named(id))
        .unwrap_or_else(|error| panic!("{id}: {error}"))
        .body;
}

#[test]
fn Test_Every_Shipped_Profile_Should_Parse_And_Be_Distinct()
{
    let catalogue = Shipped();

    assert_eq!(catalogue.Profiles().len(), 14, "the plan ships fourteen profiles");
    let identifiers: BTreeSet<&str> = catalogue
        .Profiles()
        .iter()
        .map(|profile| return profile.id.as_str())
        .collect();
    assert_eq!(identifiers.len(), 14, "two profiles share an identifier");
}

#[test]
fn Test_Every_Format_Should_Be_Reached_By_A_Shipped_Profile()
{
    let catalogue = Shipped();

    for format in Format::All()
    {
        assert!(
            catalogue
                .Profiles()
                .iter()
                .any(|profile| return profile.format == *format),
            "{} is a renderer no profile can reach",
            format.Label()
        );
    }
}

#[test]
fn Test_Every_Content_Kind_Should_Be_Reached_By_A_Shipped_Profile()
{
    let catalogue = Shipped();

    for content in Content::All()
    {
        assert!(
            catalogue.Profiles().iter().any(|profile| {
                return profile
                    .sections
                    .iter()
                    .any(|section| return section.content == *content);
            }),
            "{} is selectable and nothing selects it",
            content.Label()
        );
    }
}

#[test]
fn Test_Every_Shipped_Profile_Should_Render_Content_From_The_Store()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, profile).unwrap_or_else(|error| panic!("{error}"));

        assert!(!output.body.trim().is_empty(), "{} rendered nothing", profile.id);
        assert!(
            output.stamp.sections.iter().all(|(_, items)| return *items > 0),
            "{} rendered a section with no item: {:?}",
            profile.id,
            output.stamp.sections
        );
        assert!(!output.stamp.inputs.is_empty(), "{} consumed nothing", profile.id);
    }
}

/// P3, first half.
#[test]
fn Test_A_Rebuild_Should_Be_Byte_Identical()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let first = Build(&store, profile).expect("builds");
        let second = Build(&store, profile).expect("rebuilds");

        assert_eq!(first.body, second.body, "{} does not rebuild to itself", profile.id);
        assert_eq!(first.Sidecar().expect("stamps"), second.Sidecar().expect("stamps"));
    }
}

/// P3, second half.
#[test]
fn Test_Building_From_A_Fresh_Import_Should_Equal_Building_From_The_Original()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports").Write().expect("writes");
    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &Bundle::Parse(&bundle).expect("parses")).expect("imports");

    for profile in Shipped().Profiles()
    {
        let original = Build(&source, profile).expect("builds from the original");
        let imported = Build(&rebuilt, profile).expect("builds from the import");

        assert_eq!(
            original.body, imported.body,
            "{} renders differently after a bundle round trip",
            profile.id
        );
        assert_eq!(original.stamp, imported.stamp, "{}", profile.id);
    }
}

/// Insertion order decides `uid`, and nothing in a projection may be ordered by one.
#[test]
fn Test_Insertion_Order_Should_Not_Reach_The_Output()
{
    let forwards = Populated_In_Order(false);
    let backwards = Populated_In_Order(true);

    for profile in Shipped().Profiles()
    {
        let first = Build(&forwards, profile).expect("builds");
        let second = Build(&backwards, profile).expect("builds");

        assert_eq!(
            first.body, second.body,
            "{} orders its output by the order the store was written in",
            profile.id
        );
    }
}

#[test]
fn Test_Every_Body_Should_Be_Lf_Utf8_Without_A_Bom()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, profile).expect("builds");

        assert!(!output.body.contains('\r'), "{} carries a carriage return", profile.id);
        assert!(
            !output.body.starts_with('\u{feff}'),
            "{} starts with a byte order mark",
            profile.id
        );
        assert!(
            !output.Sidecar().expect("stamps").contains('\r'),
            "{}: the sidecar carries a carriage return",
            profile.id
        );
    }
}

#[test]
fn Test_Every_Body_Should_Declare_Itself_Generated()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, profile).expect("builds");

        assert!(
            output.body.contains("nomos_generated") || output.body.contains("nomos-generated"),
            "{} does not say it is generated",
            profile.id
        );
        assert!(
            output.body.contains("do_not_edit") || output.body.contains("do-not-edit"),
            "{} does not say it must not be edited",
            profile.id
        );
        assert!(
            output.body.contains(DO_NOT_EDIT.get(..40).unwrap_or(DO_NOT_EDIT))
                || output.body.contains("Generated by nomos"),
            "{} carries the marker without the sentence that explains it",
            profile.id
        );
    }
}

/// The done-when, checked against the environment this build actually runs in rather than
/// against a pattern that guesses what one looks like.
#[test]
fn Test_No_Body_Should_Carry_This_Machine()
{
    let store = Populated();
    let directory = std::env::current_dir().expect("has a working directory");
    let temporary = std::env::temp_dir();
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| return std::env::var("HOSTNAME"))
        .unwrap_or_default();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, profile).expect("builds");

        for environmental in [
            directory.display().to_string(),
            temporary.display().to_string(),
            host.clone(),
        ]
        {
            if environmental.trim().is_empty()
            {
                continue;
            }
            assert!(
                !output.body.contains(&environmental),
                "{} carries {environmental}, so the output depends on the machine that built it",
                profile.id
            );
        }

        assert!(
            !output.body.contains("://") && !output.body.contains(":\\"),
            "{} carries an absolute location",
            profile.id
        );
    }
}

#[test]
fn Test_A_Soft_Deleted_Node_Should_Not_Be_Projected()
{
    let store = Populated();

    let rendered = Rendered(&store, "mcp-resource");

    assert!(rendered.contains("AGT-EXEC-001"), "the fixture projected no node at all");
    assert!(
        !rendered.contains("REQ-RETIRED-009"),
        "a deleted node reached a projection"
    );
}

#[test]
fn Test_A_Section_That_Selects_Nothing_Should_Be_Refused()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "empty", "title": "Empty", "format": "markdown", "output": "empty.md",
             "sections": [{ "title": "Ghosts", "content": "nodes",
                            "filter": { "kind": "ghost" } }] }"#,
    )
    .expect("parses");

    let refusal = Build(&store, &profile).expect_err("must refuse");

    assert!(format!("{refusal}").contains("selected no nodes"), "{refusal}");
}

#[test]
fn Test_A_Section_That_Declares_It_May_Be_Empty_Should_Render()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "empty", "title": "Empty", "format": "markdown", "output": "empty.md",
             "sections": [{ "title": "Ghosts", "content": "nodes", "may_be_empty": true,
                            "filter": { "kind": "ghost" } },
                          { "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");

    let output = Build(&store, &profile).expect("builds");

    assert!(output.body.contains("## Ghosts"), "{}", output.body);
}

#[test]
fn Test_A_Filter_A_Content_Kind_Does_Not_Honour_Should_Be_Refused()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "misfiltered", "title": "Misfiltered", "format": "markdown",
             "output": "misfiltered.md",
             "sections": [{ "title": "Suites", "content": "suites",
                            "filter": { "relation_type": "verifies" } }] }"#,
    )
    .expect("parses");

    let refusal = Build(&store, &profile).expect_err("must refuse");

    assert!(format!("{refusal}").contains("relation_type"), "{refusal}");
}

#[test]
fn Test_Two_Profiles_Writing_One_Output_Should_Be_Refused()
{
    let one = Profile::Parse(
        r#"{ "id": "one", "title": "One", "format": "markdown", "output": "shared.md",
             "sections": [{ "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");
    let two = Profile::Parse(
        r#"{ "id": "two", "title": "Two", "format": "markdown", "output": "shared.md",
             "sections": [{ "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");

    let Err(refusal) = Catalogue::Of(vec![one, two])
    else
    {
        panic!("two profiles writing one output were accepted");
    };

    assert!(format!("{refusal}").contains("shared.md"), "{refusal}");
}

#[test]
fn Test_A_Selection_Should_Order_By_Identity_Rather_Than_By_Arrival()
{
    let store = Populated();
    let projection = Select(&store, &Profile_Named("mcp-resource")).expect("selects");

    for section in &projection.sections
    {
        let identities: Vec<&str> = section
            .items
            .iter()
            .map(|item| return item.identity.as_str())
            .collect();
        let mut sorted = identities.clone();
        sorted.sort_unstable();

        assert_eq!(identities, sorted, "{} is not ordered by identity", section.title);
    }
}

#[test]
fn Test_The_Sidecar_Should_Round_Trip()
{
    let store = Populated();
    let output = Build(&store, &Profile_Named("domain-specification")).expect("builds");

    let parsed = Stamp::Parse(&output.Sidecar().expect("stamps")).expect("parses");

    assert_eq!(parsed, output.stamp);
    assert_eq!(output.sidecar_path, format!("{}{SIDECAR_SUFFIX}", output.path));
}

#[test]
fn Test_An_Unchanged_Store_Should_Be_Fresh()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");

    let freshness = Check(
        &store,
        &profile,
        Some(&built.body),
        Some(&built.Sidecar().expect("stamps")),
    )
    .expect("checks");

    assert!(freshness.Is_Fresh(), "{}", freshness.Report(&built.path));
}

#[test]
fn Test_A_Changed_Store_Should_Be_Stale()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");

    store
        .Connection()
        .execute(
            "UPDATE nodes SET title = 'Renamed' WHERE node_id = 'AGT-EXEC-001'",
            [],
        )
        .expect("changes the store");

    let freshness = Check(
        &store,
        &profile,
        Some(&built.body),
        Some(&built.Sidecar().expect("stamps")),
    )
    .expect("checks");

    assert!(freshness.stale.is_some(), "a changed store reads as current");
    assert!(freshness.Report(&built.path).contains("stale"));
}

#[test]
fn Test_A_Changed_Profile_Should_Be_Stale()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");
    let mut retitled = profile.clone();
    retitled.title = "Renamed resource".to_owned();

    let freshness = Check(
        &store,
        &retitled,
        Some(&built.body),
        Some(&built.Sidecar().expect("stamps")),
    )
    .expect("checks");

    assert!(freshness.stale.is_some(), "a rewritten profile reads as current");
}

#[test]
fn Test_An_Edited_Output_Should_Be_Reported_As_Edited()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");
    let tampered = format!("{}\nhand written\n", built.body);

    let freshness = Check(
        &store,
        &profile,
        Some(&tampered),
        Some(&built.Sidecar().expect("stamps")),
    )
    .expect("checks");

    assert!(freshness.edited.is_some(), "a hand-edited output reads as generated");
    assert!(freshness.stale.is_none(), "the store did not change");
}

#[test]
fn Test_An_Output_That_Was_Never_Built_Should_Be_Absent()
{
    let store = Populated();

    let freshness = Check(&store, &Profile_Named("mcp-resource"), None, None).expect("checks");

    assert!(freshness.absent);
    assert!(!freshness.Is_Fresh());
    assert!(freshness.Report("mcp/specification.json").contains("never been built"));
}

#[test]
fn Test_A_Markdown_Table_Cell_Should_Not_Break_The_Table()
{
    let store = Populated();

    let rendered = Rendered(&store, "traceability-matrix");

    for line in rendered.lines().filter(|line| return line.starts_with("| "))
    {
        assert_eq!(
            line.matches(" | ").count() + 2,
            line.split(" | ").count() + 1,
            "a cell split the row: {line}"
        );
    }
    assert!(rendered.contains("verifies"), "{rendered}");
}

#[test]
fn Test_A_Diagram_Should_Name_Every_Relation_It_Draws()
{
    let store = Populated();

    let rendered = Rendered(&store, "diagram-set");

    assert!(rendered.starts_with("%% nomos_generated: true"), "{rendered}");
    assert!(rendered.contains("graph LR"), "{rendered}");
    assert!(
        rendered.contains("|\"verifies\"|"),
        "the diagram drew an edge without naming it: {rendered}"
    );
}

#[test]
fn Test_A_Context_Pack_Should_Carry_Its_Inputs_Digest()
{
    let store = Populated();
    let profile = Profile_Named("implementation-context-pack");
    let output = Build(&store, &profile).expect("builds");

    let parsed: serde_json::Value = serde_json::from_str(&output.body).expect("is json");

    assert_eq!(
        parsed.get("inputs_digest").and_then(serde_json::Value::as_str),
        Some(output.stamp.inputs_digest.as_str())
    );
}

#[test]
fn Test_A_Yaml_Projection_Should_Parse_As_Yaml()
{
    let store = Populated();
    let output = Build(&store, &Profile_Named("contract-yaml")).expect("builds");

    let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output.body).expect("is yaml");

    assert_eq!(
        parsed.get("nomos_generated").and_then(serde_yaml_ng::Value::as_bool),
        Some(true)
    );
}

#[test]
fn Test_An_Html_Projection_Should_Escape_What_It_Renders()
{
    let mut store = Populated();
    let document = store
        .Put_Source_Document("volumes/04-escapes.md", "v14.36", "# Escapes\n\n<script>x</script>\n")
        .expect("stores");
    store
        .Put_Source_Blocks(document, &Segment("# Escapes\n\n<script>x</script>\n"))
        .expect("stores blocks");

    let rendered = Rendered(&store, "html-site");

    assert!(rendered.contains("&lt;script&gt;"), "the renderer emitted raw markup");
    assert!(!rendered.contains("<script>"), "the renderer emitted raw markup");
}

#[test]
fn Test_The_Governing_Records_Should_Project_As_A_Document_Suite()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("seeds");

    for id in ["domain-specification", "html-site"]
    {
        let profile = Profile_Named(id);
        let first = Build(&store, &profile).unwrap_or_else(|error| panic!("{id}: {error}"));
        let second = Build(&store, &profile).expect("rebuilds");

        assert_eq!(first.body, second.body, "{id} does not rebuild to itself");
        assert!(
            first.body.contains("The specification is a database"),
            "{id} projected none of the records that govern it"
        );
        assert!(
            first.stamp.inputs.len() > 100,
            "{id} consumed {} inputs from the governing records",
            first.stamp.inputs.len()
        );
    }
}
