use nomos_spec_bundle::{Bundle, Export, Import};
use nomos_spec_model::{ContentHash, Segment};
use nomos_spec_project::{
    Build, Catalogue, Check, Content, Format, Freshness, Profile, Select, Stamp, DO_NOT_EDIT,
    SIDECAR_SUFFIX,
};
use nomos_spec_store::SpecificationStore;
use std::collections::BTreeSet;

const CORE: &str = "# Core architecture\n\nIdentity is not a path.\n\n\
                    ## Domain model\n\n| Model | Owns |\n| --- | --- |\n\
                    | WorkspaceContext | the workspace |\n| BuildVariant | one build |\n\n\
                    ```rust\nlet quoted = \"a \\\"nested\\\" string\";\n```\n";

const CONFORMANCE: &str = "# Conformance\n\nUnknown is not pass \u{2014} ni\u{00f1}o, \
                           \u{4e2d}\u{6587}, \u{1f600}.\n";

/// The order the fixture's documents and graph rows arrive in.
///
/// Named rather than a bool. `Populate_Graph(store, true)` said nothing at the call site
/// about what `true` was true of, and the whole point of the fixture is that the two
/// orders must produce the same bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order
{
    Forwards,
    Backwards,
}

fn Populated() -> SpecificationStore
{
    return Populated_In_Order(Order::Forwards);
}

fn Populated_In_Order(order: Order) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut documents = vec![
        ("volumes/02-core.md", CORE),
        ("volumes/03-conformance.md", CONFORMANCE),
    ];
    if order == Order::Backwards
    {
        documents.reverse();
    }

    for (path, text) in documents
    {
        Put_Document(&mut store, path, text);
    }
    Populate_Graph(&store, order);

    return store;
}

/// One document and the blocks it segments into, which the store holds separately.
fn Put_Document(store: &mut SpecificationStore, path: &str, text: &str)
{
    let document = store
        .Put_Source_Document(path, "v14.36", text)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &Segment(text))
        .expect("stores the blocks");
}

/// The suites the fixture's two documents belong to, and the headings inside them.
const SUITES_AND_HEADINGS: &str =
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
     WHERE path = 'volumes/03-conformance.md';";

/// Every node the fixture projects, one statement per node so the set can be reversed.
const NODES: &str =
    "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
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

/// The aliases, relations, statements, lineage and omissions that hang off those nodes.
const GRAPH_EDGES: &str =
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
       AND b.ordinal = 1;";

fn Populate_Graph(store: &SpecificationStore, order: Order)
{
    let mut statements: Vec<&str> = NODES.split(";\n").collect();
    statements.reverse();
    let reversed = statements.join(";\n");

    store
        .Connection()
        .execute_batch(SUITES_AND_HEADINGS)
        .expect("populates the suites and headings");
    store
        .Connection()
        .execute_batch(if order == Order::Backwards { &reversed } else { NODES })
        .expect("populates the nodes");

    Populate_Graph_Edges(store);
}

fn Populate_Graph_Edges(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(GRAPH_EDGES)
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
    return Build(store, &For_Building(&Profile_Named(id)))
        .unwrap_or_else(|error| panic!("{id}: {error}"))
        .body;
}

/// The node the subject-addressed profiles are pointed at in this fixture.
///
/// It holds a statement and sits at the *target* end of its only relation, which is what
/// makes it the useful one: a subject shown only the edges it starts would render an empty
/// Relations section here, and every assertion below would still pass.
const SUBJECT_IN_FIXTURE: &str = "AGT-EXEC-001";

/// A profile the catalogue-wide assertions can build.
///
/// A subject-addressed profile is a template and `Build` refuses one, so a loop over every
/// shipped profile has to say which subject it means. Resolving here rather than skipping
/// is the point: skipping would quietly drop four profiles from every assertion in this
/// file, and the assertions would go on reading as though they covered the catalogue.
fn For_Building(profile: &Profile) -> Profile
{
    return profile
        .For(profile.Names_A_Subject().then_some(SUBJECT_IN_FIXTURE))
        .unwrap_or_else(|error| panic!("{}: {error}", profile.id));
}

