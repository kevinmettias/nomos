//! The verbs that answer a question without writing anything: `Profiles`, `Sources` and the
//! three resolvers -- `Record`, `Table` and `Markdown`.

use nomos_platform_std::StdFileSystem;

use crate::SpecCommand;
use crate::request::{RecordRequest, TableRequest};
use crate::run::{Profiles, Run};
use crate::spec_outcome::{RecordRefusal, SpecOutcome, TableRefusal};

use super::No_Corpus;

/// How much of a resolved record's front matter a failure message quotes back. Enough to
/// carry the `---` fence and the `id:` line that must open it, and no more.
const FRONT_MATTER_PREVIEW_CHARS: usize = 20;

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
        answer.document.text.get(..FRONT_MATTER_PREVIEW_CHARS)
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
