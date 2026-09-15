//! The codes `gate` leaves the process with: the census of them, the position each holds in
//! it, and the list the usage text prints -- one file, because the two sides of that
//! comparison are the subject.

use super::super::exit_code::Every_Exit_Code;
use super::super::parsing::USAGE;
use super::super::ExitCode;

/// The position each [`ExitCode`] holds in [`Every_Exit_Code`], one name per code, so the
/// exhaustive mirror below claims a named position rather than leaving three numerals to be
/// read against the census by eye.
const OK_ORDINAL: usize = 0;
const VIOLATIONS_ORDINAL: usize = 1;
const USAGE_ORDINAL: usize = 2;
const CONTRADICTORY_ORDINAL: usize = 3;
const VACUOUS_ORDINAL: usize = 4;

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Violations => "Violations",
        ExitCode::Usage => "Usage",
        ExitCode::Contradictory => "Contradictory",
        ExitCode::Vacuous => "Vacuous",
    };
}

/// `Every_Exit_Code`'s own mirror, named in its doc comment.
///
/// The match has no wildcard arm. A variant added to [`ExitCode`] without a matching arm
/// added here fails this file to *compile*, not merely to pass.
#[test]
fn Test_Every_Exit_Code_Should_Be_Matched_Exhaustively()
{
    fn Ordinal(code: ExitCode) -> usize
    {
        return match code
        {
            ExitCode::Ok => OK_ORDINAL,
            ExitCode::Violations => VIOLATIONS_ORDINAL,
            ExitCode::Usage => USAGE_ORDINAL,
            ExitCode::Contradictory => CONTRADICTORY_ORDINAL,
            ExitCode::Vacuous => VACUOUS_ORDINAL,
        };
    }

    for (index, code) in Every_Exit_Code().iter().enumerate()
    {
        assert_eq!(
            Ordinal(*code),
            index,
            "{} is not matched at the position Every_Exit_Code() puts it, so the exhaustive \
             match and the census have drifted apart",
            Labelled(*code)
        );
    }
}

/// Zero is the only success this group leaves the process with -- `ExitCode::Value` returns
/// it for `Ok` and only for `Ok`.
#[test]
fn Test_Value_Should_Be_Zero_If_And_Only_If_The_Code_Is_Ok()
{
    for code in Every_Exit_Code().iter().copied()
    {
        assert_eq!(
            code.Value() == 0,
            code == ExitCode::Ok,
            "{} exits {}, and zero is read as success",
            Labelled(code),
            code.Value()
        );
    }
}

/// The codes this file documents are the codes this group can exit with.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let (_, spelled) = USAGE
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented, implemented,
        "the usage text and ExitCode disagree about what this command can exit with"
    );
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}
