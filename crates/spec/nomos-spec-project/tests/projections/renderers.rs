//! One claim per format, about the thing that format alone can get wrong.
//!
//! Markdown can have a cell break a row, mermaid can draw an edge it does not name, JSON and
//! YAML have to parse as themselves, and HTML has to escape what it is handed. None of these
//! is visible from the selection — they are all properties of the writing.

use crate::store::{For_Building, Populated, Profile_Named, Rendered_Profile_Body};
use nomos_spec_project::Build;

/// The two pipes that close off the left and right edges of a markdown table row. The interior
/// separators the row carries are the `" | "` occurrences counted below, so adding these two
/// gives the row's total pipe count to set against the number of cells it splits into.
const ROW_EDGE_PIPES: usize = 2;

#[test]
fn Test_A_Markdown_Table_Cell_Should_Not_Break_The_Table()
{
    let store = Populated();

    let rendered = Rendered_Profile_Body(&store, "traceability-matrix");

    for line in rendered.lines().filter(|line| return line.starts_with("| "))
    {
        assert_eq!(
            line.matches(" | ").count() + ROW_EDGE_PIPES,
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

    let rendered = Rendered_Profile_Body(&store, "diagram-set");

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
    // Through `For_Building` rather than directly: `OD-PROJECT-005` scoped this profile to a
    // subject, so its output path carries `{subject}` and `Build` refuses a template given
    // none. `For_Building` supplies the fixture's own subject for exactly the profiles that
    // need one, which is the same thing every other caller in this suite does.
    let profile = For_Building(&Profile_Named("implementation-context-pack"));
    let output = Build(&store, &profile)
        .expect("For_Building supplied the subject this profile's output path needs");

    let parsed: serde_json::Value =
        serde_json::from_str(&output.body).expect("the context pack renders JSON, so its body parses as JSON");

    assert_eq!(
        parsed.get("inputs_digest").and_then(serde_json::Value::as_str),
        Some(output.stamp.inputs_digest.as_str())
    );
}

#[test]
fn Test_A_Yaml_Projection_Should_Parse_As_Yaml()
{
    let store = Populated();
    let output = Build(&store, &Profile_Named("contract-yaml"))
        .expect("contract-yaml is a whole-store profile over the populated fixture");

    let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output.body)
        .expect("the contract-yaml profile renders YAML, so its body parses as YAML");

    assert_eq!(
        parsed.get("nomos_generated").and_then(serde_yaml_ng::Value::as_bool),
        Some(true)
    );
}

#[test]
fn Test_An_Html_Projection_Should_Escape_What_It_Renders()
{
    use nomos_spec_model::Segment;

    let mut store = Populated();
    let document = store
        .Put_Source_Document("volumes/04-escapes.md", "v14.36", "# Escapes\n\n<script>x</script>\n")
        .expect("the in-memory store accepted the source document this test wrote");
    store
        .Put_Source_Blocks(document, &Segment("# Escapes\n\n<script>x</script>\n"))
        .expect("stores blocks");

    let rendered = Rendered_Profile_Body(&store, "html-site");

    assert!(rendered.contains("&lt;script&gt;"), "the renderer emitted raw markup");
    assert!(!rendered.contains("<script>"), "the renderer emitted raw markup");
}

/// How many profiles this claim checks against the seeded governing records, which is the
/// length the array below is declared at.
const GOVERNING_RECORD_PROFILE_COUNT: usize = 2;

/// Every profile this claim checks against the seeded governing records.
fn Governing_Record_Profile_Ids() -> [&'static str; GOVERNING_RECORD_PROFILE_COUNT]
{
    return ["domain-specification", "html-site"];
}

#[test]
fn Test_The_Governing_Records_Should_Project_As_A_Document_Suite()
{
    use nomos_spec_store::SpecificationStore;

    let mut store = SpecificationStore::In_Memory()
        .expect("an in-memory store is constructed for this test");
    nomos_spec_store::Seed_Governing_Records(&mut store)
        .expect("the seed wrote the governing records into the store this test opened");

    for id in Governing_Record_Profile_Ids()
    {
        Assert_Profile_Projects_The_Governing_Records(&store, id);
    }
}

/// The floor a profile's input count has to clear before consuming the governing records is
/// evidence it read them rather than a handful of records that happen to be seeded.
const GOVERNING_RECORD_INPUT_FLOOR: usize = 100;

/// One profile, built twice and checked: it rebuilds to itself, it carries the governing
/// records' own text, and it consumed enough of them for the count to be more than a
/// coincidence.
fn Assert_Profile_Projects_The_Governing_Records(store: &nomos_spec_store::SpecificationStore, id: &str)
{
    let profile = Profile_Named(id);
    // Unreachable while both profiles project the governing records. The id has to be in
    // the message because the caller builds two of them and `second` below is a plain
    // `expect`: this is the only arm that can say which profile refused its first build.
    let first = Build(store, &profile).unwrap_or_else(|error| panic!("{id}: {error}"));
    let second = Build(store, &profile)
        .expect("the same profile over the same store rebuilds to the same body");

    assert_eq!(first.body, second.body, "{id} does not rebuild to itself");
    assert!(
        first.body.contains("The specification is a database"),
        "{id} projected none of the records that govern it"
    );
    assert!(
        first.stamp.inputs.len() > GOVERNING_RECORD_INPUT_FLOOR,
        "{id} consumed {} inputs from the governing records",
        first.stamp.inputs.len()
    );
}

/// A cited neighbour's declared status reaches the rendered pack.
///
/// Rendered rather than selected, because the selection already carried it and the rendering
/// dropped it — the field was computed and discarded on the way out, so every test over
/// `Select_Projection` passed while the one artifact `OD-PROJECT-005` is about said nothing.
///
/// Over the real seeded records: the claim is about governing records' declared lifecycle
/// status, and the volume fixture has front matter for two documents rather than a graph of
/// them.
#[test]
fn Test_A_Packed_Neighbour_Should_Carry_Its_Declared_Status()
{
    let mut store = nomos_spec_store::SpecificationStore::In_Memory()
        .expect("an in-memory store is constructed for this test");
    nomos_spec_store::Seed_Governing_Records(&mut store)
        .expect("the seed wrote the governing records into the store this test opened");
    let profile = Profile_Named("implementation-context-pack")
        .For(Some("OD-PROJECT-005"))
        .expect("this profile is per-subject");

    let output = Build(&store, &profile)
        .expect("the pack renders over the seeded store for the subject it names");
    let neighbourhood = Neighbourhood_Items(&output.body);

    assert!(!neighbourhood.is_empty(), "the pack cited no neighbour: {}", output.body);
    assert!(
        neighbourhood
            .iter()
            .any(|item| return item.get("status").and_then(serde_json::Value::as_str) == Some("accepted")),
        "no cited neighbour carried a declared status: {}",
        output.body
    );
}

/// The items of the pack's `Neighbourhood` section, read out of its rendered JSON body.
///
/// Empty when the pack carries no such section at all, which is why the caller asserts the
/// items are non-empty rather than treating an empty read as an answer.
fn Neighbourhood_Items(body: &str) -> Vec<serde_json::Value>
{
    let parsed: serde_json::Value =
        serde_json::from_str(body).expect("the context pack renders JSON, so its body parses as JSON");

    return parsed
        .get("sections")
        .and_then(serde_json::Value::as_array)
        .and_then(|sections| return sections.iter().find(|section| return section.get("title").and_then(serde_json::Value::as_str) == Some("Neighbourhood")))
        .and_then(|section| return section.get("items"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
}

/// An identifier naming another item of the same projection is a link to it.
///
/// Before `P95-THE-HTML-PROJECTION-RESOLVES-NO-IDENTIFIER` the HTML renderer wrote no anchor
/// at all -- no `a`, no `href`, no per-item `id` -- so every identifier a projection carried
/// was escaped text a reader could not follow, including both endpoints of every relations
/// row. The markdown renderer beside it produced output GitHub resolves for the same content,
/// so the format presumably added to improve on markdown gave a reader strictly less.
#[test]
fn Test_An_Html_Identifier_Naming_An_Item_Present_Should_Link_To_It()
{
    let rendered = nomos_spec_project::Render_Projection(&Projection_Of(&[
        ("Subjects", &[("CDM-ONE", &[][..])]),
        ("Relations", &[("CDM-ONE relates-to CDM-TWO", &[("from", "CDM-ONE"), ("to", "CDM-TWO")])]),
    ]))
    .expect("html renders");

    assert!(rendered.contains("<article id=\"cdm-one\">"), "{rendered}");
    assert!(
        rendered.contains("<a href=\"#cdm-one\">CDM-ONE</a>"),
        "an endpoint the projection carries did not become a link: {rendered}"
    );
}

/// An identifier the projection does not carry stays text rather than becoming a dead link.
///
/// A relations section names both ends of every edge, and in a subject-scoped profile the far
/// end is routinely a record the projection does not hold. A link there would scroll nowhere,
/// which is worse than the text it replaced: a dead link reads as a promise.
#[test]
fn Test_An_Html_Identifier_Naming_Nothing_Present_Should_Stay_Text()
{
    let rendered = nomos_spec_project::Render_Projection(&Projection_Of(&[
        ("Subjects", &[("CDM-ONE", &[][..])]),
        ("Relations", &[("CDM-ONE relates-to CDM-TWO", &[("from", "CDM-ONE"), ("to", "CDM-TWO")])]),
    ]))
    .expect("html renders");

    assert!(
        !rendered.contains("#cdm-two"),
        "an endpoint the projection does not carry became a dead link: {rendered}"
    );
    assert!(rendered.contains("<dd>CDM-TWO</dd>"), "{rendered}");
}

/// An identity carrying a character that would break an attribute cannot break one.
///
/// Two positions, and they are safe for different reasons. The `id` and `href` are slugs, and
/// `Slug_Of_Text` emits ASCII alphanumerics and `-` and nothing else, so nothing an author
/// writes can reach the attribute at all. The displayed text is the identity as authored and
/// is escaped. Asserting both is what says the renderer did not get one right by getting the
/// other wrong.
#[test]
fn Test_An_Html_Identity_Should_Not_Break_The_Attribute_It_Is_Addressed_By()
{
    let hostile = "A\" onload=\"x";

    let rendered = nomos_spec_project::Render_Projection(&Projection_Of(&[(
        "Subjects",
        &[(hostile, &[("names", hostile)][..])],
    )]))
    .expect("html renders");

    assert!(rendered.contains("<article id=\"a-onload-x\">"), "{rendered}");
    assert!(!rendered.contains("onload=\"x\""), "the attribute was broken: {rendered}");
    assert!(rendered.contains("&quot;"), "the displayed identity was not escaped: {rendered}");
}

/// An html projection built by hand, so a renderer claim can name the shape it is about
/// rather than depend on what a fixture store happens to hold.
fn Projection_Of(sections: &[(&str, &[(&str, &[(&str, &str)])])]) -> nomos_spec_project::Projection
{
    use nomos_spec_project::{Format, Item, Name, Section, Value};

    return nomos_spec_project::Projection {
        profile: "probe".to_owned(),
        title: "Probe".to_owned(),
        format: Format::Html,
        output: "probe.html".to_owned(),
        sections: sections
            .iter()
            .map(|(title, items)| {
                return Section {
                    title: (*title).to_owned(),
                    content: nomos_spec_project::Content::Nodes,
                    items: items
                        .iter()
                        .map(|(identity, fields)| {
                            return fields.iter().fold(Item::Of(identity), |item, (name, value)| {
                                return item.With(Name(name), Value(value));
                            });
                        })
                        .collect(),
                };
            })
            .collect(),
        inputs: Vec::new(),
    };
}
