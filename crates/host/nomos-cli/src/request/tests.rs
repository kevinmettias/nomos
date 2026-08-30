//! What `nomos request` promises, exercised.

use super::*;
use nomos_spec_model::{Severity, SubmissionKind, SubmissionState};

#[test]
fn Test_A_Submit_Command_Should_Parse_Its_Fields_And_Default_State_And_Version()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-100 --by kevin \
         --field title=t --field goal=g",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments).expect("parses");

    assert_eq!(request.kind, SubmissionKind::FeatureRequest);
    assert_eq!(request.id, "FR-100");
    assert_eq!(request.by, "kevin");
    assert_eq!(request.state, SubmissionState::Draft);
    assert_eq!(request.contract_version, 1);
    assert_eq!(
        request.fields,
        vec![("title".to_owned(), "t".to_owned()), ("goal".to_owned(), "g".to_owned())]
    );
}

#[test]
fn Test_A_Field_With_No_Equals_Should_Be_A_Usage_Error()
{
    let arguments =
        Arguments_From_Text("submit --kind feature-request --id FR-101 --by kevin --field oops");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--field"), "{error}");
}

#[test]
fn Test_An_Unrecognised_Kind_Should_Be_A_Usage_Error()
{
    let arguments = Arguments_From_Text("submit --kind nonsense --id FR-102 --by kevin");

    let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

    assert!(error.contains("--kind"), "{error}");
}

#[test]
fn Test_A_Gap_Should_Parse_Its_Blocked_Fields_And_Severity()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-103 --by kevin \
         --gap which-substrate|behaviour,goal|blocking",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments).expect("parses");

    assert_eq!(request.gaps.len(), 1);
    let gap = request.gaps.first().expect("one gap");
    assert_eq!(gap.question, "which-substrate");
    assert_eq!(gap.blocks, vec!["behaviour".to_owned(), "goal".to_owned()]);
    assert_eq!(gap.severity, Severity::Blocking);
    assert!(gap.closed_by.is_none());
}

#[test]
fn Test_A_Parsed_Submission_Should_Carry_This_Transport_Name()
{
    let arguments = Arguments_From_Text(
        "submit --kind feature-request --id FR-104 --by kevin --field title=t",
    );

    let Command::Submit(request) = Command_From_String_Arguments(&arguments).expect("parses");

    assert_eq!(request.submitted_through, "cli");
}

fn Arguments_From_Text(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}
