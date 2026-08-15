//! One claim per format, about the thing that format alone can get wrong.
//!
//! Markdown can have a cell break a row, mermaid can draw an edge it does not name, JSON and
//! YAML have to parse as themselves, and HTML has to escape what it is handed. None of these
//! is visible from the selection — they are all properties of the writing.

use crate::store::{Populated, Profile_Named, Rendered};
use nomos_spec_project::Build;

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
    use nomos_spec_model::Segment;

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
    use nomos_spec_store::SpecificationStore;

    let mut store = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("seeds");

    for id in ["domain-specification", "html-site"]
    {
        let profile = Profile_Named(id);
        // Unreachable while both profiles project the governing records. The id has to be in
        // the message because the loop builds two of them and `second` below is a plain
        // `expect`: this is the only arm that can say which profile refused its first build.
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