#[test]
fn Test_Every_Shipped_Profile_Should_Parse_And_Be_Distinct()
{
    let catalogue = Shipped();

    assert_eq!(
        catalogue.Profiles().len(),
        18,
        "fourteen whole-store profiles and four subject-addressed ones"
    );
    let identifiers: BTreeSet<&str> = catalogue
        .Profiles()
        .iter()
        .map(|profile| return profile.id.as_str())
        .collect();
    assert_eq!(identifiers.len(), 18, "two profiles share an identifier");
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
        let output = Build(&store, &For_Building(profile))
            .unwrap_or_else(|error| panic!("{error}"));

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
        let buildable = For_Building(profile);
        let first = Build(&store, &buildable).expect("builds");
        let second = Build(&store, &buildable).expect("rebuilds");

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
        let buildable = For_Building(profile);
        let original = Build(&source, &buildable).expect("builds from the original");
        let imported = Build(&rebuilt, &buildable).expect("builds from the import");

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
    let forwards = Populated_In_Order(Order::Forwards);
    let backwards = Populated_In_Order(Order::Backwards);

    for profile in Shipped().Profiles()
    {
        let buildable = For_Building(profile);
        let first = Build(&forwards, &buildable).expect("builds");
        let second = Build(&backwards, &buildable).expect("builds");

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
        let output = Build(&store, &For_Building(profile)).expect("builds");

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
        let output = Build(&store, &For_Building(profile)).expect("builds");

        Assert_Declares_Itself_Generated(&output.body, &profile.id);
    }
}

/// A body has to say it is generated, say it must not be edited, and carry the sentence that
/// explains the marker rather than the marker on its own.
fn Assert_Declares_Itself_Generated(body: &str, id: &str)
{
    assert!(
        body.contains("nomos_generated") || body.contains("nomos-generated"),
        "{id} does not say it is generated"
    );
    assert!(
        body.contains("do_not_edit") || body.contains("do-not-edit"),
        "{id} does not say it must not be edited"
    );
    assert!(
        body.contains(DO_NOT_EDIT.get(..40).unwrap_or(DO_NOT_EDIT))
            || body.contains("Generated by nomos"),
        "{id} carries the marker without the sentence that explains it"
    );
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
        let output = Build(&store, &For_Building(profile)).expect("builds");

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

/// The freshness of a body and the stamp beside it, against the store they came from.
fn Checked(store: &SpecificationStore, profile: &Profile, body: &str, sidecar: &str) -> Freshness
{
    return Check(store, profile, Some(body), Some(sidecar)).expect("checks");
}

#[test]
fn Test_An_Unchanged_Store_Should_Be_Fresh()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");

    let freshness = Checked(&store, &profile, &built.body, &built.Sidecar().expect("stamps"));

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
        .execute("UPDATE nodes SET title = 'Renamed' WHERE node_id = 'AGT-EXEC-001'", [])
        .expect("changes the store");
    let freshness = Checked(&store, &profile, &built.body, &built.Sidecar().expect("stamps"));

    assert!(freshness.stale.is_some(), "a changed store reads as current");
    assert!(freshness.Report(&built.path).contains("stale"));
    // The store moving is not the file being written by hand, and the third verdict must not
    // swallow the first. This body does differ from what the store now renders, so a
    // divergence check asked unconditionally would report both and `diverged` would stop
    // naming anything in particular.
    assert!(
        freshness.diverged.is_none(),
        "a store that moved under an untouched file reads as hand-written: {}",
        freshness.Report(&built.path)
    );
}

#[test]
fn Test_A_Changed_Profile_Should_Be_Stale()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");
    let mut retitled = profile.clone();
    retitled.title = "Renamed resource".to_owned();

    let freshness = Checked(&store, &retitled, &built.body, &built.Sidecar().expect("stamps"));

    assert!(freshness.stale.is_some(), "a rewritten profile reads as current");
}

#[test]
fn Test_An_Edited_Output_Should_Be_Reported_As_Edited()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");
    let tampered = format!("{}\nhand written\n", built.body);

    let freshness = Checked(&store, &profile, &tampered, &built.Sidecar().expect("stamps"));

    assert!(freshness.edited.is_some(), "a hand-edited output reads as generated");
    assert!(freshness.stale.is_none(), "the store did not change");
    assert!(
        freshness.diverged.is_none(),
        "an edit its own stamp already contradicts is reported twice: {}",
        freshness.Report(&built.path)
    );
}

