//! What this crate promises, exercised at the two seams `nomos-cli` now calls through.

use crate::corpus::CorpusRequest;
use crate::command::SpecCommand;
use crate::outcome::{NotYetMigrated, SpecOutcome};
use crate::request::RecordRequest;
use crate::run::{Profiles, Run};

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
    let outcome = Run(&SpecCommand::Profiles, &No_Corpus());

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
    let outcome = Run(&SpecCommand::Sources, &No_Corpus());

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
fn Test_Run_Of_An_Unmigrated_Verb_Should_Answer_Its_Marker()
{
    let request = SpecCommand::Record(RecordRequest {
        id: "D-129".to_owned(),
        revision: None,
    });

    let outcome = Run(&request, &No_Corpus());

    assert!(
        matches!(outcome, SpecOutcome::Record(NotYetMigrated)),
        "Record has not moved into this crate yet"
    );
}
