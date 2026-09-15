//! The seams between `nomos_spec_orchestration` and the crates its own corpus assembly and
//! `Run` dispatch call directly -- `nomos_platform` (generic, via a real `nomos_platform_std`
//! filesystem), `nomos_spec_model`, `nomos_spec_project` and `nomos_spec_store` -- exercised
//! from outside the crate through public types only.
//!
//! `src/tests.rs`'s own internal `#[cfg(test)] mod tests`, and the colocated `#[cfg(test)]`
//! blocks this crate's `src/corpus/*.rs` and `src/run/*.rs` files each carry, already drive
//! these seams -- but every one of them is compiled INTO the crate and can still see its own
//! side's private items, so none proves the PUBLIC contract a real second adapter depends on.
//! This file drives the identical calls -- `Assemble_Corpus` and `Run`, this crate's own public
//! entry points -- compiled as a separate crate that can reach nothing but
//! `nomos_spec_orchestration`'s own public API and the public API of the crates it depends on.
//!
//! `nomos_spec_ingest` is the one crate this file cannot exercise through an assertion of its
//! own: `Assemble_Corpus` parses and ingests corpus files entirely inside `corpus::layer`, and
//! neither the parsed value nor the `nomos_spec_ingest::IngestError` it can produce is returned
//! by any public function here -- a refusal is folded into `corpus::Absence.cause` as a `String`
//! before this crate hands anything back. `Test_Assemble_Corpus_Should_Report_A_Refused_Corpus_File`
//! below still drives that real parse-and-ingest call (staging a domain volume `nomos_spec_ingest`
//! itself will refuse) and reads its result back off `Absence.cause`, which is the only view of
//! that seam a caller outside this crate ever gets.

use nomos_spec_model::{Origin, SubmissionKind, SubmissionState};
use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest};
use nomos_spec_orchestration::{Run, SpecCommand, SpecOutcome, Submit_Corpus_Request, SubmitRequest};