/// The case the other three cannot see.
///
/// `stale` compares the stamp's inputs against the store and `edited` compares the stamp's
/// digest against the file, so a body and the `content_digest` describing it, rewritten
/// together, agree with each other and satisfy both. Before this, that reported "is
/// current" and exited 0 — which is what the gate step added by P10-REQUIRED-PROJECTIONS
/// was resting on, since the commit that ships a projection can rewrite the stamp beside it.
///
/// `Test_An_Edited_Output_Should_Be_Reported_As_Edited` leaves the stamp alone, so it is
/// caught by the digest comparison and never reaches the bytes.
#[test]
fn Test_A_Stamp_Rewritten_To_Agree_With_An_Edited_Body_Should_Still_Be_Refused()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("builds");
    let tampered = format!("{}\nhand written\n", built.body);
    let mut agreeing = built.stamp.clone();
    agreeing.content_digest = ContentHash::Of(&tampered).As_Str().to_owned();
    let freshness = Checked(&store, &profile, &tampered, &agreeing.Render().expect("stamps"));

    assert!(
        !freshness.Is_Fresh(),
        "a body and a stamp rewritten together read as current: {}",
        freshness.Report(&built.path)
    );
    assert!(
        freshness.diverged.is_some(),
        "the refusal did not come from the comparison against what the store renders"
    );
    Assert_Divergence_Is_The_Only_Verdict(&freshness, &built.path);
}

/// Neither of the old two verdicts may claim the rewritten-stamp case. The stamp is internally
/// consistent, so saying "the stamp declares X and the file hashes to Y" would be false — they
/// are the same value — and the store never moved. The report needs words of its own too.
fn Assert_Divergence_Is_The_Only_Verdict(freshness: &Freshness, path: &str)
{
    assert!(freshness.edited.is_none(), "the stamp agrees with the file it describes");
    assert!(freshness.stale.is_none(), "the store did not change");

    let said = freshness.Report(path);

    assert!(said.contains("diverged"), "the new case has no words of its own: {said}");
    assert!(!said.contains("edited"), "the new case reuses the edited sentence: {said}");
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

// ---------------------------------------------------------------------------------------
// Subject-addressed projections.
//
// Every profile above projects the whole store into one fixed path. A subject profile is
// the same machinery pointed at one node, which is what lets four artefacts about a
// subject be four renderings of one record graph rather than four authorities. The
// placeholder is the whole of the mechanism: it appears in the output path and in the
// filters, and a run supplies what it stands for.

/// A store holding two nodes whose identifiers share a prefix.
fn With_Overlapping_Identifiers() -> SpecificationStore
{
    let store = Populated();
    store
        .Connection()
        .execute_batch(
            "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                    suite_uid)
             SELECT 'SUBJ-1', 'concept', 'canonical', 'record', 'One', NULL, uid
             FROM suites WHERE suite_id = 'nomos';
             INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                    suite_uid)
             SELECT 'SUBJ-12', 'concept', 'canonical', 'record', 'Twelve', NULL, uid
             FROM suites WHERE suite_id = 'nomos';",
        )
        .expect("adds the overlapping identifiers");

    return store;
}

fn Nodes_Profile(filter: &str) -> Profile
{
    return Profile::Parse(&format!(
        "{{ \"id\": \"probe\", \"title\": \"Probe\", \"format\": \"markdown\", \
         \"output\": \"probe.md\", \"sections\": [{{ \"title\": \"Nodes\", \
         \"content\": \"nodes\", \"filter\": {filter} }}] }}"
    ))
    .expect("parses");
}

