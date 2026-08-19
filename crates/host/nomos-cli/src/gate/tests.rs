//! What this module promises, exercised.

use super::*;
use super::exit_code::Every_Exit_Code;
use super::parsing::USAGE;

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::Usage => "Usage",
        ExitCode::Contradictory => "Contradictory",
    };
}

/// `Every_Exit_Code`'s own mirror, named in its doc comment.
///
/// The match has no wildcard arm. A variant added to [`ExitCode`] without a matching arm
/// added here fails this file to *compile*, not merely to pass.
#[test]
fn Test_Every_ExitCode_Should_Be_Matched_Exhaustively()
{
    fn Ordinal(code: ExitCode) -> usize
    {
        return match code
        {
            ExitCode::Ok => 0,
            ExitCode::Usage => 1,
            ExitCode::Contradictory => 2,
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

/// Zero is the only success this group leaves the process with.
#[test]
fn Test_Only_Ok_Should_Carry_The_Passing_Exit_Code()
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

#[test]
fn Test_A_Root_Should_Default_To_Here()
{
    assert_eq!(
        Parse(&["plan".to_owned()]).expect("plan with no root is valid").root,
        PathBuf::from(".")
    );
}

#[test]
fn Test_A_Given_Root_Should_Win()
{
    let arguments = vec!["plan".to_owned(), "--root".to_owned(), "somewhere".to_owned()];

    assert_eq!(
        Parse(&arguments).expect("plan --root is valid").root,
        PathBuf::from("somewhere")
    );
}

/// No verb at all must not be silently read as `plan`.
#[test]
fn Test_No_Verb_Should_Refuse()
{
    let error = Parse(&[]).expect_err("must refuse");

    assert!(error.contains("usage"), "{error}");
}

/// `run`, `explain` and `compare` are named by `ARC-ROADMAP-001` but have no real
/// implementation yet, so this must refuse rather than quietly running `plan` instead.
#[test]
fn Test_An_Unimplemented_Verb_Should_Refuse()
{
    let error = Parse(&["run".to_owned()]).expect_err("must refuse");

    assert!(error.contains("run"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A mistyped flag must not be silently ignored into a default.
#[test]
fn Test_An_Unknown_Flag_Should_Refuse()
{
    let arguments = vec!["plan".to_owned(), "--rooot".to_owned(), "x".to_owned()];

    let error = Parse(&arguments).expect_err("must refuse");

    assert!(error.contains("--rooot"), "{error}");
    assert!(error.contains("usage"), "{error}");
}

/// A real run over this workspace's own three shipped rules reports all three, and exits
/// clean.
///
/// End to end, the way the shipped binary is actually called -- `nomos_gate_orchestration
/// ::Registered` composes the same three real offers `P13-GATE-ORCHESTRATION-1`'s own crate
/// test already checks; this is the assertion that the CLI seam renders what came back
/// rather than trusting the crate boundary silently.
#[test]
fn Test_A_Real_Plan_Should_Report_All_Three_Shipped_Rules()
{
    let command = GateCommand { root: PathBuf::from(".") };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);
    let rendered = String::from_utf8_lossy(&stdout).into_owned();

    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains("rules: 3"), "{rendered}");
    assert!(rendered.contains("completeness-mirror"), "{rendered}");
    assert!(rendered.contains("dependency-direction"), "{rendered}");
    assert!(rendered.contains("function-naming-convention"), "{rendered}");
    assert!(String::from_utf8_lossy(&stderr).is_empty());
}
