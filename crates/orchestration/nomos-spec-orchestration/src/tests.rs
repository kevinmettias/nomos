//! What this crate promises, exercised at the two seams `nomos-cli` now calls through.

use std::path::PathBuf;

use nomos_platform_std::StdFileSystem;

use nomos_spec_model::{Origin, SubmissionKind, SubmissionState};

use crate::corpus::{Assemble, CorpusRequest};
use crate::command::SpecCommand;
use crate::outcome::{
    CommitAnswer, FreshnessRefusal, PreviewRefusal, RecordRefusal, RenderRefusal, Reproduction, SpecOutcome,
    SubmitRefusal, TableRefusal, Verdict,
};
use crate::request::{
    CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, SubmitRequest, TableRequest,
};
use crate::run::{Profiles, Run, Submit};

/// A corpus request naming no corpus at all, so every test below runs on a machine that
/// has never heard of the v14 corpus.
fn No_Corpus() -> CorpusRequest
{
    return CorpusRequest {
        variable: "A_SPEC_ORCHESTRATION_TEST_CORPUS_VARIABLE".to_owned(),
        root: None,
        revision: "v14.36".to_owned(),
    };
}

/// A build root under this process's own temporary directory, unique per test, so
/// `Render` and `Freshness` can be exercised against a real, disk-backed
/// `nomos-platform-std::StdFileSystem` the same way `nomos-cli` runs them.
fn Scratch(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!(
        "nomos-spec-orchestration-{name}-{}",
        std::process::id()
    ));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch build root");

    return root;
}

/// A profile that builds from the embedded governing records alone, so a test naming it
/// needs no corpus -- the same profile `crates/host/nomos-cli/tests/read_surface.rs` uses
/// for the same reason.
const EMBEDDED_PROFILE: &str = "domain-specification";

#[test]
fn Test_Profiles_Should_List_What_This_Build_Ships()
{
    let profiles = Profiles().expect("the embedded catalogue parses");

    assert!(!profiles.is_empty(), "the shipped catalogue is never empty");
    assert!(
        profiles.iter().any(|profile| return profile.id == "subject-dossier"),
        "the shipped catalogue always carries subject-dossier"
    );
}