/// The reason `node_id` exists rather than reusing `identifier_prefix`.
///
/// A prefix is not an identity. Asked for `SUBJ-1` a prefix also answers with `SUBJ-12`, so
/// a per-subject artefact built on one would carry another subject's rows under a heading
/// naming this one.
#[test]
fn Test_An_Identity_Should_Select_One_Node_Where_A_Prefix_Selects_Two()
{
    let store = With_Overlapping_Identifiers();

    let exact = Build(&store, &Nodes_Profile("{ \"node_id\": \"SUBJ-1\" }")).expect("builds");
    let prefixed = Build(&store, &Nodes_Profile("{ \"identifier_prefix\": \"SUBJ-1\" }"))
        .expect("builds");

    assert!(exact.body.contains("SUBJ-1"), "{}", exact.body);
    assert!(
        !exact.body.contains("SUBJ-12"),
        "an identity matched a longer one: {}",
        exact.body
    );
    assert!(prefixed.body.contains("SUBJ-12"), "{}", prefixed.body);
}

/// Two subjects, two outputs, neither carrying the other.
#[test]
fn Test_One_Profile_Should_Serve_Every_Subject()
{
    let store = Populated();
    let declared = Profile_Named("subject-dossier");

    let first = Build(&store, &declared.For(Some("AGT-EXEC-001")).expect("resolves"))
        .expect("builds the first subject");
    let second =
        Build(&store, &declared.For(Some("D-129")).expect("resolves")).expect("builds the second");

    assert_eq!(first.path, "subjects/AGT-EXEC-001/dossier.md");
    assert_eq!(second.path, "subjects/D-129/dossier.md");
    assert!(!first.body.contains("D-129"), "a subject carried another: {}", first.body);
    assert!(
        !second.body.contains("AGT-EXEC-001"),
        "a subject carried another: {}",
        second.body
    );
}

/// A subject is shown the edges declared about it, not only the ones it declares.
///
/// `AGT-EXEC-001` sits at the target end of its only relation. Narrowing the `from` column
/// alone would render an empty Relations section here and every other assertion in this
/// file would still pass, which is why this one names the other end explicitly.
#[test]
fn Test_A_Subject_Should_See_The_Relations_At_Either_End()
{
    let store = Populated();

    let built = Build(
        &store,
        &Profile_Named("subject-dossier")
            .For(Some("AGT-EXEC-001"))
            .expect("resolves"),
    )
    .expect("builds");

    assert!(
        built.body.contains("CDM-WORKSPACECONTEXT"),
        "the relation pointing at this subject is missing: {}",
        built.body
    );
}

/// One selection, four formats — the claim the four profiles exist to make.
#[test]
fn Test_Every_Subject_Profile_Should_Render_The_Same_Subject_In_Its_Own_Format()
{
    let store = Populated();
    let expected = [
        ("subject-dossier", Format::Markdown, "subjects/AGT-EXEC-001/dossier.md"),
        ("subject-contract", Format::Yaml, "subjects/AGT-EXEC-001/contract.yaml"),
        ("subject-model", Format::Json, "subjects/AGT-EXEC-001/model.json"),
        ("subject-report", Format::Html, "subjects/AGT-EXEC-001/report.html"),
    ];

    let mut inputs = BTreeSet::new();
    for (id, format, path) in expected
    {
        let digest = Subject_Digest(&store, id, format, path);
        inputs.insert(digest);
    }

    assert_eq!(
        inputs.len(),
        1,
        "the four formats disagree about what they read, so they are not one selection"
    );
}

/// Builds one subject profile, asserts it declared and landed where it said, and returns the
/// digest of what it read.
fn Subject_Digest(store: &SpecificationStore, id: &str, format: Format, path: &str) -> String
{
    let declared = Profile_Named(id);
    let built =
        Build(store, &declared.For(Some("AGT-EXEC-001")).expect("resolves")).expect("builds");

    assert_eq!(declared.format, format, "{id}");
    assert_eq!(built.path, path, "{id}");
    assert!(built.body.contains("AGT-EXEC-001"), "{id} lost its subject");

    return built.stamp.inputs_digest;
}

/// A profile that names a subject, run without one.
#[test]
fn Test_A_Subject_Profile_Without_A_Subject_Should_Be_Refused()
{
    let error = Profile_Named("subject-dossier").For(None).expect_err("must refuse");

    let said = error.to_string();
    assert!(said.contains("subject-dossier"), "{said}");
    assert!(said.contains("--subject"), "the refusal does not say what to do: {said}");
}

