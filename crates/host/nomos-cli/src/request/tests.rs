//! What `nomos request` promises, exercised.

use super::*;
use nomos_spec_model::{Severity, SubmissionKind, SubmissionState};

#[test]
fn Test_Command_From_String_Arguments_Should_Parse_Its_Fields_And_Default_State_And_Version()
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

/// Every `--field` command line this crate refuses as a usage error -- a named provider so
/// another malformed `--field` scenario is an entry here, not a second copy of the test
/// below.
fn Malformed_Field_Command_Lines() -> Vec<&'static str>
{
    return vec!["submit --kind feature-request --id FR-101 --by kevin --field oops"];
}

#[test]
fn Test_A_Field_With_No_Equals_Should_Be_A_Usage_Error()
{
    for text in Malformed_Field_Command_Lines()
    {
        let arguments = Arguments_From_Text(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--field"), "{error}");
    }
}

/// Every `--kind` command line this crate refuses as a usage error -- a named provider so
/// another unrecognised `--kind` scenario is an entry here, not a second copy of the test
/// below.
fn Unrecognised_Kind_Command_Lines() -> Vec<&'static str>
{
    return vec!["submit --kind nonsense --id FR-102 --by kevin"];
}

#[test]
fn Test_Usage_Text_Should_Be_Appended_To_An_Unrecognised_Kinds_Refusal()
{
    for text in Unrecognised_Kind_Command_Lines()
    {
        let arguments = Arguments_From_Text(text);

        let error = Command_From_String_Arguments(&arguments).expect_err("must refuse");

        assert!(error.contains("--kind"), "{error}");
    }
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

/// Every code this group can leave the process with.
///
/// `request::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::Usage,
        ExitCode::StoreError,
        ExitCode::Unwritable,
        ExitCode::Refused,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Usage => "Usage",
        ExitCode::StoreError => "StoreError",
        ExitCode::Unwritable => "Unwritable",
        ExitCode::Refused => "Refused",
    };
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this
/// group's own enum. Eight groups print an exit-code list and only those two mirrored it;
/// the other six were correct rather than guarded, which is a different thing, and
/// `OD-AGENT-004`'s amendment says a printed vocabulary is admissible only where a test
/// compares it against its authority. The usage text is prose a person reads and
/// [`ExitCode`] is what the process returns, the two were written separately, and a code
/// added or renumbered in one of them and not the other is the failure that actually
/// happens.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = super::parsing::Usage_Text();

    assert!(
        usage.starts_with("usage: nomos request"),
        "this compared some other group's help text: {usage}"
    );

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}
