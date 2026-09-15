//! The exit codes `agent`'s own usage text documents, held against the enum this group
//! actually returns -- one file, because the two sides of that comparison are the subject.

use super::super::{Command_From_String_Arguments, ExitCode};

/// The numeric codes `agent`'s own usage text documents (see `Usage_Text`'s "exit codes"
/// line): 0 ok, 2 usage, 5 the executor could not run or answer, 6 the named crate has no
/// README row or no committed surface snapshot. Named rather than written into the test
/// below, so that the number a code answers with has one home in this file.
const DOCUMENTED_OK: i32 = 0;
const DOCUMENTED_USAGE: i32 = 2;
const DOCUMENTED_UNAVAILABLE: i32 = 5;
const DOCUMENTED_NOT_FOUND: i32 = 6;

#[test]
fn Test_Value_Should_Return_The_Documented_Exit_Code_Number()
{
    assert_eq!(ExitCode::Ok.Value(), DOCUMENTED_OK);
    assert_eq!(ExitCode::Usage.Value(), DOCUMENTED_USAGE);
    assert_eq!(ExitCode::Unavailable.Value(), DOCUMENTED_UNAVAILABLE);
    assert_eq!(ExitCode::NotFound.Value(), DOCUMENTED_NOT_FOUND);
}

/// Every code this group can leave the process with.
///
/// `agent::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[ExitCode::Ok, ExitCode::Usage, ExitCode::Unavailable, ExitCode::NotFound];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Usage => "Usage",
        ExitCode::Unavailable => "Unavailable",
        ExitCode::NotFound => "NotFound",
    };
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents, read through the refusal a verbless
/// invocation gives rather than out of `USAGE_TEXT`, which is private to `agent::parsing`:
/// the text a user is actually shown is the subject, and reaching it this way needs no
/// widening of what this test can see.
fn Documented_Exit_Codes() -> Vec<i32>
{
    let usage = Command_From_String_Arguments(&[]).expect_err("no verb prints the usage text");

    assert!(
        usage.starts_with("usage: nomos agent"),
        "this compared some other group's help text: {usage}"
    );

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );

    return documented;
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
    assert_eq!(
        Documented_Exit_Codes(),
        Sorted(Every_Exit_Code().iter().map(|code| return code.Value())),
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}