/// A subject given to a profile with nowhere to put it.
#[test]
fn Test_A_Whole_Store_Profile_Given_A_Subject_Should_Be_Refused()
{
    let error = Profile_Named("diagram-set")
        .For(Some("AGT-EXEC-001"))
        .expect_err("must refuse");

    let said = error.to_string();
    assert!(said.contains("diagram-set"), "{said}");
    assert!(said.contains("AGT-EXEC-001"), "{said}");
}

/// The last guard: a template must not reach the renderer.
///
/// `Resolved_For` is the only thing that removes the placeholder, so a profile arriving at
/// `Build` still holding one was never resolved. Rendering it would create a directory
/// literally named for the placeholder and report success.
#[test]
fn Test_An_Unresolved_Template_Should_Not_Be_Built()
{
    let store = Populated();

    let error = Build(&store, &Profile_Named("subject-dossier")).expect_err("must refuse");

    let said = error.to_string();
    assert!(said.contains("subjects/"), "{said}");
    assert!(said.contains("dossier.md"), "{said}");
}

/// A subject the store does not hold is empty, and an empty projection is already refused.
#[test]
fn Test_A_Subject_That_Matches_Nothing_Should_Not_Render_An_Empty_File()
{
    let store = Populated();

    let error = Build(
        &store,
        &Profile_Named("subject-dossier")
            .For(Some("NO-SUCH-SUBJECT"))
            .expect("resolves"),
    )
    .expect_err("must refuse");

    let said = error.to_string();
    assert!(said.contains("Subject"), "the empty section is not named: {said}");
    assert!(said.contains("selected no nodes"), "{said}");
}

/// The whole-store profiles are untouched by any of this.
#[test]
fn Test_A_Whole_Store_Profile_Should_Render_Exactly_What_It_Did_Before()
{
    let store = Populated();

    for id in ["diagram-set", "traceability-matrix", "domain-specification"]
    {
        let declared = Profile_Named(id);

        assert!(!declared.Names_A_Subject(), "{id} unexpectedly names a subject");
        assert_eq!(
            Build(&store, &declared).expect("builds").body,
            Build(&store, &declared.For(None).expect("resolves"))
                .expect("builds")
                .body,
            "{id} renders differently when resolved against no subject"
        );
    }
}

/// A subject reaches the filesystem, so the path guard now faces caller input.
///
/// `Path_Is_Relative` was written to check a path an author committed to a profile. A
/// subject makes part of that path something a run supplies, which is a different threat:
/// the profile is honest and the argument is not. The guard already refuses `..`, a drive
/// letter and a backslash, and this is the assertion that keeps it refusing them once the
/// path stopped being fully authored.
#[test]
fn Test_A_Subject_Should_Not_Be_Able_To_Escape_The_Build_Root()
{
    let declared = Profile_Named("subject-dossier");

    for escape in ["../../escaped", "..\\windows", "C:/absolute", "a/../../b"]
    {
        let resolved = declared.For(Some(escape)).expect("resolves");

        assert!(
            resolved.Validate().is_err(),
            "the subject {escape:?} produced the writable path {}",
            resolved.output
        );
    }

    assert!(
        declared.For(Some("D-129")).expect("resolves").Validate().is_ok(),
        "an ordinary identifier was refused"
    );
}


// ---------------------------------------------------------------------------------------
// Rebuildability: which profiles a checkout with no corpus can render.
//
// The three corpora live outside this repository and CI has none of them, so the store a
// runner assembles holds this repository's governing records and nothing else. Which
// profiles that store can answer for decides which ones may be *required* by the gate, and
// it was folklore until it was written down here.
//
// The declaration lives in this test file rather than in the crate's public surface, and
// that is a choice rather than a shortcut. Nothing in the crate or the host reads it — the
// gate reads --require and the renderer reads sections — so a public method would be API
// with no consumer, which is the shape this repository argues against wherever it decides
// that only what something uses should exist. What the declaration is *for* is the
// comparison below, so it lives with the comparison. `OD-PROJECT-002` states the rule for a
// reader.

