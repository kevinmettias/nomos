//! Subject-addressed projections.
//!
//! Every profile in the rest of this suite projects the whole store into one fixed path. A
//! subject profile is the same machinery pointed at one node, which is what lets four
//! artefacts about a subject be four renderings of one record graph rather than four
//! authorities. The placeholder is the whole of the mechanism: it appears in the output path
//! and in the filters, and a run supplies what it stands for.

use crate::store::{Populated, Profile_Named};
use nomos_spec_project::{Build, Format, Profile};
use nomos_spec_store::SpecificationStore;
use std::collections::BTreeSet;

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