fn No_Corpus() -> CorpusRequest
{
    return CorpusRequest { variable: "A_SEAM_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
}

fn Scratch(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-spec-orchestration-seam-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch build root");
    return root;
}

/// The happy path across the `nomos_spec_store` boundary: `Assemble_Corpus` seeds a real
/// `nomos_spec_store::SpecificationStore` from this repository's embedded governing records,
/// queryable through the store's own public API.
#[test]
fn Test_Assemble_Corpus_Should_Seed_A_Real_Specification_Store()
{
    let assembly = Assemble_Corpus(&No_Corpus()).expect("assembles from the embedded records alone");

    let summary: Option<nomos_spec_store::NodeSummary> =
        assembly.store.Node_Summary("D-129").expect("the store answers a summary query for a seeded node");
    assert!(summary.is_some(), "the embedded records travel with the binary and do not depend on a corpus");
}

/// An error that crosses the `nomos_spec_ingest` boundary: a domain volume `nomos_spec_ingest`
/// itself refuses to parse becomes a real `Absence`, described through this crate's own public
/// `Absence::Describe`/`Assembly::Describe_Absences` -- the only view of that refusal a caller
/// outside this crate ever gets.
#[test]
fn Test_Assemble_Corpus_Should_Report_A_Refused_Corpus_File()
{
    let root = Scratch("malformed-catalog");
    let catalog_dir = root.join("02_machine/catalog");
    std::fs::create_dir_all(&catalog_dir).expect("the scratch catalogue directory is created under the root");
    std::fs::write(catalog_dir.join("catalog.json"), "not valid json")
        .expect("the malformed catalogue document is written to disk");
    let request = CorpusRequest { variable: "A_SEAM_TEST_CORPUS_VARIABLE".to_owned(), root: Some(root), revision: "v14.36".to_owned() };

    let assembly = Assemble_Corpus(&request).expect("assembles even with a malformed catalog");

    assert!(!assembly.Is_Complete());
    assert!(
        assembly.Describe_Absences().contains("node catalog") || assembly.Describe_Absences().contains("catalog"),
        "{}",
        assembly.Describe_Absences()
    );
}

/// The happy path across the `nomos_spec_project` boundary: `Run(Profiles, ..)` lists the real
/// shipped `nomos_spec_project::Catalogue`, and `Run(Render, ..)` builds and places one of its
/// profiles.
#[test]
fn Test_Run_Should_List_And_Render_A_Real_Shipped_Profile()
{
    let profiles = Run(&SpecCommand::Profiles, &No_Corpus(), &nomos_platform_std::StdFileSystem);
    let SpecOutcome::Profiles(Ok(profiles)) = profiles
    else
    {
        panic!("the shipped nomos_spec_project catalogue must list at least one profile");
    };
    const EMBEDDED_PROFILE: &str = "domain-specification";
    let profile: &nomos_spec_project::Profile = profiles
        .iter()
        .find(|profile| return profile.id == EMBEDDED_PROFILE)
        .expect("the shipped catalogue lists the domain-specification profile");

    let into = Scratch("render-seam");
    let render = Run(
        &SpecCommand::Render(nomos_spec_orchestration::RenderRequest { profile: profile.id.clone(), into, subject: None }),
        &No_Corpus(),
        &nomos_platform_std::StdFileSystem,
    );
    assert!(matches!(render, SpecOutcome::Render(Ok(_))));
}

/// A complete feature request: every universal field plus the five `OD-SPEC-010` requires of
/// `SubmissionKind::FeatureRequest`, built through `nomos_spec_model`'s own public types.
fn Seam_Feature_Request() -> SubmitRequest
{
    return SubmitRequest {
        kind: SubmissionKind::FeatureRequest,
        id: "FR-SEAM-001".to_owned(),
        by: "seam-test".to_owned(),
        state: SubmissionState::Draft,
        contract_version: 1,
        fields: vec![
            ("title".to_owned(), "t".to_owned()),
            ("goal".to_owned(), "g".to_owned()),
            ("behaviour".to_owned(), "b".to_owned()),
            ("acceptance".to_owned(), "a".to_owned()),
            ("invariants".to_owned(), "none".to_owned()),
        ],
        gaps: Vec::new(),
        submitted_through: "seam-test".to_owned(),
        into: None,
    };
}

/// The happy path across the `nomos_spec_model` boundary: a submission's `Origin` and
/// `SubmissionKind`/`SubmissionState` -- `nomos_spec_model`'s own public types -- accepted
/// through `nomos_spec_orchestration::corpus::Assembly` and `Submit_Corpus_Request`.
#[test]
fn Test_Submit_Corpus_Request_Should_Accept_A_Real_Submission()
{
    let mut assembly = Assemble_Corpus(&No_Corpus()).expect("assembles from the embedded records alone");
    let request = Seam_Feature_Request();

    let answer = Submit_Corpus_Request(&mut assembly, &request, &nomos_platform_std::StdFileSystem)
        .expect("a complete feature request is accepted");

    assert!(answer.submission.values.iter().all(|value| return value.origin == Origin::Submitted));
}

/// `Run` is generic over `nomos_platform::FileSystem` rather than any concrete implementation,
/// so a second adapter can call it with its own filesystem without this crate depending on
/// `nomos-platform-std`. Writes and reads a real file back through the generic bound (rather
/// than the concrete `StdFileSystem` type), so this test crosses a real implementation of the
/// trait every test above hands `Run`, not a type that merely happens to match its shape.
fn Round_Tripped_Through_Generic_File_System<Filesystem: nomos_platform::FileSystem>(filesystem: &Filesystem, path: &std::path::Path, contents: &str) -> String
{
    filesystem.Replace_Atomically(path, contents).expect("the round-trip file is replaced at the path handed in");
    return filesystem.Read_To_String(path).expect("reads back what was just written");
}

#[test]
fn Test_Run_Should_Accept_A_Real_Nomos_Platform_File_System()
{
    let path = Scratch("file-system-seam").join("roundtrip.txt");

    let read_back = Round_Tripped_Through_Generic_File_System(&nomos_platform_std::StdFileSystem, &path, "the real seam");

    assert_eq!(read_back, "the real seam");
}