#[test]
fn Test_Run_Of_Profiles_Should_Never_Touch_A_Store()
{
    // A request naming a variable that is not set. If `Run` assembled a store for
    // `Profiles`, this would still succeed, so the point of this test is that it succeeds
    // *without* the failure path corpus assembly could have taken ever being reachable --
    // asserted by `Test_Profiles_Should_List_What_This_Build_Ships` proving `Profiles()`
    // alone already returns the same answer, unconditionally.
    let outcome = Run(&SpecCommand::Profiles, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Profiles(profiles) = outcome
    else
    {
        panic!("Run(Profiles, ..) must answer SpecOutcome::Profiles");
    };
    assert!(profiles.is_ok());
}

#[test]
fn Test_Run_Of_Sources_Should_Report_The_Corpus_As_Absent_When_None_Is_Named()
{
    let outcome = Run(&SpecCommand::Sources, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Sources(answer) = outcome
    else
    {
        panic!("Run(Sources, ..) must answer SpecOutcome::Sources");
    };
    let answer = answer.expect("an in-memory store assembles even with no corpus");

    assert!(!answer.Is_Complete(), "no corpus was named, so this store is not whole");
    assert!(
        answer.Describe_Absences().contains("A_SPEC_ORCHESTRATION_TEST_CORPUS_VARIABLE"),
        "{}",
        answer.Describe_Absences()
    );
    assert!(
        answer.read.iter().any(|line| return line.contains("governing record")),
        "the embedded governing records are always read"
    );
}

#[test]
fn Test_Run_Of_Record_Should_Resolve_A_Governing_Record_With_No_Corpus()
{
    let request = SpecCommand::Record(RecordRequest {
        id: "D-132".to_owned(),
        revision: None,
    });

    let outcome = Run(&request, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Record(answer) = outcome
    else
    {
        panic!("Run(Record, ..) must answer SpecOutcome::Record");
    };
    let answer = answer.expect("D-132 is a governing record, embedded even with no corpus");

    assert_eq!(answer.id, "D-132");
    assert!(
        answer.document.text.starts_with("---\nid: D-132\n"),
        "{:?}",
        answer.document.text.get(..20)
    );
}

/// A node the catalog mints with no source document behind it is a real and common answer,
/// and reporting it the same way as an identifier nothing in the store recognizes would tell
/// a reader the wrong thing about which one they hit.
#[test]
fn Test_Run_Of_Record_Should_Tell_An_Unsourced_Node_From_An_Unknown_One()
{
    let known = Run(
        &SpecCommand::Record(RecordRequest {
            id: "ADR-DOC-001".to_owned(),
            revision: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );
    let SpecOutcome::Record(Err(RecordRefusal::NotFound { node, .. })) = known
    else
    {
        panic!("a node with no source document behind it must refuse NotFound");
    };
    assert!(node.is_some(), "the graph knows ADR-DOC-001 even with no document behind it");

    let unknown = Run(
        &SpecCommand::Record(RecordRequest {
            id: "D-9999".to_owned(),
            revision: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );
    let SpecOutcome::Record(Err(RecordRefusal::NotFound { node, .. })) = unknown
    else
    {
        panic!("an identifier nothing holds must refuse NotFound");
    };
    assert!(node.is_none(), "nothing in the store is identified D-9999");
}

#[test]
fn Test_Run_Of_Table_Should_Refuse_An_Address_Nothing_Matches()
{
    let outcome = Run(
        &SpecCommand::Table(TableRequest {
            document: "no-such-document.md".to_owned(),
            block: None,
            table: None,
            revision: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Table(Err(TableRefusal::NoSuchDocument)) = outcome
    else
    {
        panic!("Run(Table, ..) over an address nothing matches must refuse NoSuchDocument");
    };
}

#[test]
fn Test_Run_Of_Markdown_Should_Render_A_Governing_Record_Back_Out()
{
    let outcome = Run(
        &SpecCommand::Markdown(RecordRequest {
            id: "D-132".to_owned(),
            revision: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Markdown(projection) = outcome
    else
    {
        panic!("Run(Markdown, ..) must answer SpecOutcome::Markdown");
    };
    let projection = projection.expect("D-132 is a governing record with declared front matter");

    assert_eq!(projection.node_id, "D-132");
}

#[test]
fn Test_Run_Of_Render_Should_Place_A_Projection_Built_With_No_Corpus()
{
    let into = Scratch("render");
    let request = SpecCommand::Render(RenderRequest {
        profile: EMBEDDED_PROFILE.to_owned(),
        into: into.clone(),
        subject: None,
    });

    let outcome = Run(&request, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Render(answer) = outcome
    else
    {
        panic!("Run(Render, ..) must answer SpecOutcome::Render");
    };
    let answer = answer.expect("domain-specification builds from the embedded records alone");

    assert_eq!(answer.id, EMBEDDED_PROFILE);
    let body = std::fs::read_to_string(&answer.body).expect("the body was written");
    assert!(body.starts_with("---\nnomos_generated: true\n"), "{:?}", body.get(..40));
    let sidecar = std::fs::read_to_string(&answer.sidecar).expect("the sidecar was written");
    assert!(sidecar.contains("\"profile\": \"domain-specification\""), "{sidecar:.200}");
}

#[test]
fn Test_Run_Of_Render_Should_Refuse_An_Unknown_Profile()
{
    let into = Scratch("render-unknown");
    let request = SpecCommand::Render(RenderRequest {
        profile: "no-such-profile".to_owned(),
        into,
        subject: None,
    });

    let outcome = Run(&request, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Render(Err(RenderRefusal::NoSuchProfile { requested, known })) = outcome
    else
    {
        panic!("an unknown profile must refuse NoSuchProfile");
    };
    assert_eq!(requested, "no-such-profile");
    assert!(known.iter().any(|id| return id == EMBEDDED_PROFILE), "{known:?}");
}

/// Runs `Run(Freshness, ..)` over `into` for the embedded profile alone, with no `--require`,
/// and hands back the one profile outcome `--profile` narrows this run to.
///
/// Shared by every `Test_Run_Of_Freshness_Should_Report_*` test below: each stages `into`
/// differently (freshly rendered, empty, ...) but asks the identical question of it, and a
/// second, independent reading of the same `Run`/destructure/narrow steps would drift from
/// this one silently the way `check-intrafile-duplication` caught it drifting the first time.
fn Single_Freshness_Outcome(into: PathBuf) -> Verdict
{
    let outcome = Run(
        &SpecCommand::Freshness(FreshnessRequest {
            into,
            profile: Some(EMBEDDED_PROFILE.to_owned()),
            require: Vec::new(),
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Freshness(answer) = outcome
    else
    {
        panic!("Run(Freshness, ..) must answer SpecOutcome::Freshness");
    };
    let answer = answer.expect("a resolvable single profile does not refuse");
    let examined = answer.examined.len();
    let [outcome] = answer.examined.try_into().unwrap_or_else(|_| {
        panic!("--profile narrows this run to exactly one profile: {examined:?}");
    });

    return outcome.verdict;
}

#[test]
fn Test_Run_Of_Freshness_Should_Report_A_Freshly_Rendered_Output_As_Current()
{
    let into = Scratch("freshness-current");
    let render = Run(
        &SpecCommand::Render(RenderRequest {
            profile: EMBEDDED_PROFILE.to_owned(),
            into: into.clone(),
            subject: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );
    assert!(matches!(render, SpecOutcome::Render(Ok(_))), "the fixture did not render");

    let verdict = Single_Freshness_Outcome(into);
    assert!(
        matches!(&verdict, Verdict::Compared(Ok(freshness)) if freshness.Is_Fresh()),
        "{verdict:?}"
    );
}

#[test]
fn Test_Run_Of_Freshness_Should_Report_An_Empty_Build_Root_As_Absent()
{
    let into = Scratch("freshness-absent");

    let verdict = Single_Freshness_Outcome(into);
    assert!(matches!(verdict, Verdict::Absent), "{verdict:?}");
}

#[test]
fn Test_Run_Of_Freshness_Should_Refuse_An_Unexamined_Requirement()
{
    let into = Scratch("freshness-unexamined");

    let outcome = Run(
        &SpecCommand::Freshness(FreshnessRequest {
            into,
            profile: Some(EMBEDDED_PROFILE.to_owned()),
            require: vec!["diagram-set".to_owned()],
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Freshness(Err(FreshnessRefusal::RequirementUnexamined { requested, only })) = outcome
    else
    {
        panic!("a requirement outside --profile's narrowing must refuse RequirementUnexamined");
    };
    assert_eq!(requested, "diagram-set");
    assert_eq!(only.as_deref(), Some(EMBEDDED_PROFILE));
}

/// A governing record, rendered from the store's own rows -- the same bytes
/// `Run(Markdown, ..)` would answer, fetched here so a preview/commit test can stage an
/// edited copy of something real rather than an invented fixture.
fn Governing_Markdown(id: &str) -> String
{
    let outcome = Run(
        &SpecCommand::Markdown(RecordRequest { id: id.to_owned(), revision: None }),
        &No_Corpus(),
        &StdFileSystem,
    );
    let SpecOutcome::Markdown(projection) = outcome
    else
    {
        panic!("Run(Markdown, ..) must answer SpecOutcome::Markdown");
    };

    return projection.expect("a governing record is embedded even with no corpus").markdown;
}

/// A canonical heading rename -- the same edit
/// `crates/host/nomos-cli/tests/authoring_surface.rs` stages against a real record on disk,
/// here staged against a governing record's own embedded copy so this crate's tests need no
/// corpus and touch no file this repository tracks.
fn Renamed_Decision_Heading(markdown: &str) -> String
{
    return markdown.replace("## Decision", "## The decision");
}

/// Stages `Renamed_Decision_Heading(Governing_Markdown(id))` at `into`, as `staged.md`, and
/// hands back the path it was written to alongside the edited text itself -- so a preview
/// test can check the staging and a commit test can also check the bytes landed unchanged.
fn Staged_Heading_Rename(id: &str, into: &std::path::Path) -> (PathBuf, String)
{
    let edited = Renamed_Decision_Heading(&Governing_Markdown(id));
    let staged = into.join("staged.md");
    std::fs::write(&staged, &edited).expect("writes the staged edit");

    return (staged, edited);
}

#[test]
fn Test_Run_Of_Preview_Should_Describe_A_Staged_Edit_And_Write_Nothing()
{
    let (staged, _edited) = Staged_Heading_Rename("D-132", &Scratch("preview"));

    let outcome = Run(
        &SpecCommand::Preview(EditRequest {
            id: "D-132".to_owned(),
            from: staged,
            rename: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Preview(preview) = outcome
    else
    {
        panic!("Run(Preview, ..) must answer SpecOutcome::Preview");
    };
    let preview = preview.expect("a canonical heading rename previews cleanly");

    assert!(preview.Wording_Moved(), "a heading rename must count as wording moved");
    assert!(preview.Describe().contains("normative wording moved"), "{}", preview.Describe());
}

#[test]
fn Test_Run_Of_Preview_Should_Refuse_A_Staged_File_That_Cannot_Be_Read()
{
    let missing = PathBuf::from("no-such-staged-file-anywhere.md");
    let outcome = Run(
        &SpecCommand::Preview(EditRequest {
            id: "D-132".to_owned(),
            from: missing.clone(),
            rename: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Preview(Err(PreviewRefusal::Unreadable { path, .. })) = outcome
    else
    {
        panic!("a --from naming nothing must refuse PreviewRefusal::Unreadable");
    };
    assert_eq!(path, missing);
}

/// Runs `Run(Commit, ..)` over a staged edit already written at `staged`, into `into`, and
/// unwraps down to the [`CommitAnswer`] a clean commit produces.
fn Committed(id: &str, staged: PathBuf, into: PathBuf) -> CommitAnswer
{
    let outcome = Run(
        &SpecCommand::Commit(CommitRequest {
            edit: EditRequest {
                id: id.to_owned(),
                from: staged,
                rename: None,
            },
            into,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Commit(answer) = outcome
    else
    {
        panic!("Run(Commit, ..) must answer SpecOutcome::Commit");
    };

    return answer.expect("a canonical heading rename commits cleanly");
}

#[test]
fn Test_Run_Of_Commit_Should_Write_The_Record_And_Close_The_Round_Trip()
{
    let into = Scratch("commit");
    let (staged, edited) = Staged_Heading_Rename("D-132", &into);

    let answer = Committed("D-132", staged, into);

    assert_eq!(answer.report.node_id, "D-132");
    assert!(answer.vacated.is_none(), "this edit did not rename the record's path");
    let written = std::fs::read_to_string(&answer.destination).expect("the record was written");
    assert_eq!(written, edited, "the bytes on disk must be exactly what was staged");
    assert!(
        matches!(answer.reproduction, Ok(Reproduction::Matched { .. })),
        "the round trip must close: {:?}",
        answer.reproduction
    );
}

/// Every universal field and every field `OD-SPEC-010` requires of `SubmissionKind::
/// FeatureRequest`, the same set `crates/host/nomos-cli/tests/request_submit.rs` submits
/// through the CLI's own surface.
fn Complete_Feature_Request(id: &str, into: Option<PathBuf>) -> SubmitRequest
{
    return SubmitRequest {
        kind: SubmissionKind::FeatureRequest,
        id: id.to_owned(),
        by: "kevin".to_owned(),
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
        submitted_through: "test".to_owned(),
        into,
    };
}

// `Submit` is a sibling of `Run`, not a `SpecCommand` case -- see this crate's own top-level
// documentation, "A second command group" -- so every test below assembles the store itself,
// the same way `nomos-cli::request::Submit` does.
#[test]
fn Test_Submit_Should_Accept_A_Complete_Submission_With_Submitted_Origin()
{
    let mut assembly = Assemble(&No_Corpus()).expect("assembles from the embedded records alone");

    let request = Complete_Feature_Request("FR-ORCH-001", None);
    let answer = Submit(&mut assembly, &request, &StdFileSystem).expect("a complete feature request is accepted");

    assert_eq!(answer.submission.submitted_through, "test");
    assert!(answer.submission.values.iter().all(|value| return value.origin == Origin::Submitted));
    assert!(answer.written.is_none(), "no --into was given");
}

#[test]
fn Test_Submit_Should_Place_Its_Subject_Dossier_Projection_When_Into_Is_Given()
{
    let into = Scratch("submit");
    let mut assembly = Assemble(&No_Corpus()).expect("assembles from the embedded records alone");

    let request = Complete_Feature_Request("FR-ORCH-002", Some(into));
    let answer = Submit(&mut assembly, &request, &StdFileSystem).expect("a complete feature request is accepted");
    let written = answer.written.expect("--into was given");

    assert_eq!(written.id, "subject-dossier");
    let body = std::fs::read_to_string(&written.body).expect("the body was written");
    assert!(body.contains("FR-ORCH-002"), "{body}");
}

#[test]
fn Test_Submit_Should_Refuse_An_Incomplete_Submission_And_Write_Nothing()
{
    let mut request = Complete_Feature_Request("FR-ORCH-003", None);
    request.fields.truncate(1);
    let mut assembly = Assemble(&No_Corpus()).expect("assembles from the embedded records alone");

    let SubmitRefusal::Refused(refusal) =
        Submit(&mut assembly, &request, &StdFileSystem).expect_err("an incomplete submission must be refused")
    else
    {
        panic!("an incomplete submission must refuse SubmitRefusal::Refused");
    };
    assert!(format!("{refusal}").contains("goal"), "{refusal}");
}