/// The content kinds a store holding only the governing records can answer for.
///
/// Wrong the first time it was written: `Rows` was missing. Governing records carry markdown
/// tables and `Write_Record` writes their rows, and the census below is what said so.
///
/// `Seed_Governing_Records` writes nodes, source documents, source blocks, headings, table
/// rows, front matter and relations, and the heading and block dispositions it records are
/// lineage. It writes no suites, no normative statements and no omissions.
const SEEDED_BY_RECORDS: &[Content] = &[
    Content::Documents,
    Content::Headings,
    Content::Blocks,
    Content::Rows,
    Content::Nodes,
    Content::Relations,
    Content::Lineage,
];

/// Whether every section of a profile reaches a kind a seeded store answers for.
///
/// **Necessary for rebuildability and not sufficient**, and the name is chosen to say so. An
/// earlier version of this was called `Rebuildable_From_Records` and returned true for
/// `feature-design`, which refuses on a seeded store: its `Concepts` section reaches `Nodes`,
/// a seeded kind, and filters it on node kind `concept`, which only a corpus has. A governing
/// record is kind `decision` or `architecture` and nothing else.
///
/// So the missing half is data-dependent and cannot be read off a profile at all. Whether a
/// filter selects anything is a question about a store, which is why requirability is
/// established by rendering and this is only the cheap screen in front of it.
fn Reaches_Only_Seeded_Content(profile: &Profile) -> bool
{
    return profile
        .sections
        .iter()
        .all(|section| return SEEDED_BY_RECORDS.contains(&section.content));
}

/// A one-section profile used to ask a store whether it can answer for one content kind.
///
/// `may_be_empty` is set so the question comes back as a count rather than as a refusal — the
/// refusal is what production wants and is the wrong instrument for a census.
fn Probe(content: Content) -> Profile
{
    return Profile {
        id: format!("probe-{}", content.Label()),
        title: format!("Probe {}", content.Label()),
        format: Format::Markdown,
        output: format!("probe/{}.md", content.Label()),
        sections: vec![nomos_spec_project::ProfileSection {
            title: format!("{content:?}"),
            content,
            filter: nomos_spec_project::Filter::default(),
            may_be_empty: true,
        }],
    };
}

fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("seeds");

    return store;
}

/// A set of profile identifiers, so the assertions below compare sets rather than counts.
fn Named(ids: &[&str]) -> BTreeSet<String>
{
    return ids.iter().map(|id| return (*id).to_owned()).collect();
}

/// How many items a `may_be_empty` probe for one content kind came back with.
fn Answered_Items(store: &SpecificationStore, content: Content) -> usize
{
    let probe = Probe(content);
    let projection = Select(store, &probe).expect("a may_be_empty probe never refuses");

    return projection
        .sections
        .first()
        .map_or(0, |section| return section.items.len());
}

/// The declaration against a store that was really seeded, in both directions.
///
/// One direction alone is the failure a completeness check runs into every time it is written
/// once. Asserting only that every declared kind answers would let the list shrink to nothing
/// and still pass; asserting only that every undeclared kind is silent would let it grow to
/// all ten. So the two sets are compared for equality.
#[test]
fn Test_A_Seeded_Store_Should_Answer_For_Exactly_The_Declared_Content_Kinds()
{
    let store = Seeded();
    let mut answered: BTreeSet<&str> = BTreeSet::new();
    for content in Content::All()
    {
        if Answered_Items(&store, *content) > 0
        {
            answered.insert(content.Label());
        }
    }
    let declared: BTreeSet<&str> = SEEDED_BY_RECORDS
        .iter()
        .map(|content| return content.Label())
        .collect();

    assert_eq!(
        answered, declared,
        "SEEDED_BY_RECORDS disagrees with what Seed_Governing_Records writes. A kind the \
         store answered for but the list omits would make a rebuildable profile look \
         corpus-backed; a kind the list claims but the store cannot answer for would make a \
         profile requirable that refuses on every runner."
    );
}

/// What one pass over the shipped profiles found each of them doing over a seeded store.
#[derive(Default)]
struct Outcomes
{
    renders: BTreeSet<String>,
    screened_but_refuses: BTreeSet<String>,
    skipped: BTreeSet<String>,
}

