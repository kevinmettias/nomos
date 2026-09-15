//! The exit codes `work`'s own usage text documents, held against the enum this
//! group actually returns -- one file, because the two sides of that comparison
//! are the subject.

use super::super::ExitCode;
use super::super::parse::Usage_Text;

/// Every code this group can leave the process with.
///
/// `work::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::ValidationError,
        ExitCode::Usage,
        ExitCode::ClaimUnavailable,
        ExitCode::Conflict,
        ExitCode::StoreError,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::ValidationError => "ValidationError",
        ExitCode::Usage => "Usage",
        ExitCode::ClaimUnavailable => "ClaimUnavailable",
        ExitCode::Conflict => "Conflict",
        ExitCode::StoreError => "StoreError",
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
/// happens — and this group's codes are the ones agents branch on rather than parsing
/// output, which is `ExitCode`'s own doc's reason for existing.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = Usage_Text();
    let documented = Documented_Codes(&usage);

    assert!(
        usage.starts_with("usage: nomos work"),
        "this compared some other group's help text: {usage}"
    );
    assert!(
        !documented.codes.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {}",
        documented.spelled
    );
    assert_eq!(
        documented.codes,
        Sorted(Every_Exit_Code().iter().map(|code| return code.Value())),
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}

/// The codes the usage text documents, read out of the prose that follows its `exit codes:`
/// heading.
struct DocumentedCodes
{
    /// That prose, so a caller reporting an empty read can say what it read.
    spelled: String,
    /// The codes spelled in it, in ascending order.
    codes: Vec<i32>,
}

/// Reads the prose after the usage text's `exit codes:` heading into [`DocumentedCodes`].
fn Documented_Codes(usage: &str) -> DocumentedCodes
{
    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");

    return DocumentedCodes {
        spelled: spelled.to_owned(),
        codes: Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok())),
    };
}