/// One pass over the shipped profiles, sorting each into what it did.
fn Screen_The_Shipped_Profiles(store: &SpecificationStore) -> Outcomes
{
    let mut screened = Outcomes::default();
    for profile in Shipped().Profiles()
    {
        // A subject profile refuses for a reason that is not about the corpus, so it would
        // report a false disagreement. Excluded, and the exclusion is asserted below so it
        // cannot quietly grow.
        if profile.Names_A_Subject()
        {
            screened.skipped.insert(profile.id.clone());
        }
        else
        {
            Sort_One_Profile(store, profile, &mut screened);
        }
    }

    return screened;
}

/// Whether one profile renders, and whether the screen said it would.
fn Sort_One_Profile(store: &SpecificationStore, profile: &Profile, screened: &mut Outcomes)
{
    let reaches = Reaches_Only_Seeded_Content(profile);
    let rendered = Build(store, profile).is_ok();

    assert!(
        !rendered || reaches,
        "{} renders over a seeded store while reaching a kind SEEDED_BY_RECORDS does not \
         admit, so the declaration is missing a kind that is really seeded",
        profile.id
    );

    if rendered
    {
        screened.renders.insert(profile.id.clone());
    }
    else if reaches
    {
        screened.screened_but_refuses.insert(profile.id.clone());
    }
}

/// The screen against the behaviour, in the one direction that holds — and the witnesses to
/// the direction that does not.
///
/// Rendering implies reaching only seeded kinds. The converse fails, and the two profiles that
/// pass the screen and still refuse are named, because reading the screen as the answer is the
/// error this asserts against. `api-documentation` and `requirement-catalog` refuse for the
/// same filter reason but appear in neither set: they also reach `statements` and so fail the
/// screen first, which is why the witness list is shorter than the list of profiles a
/// corpus-kind filter stops.
///
/// The set that renders is pinned too. Without that, a change making a rebuildable profile
/// refuse would move it quietly into the witness list and pass.
#[test]
fn Test_Rendering_Over_A_Seeded_Store_Should_Imply_Reaching_Only_Seeded_Content()
{
    let store = Seeded();
    let screened = Screen_The_Shipped_Profiles(&store);

    assert_eq!(
        screened.renders,
        Named(&["diagram-set", "domain-specification", "html-site", "traceability-matrix"]),
        "the set of profiles that render without a corpus moved. This is the set the Required \
         projections step may draw from, so a change here changes what the gate can ask for."
    );
    assert_eq!(
        screened.screened_but_refuses,
        Named(&["feature-design", "release-specification"]),
        "the witnesses to Reaches_Only_Seeded_Content being insufficient moved. Each reaches \
         only seeded kinds and still refuses, because it filters nodes on a kind only a corpus \
         has. An empty set here would mean the screen had silently become the answer."
    );
    assert_eq!(
        screened.skipped,
        Named(&["subject-contract", "subject-dossier", "subject-model", "subject-report"]),
        "the set of profiles excluded from this comparison moved. A profile excluded here is a \
         profile this test says nothing about."
    );
}

/// Every profile the gate requires, rendered rather than predicted.
///
/// This is the authority on requirability, because the screen cannot see a filter. A change
/// making a required profile refuse fails here instead of on the next person's commit, which
/// is the difference between a red gate somebody caused and one somebody inherited.
///
/// The pair is written here and in `.github/workflows/gate.yml`. That duplication is chosen:
/// the workflow is what CI runs and this is what a local `cargo test` can check, and deriving
/// one from the other would put a workflow parser in this band. `OD-PROJECT-002` records it.
#[test]
fn Test_Every_Required_Profile_Should_Render_Over_A_Seeded_Store()
{
    let store = Seeded();

    for id in ["diagram-set", "domain-specification"]
    {
        let profile = Profile_Named(id);

        assert!(
            Reaches_Only_Seeded_Content(&profile),
            "{id} is required by the gate and reaches a content kind no runner can answer for"
        );
        Build(&store, &profile)
            .unwrap_or_else(|error| panic!("{id} is required by the gate and refused: {error}"));
    }
}
