//! Chains judged branch by branch, nesting, file definitions, and every refusal -- each against a
//! state worked out by hand from the specification's rules.

use crate::{ConditionalReading, Read_Conditionals};
use nomos_cap_csharp_semantics::{Branch, BranchState, ConditionalRegion, DefinitionEffect, FileDefinition};
use std::collections::BTreeSet;

fn Symbols(symbols: &[&str]) -> BTreeSet<String>
{
    return symbols.iter().map(|symbol| return (*symbol).to_owned()).collect();
}

fn Regions(source: &str, symbols: &[&str]) -> Vec<ConditionalRegion>
{
    return match Read_Conditionals(source, &Symbols(symbols))
    {
        ConditionalReading::Read { regions, .. } => regions,
        ConditionalReading::Refused(failure) => panic!("the fixture's directives nest: {failure}"),
    };
}

fn States(source: &str, symbols: &[&str]) -> Vec<BranchState>
{
    return Regions(source, symbols).iter().map(|region| return region.state).collect();
}

fn Refusal(source: &str) -> String
{
    return match Read_Conditionals(source, &BTreeSet::new())
    {
        ConditionalReading::Refused(failure) => failure.to_string(),
        ConditionalReading::Read { regions, .. } => panic!("expected a refusal, read {regions:?}"),
    };
}

const CHAIN: &str = "#if A\nclass One {}\n#elif B\nclass Two {}\n#else\nclass Three {}\n#endif\n";

#[test]
fn Test_A_Chain_Should_Compile_Exactly_Its_First_True_Branch()
{
    use BranchState::{Compiled, Skipped};

    assert_eq!(States(CHAIN, &["A"]), [Compiled, Skipped, Skipped]);
    assert_eq!(States(CHAIN, &["A", "B"]), [Compiled, Skipped, Skipped], "an #elif after a taken branch is skipped");
    assert_eq!(States(CHAIN, &["B"]), [Skipped, Compiled, Skipped]);
    assert_eq!(States(CHAIN, &[]), [Skipped, Skipped, Compiled]);
}

#[test]
fn Test_A_Region_Should_Name_Its_Branch_Lines_And_Condition()
{
    assert_eq!(
        Regions(CHAIN, &["B"]),
        [
            ConditionalRegion { branch: Branch::If, line: 1, end_line: 3, condition: "A".to_owned(), state: BranchState::Skipped },
            ConditionalRegion { branch: Branch::Elif, line: 3, end_line: 5, condition: "B".to_owned(), state: BranchState::Compiled },
            ConditionalRegion { branch: Branch::Else, line: 5, end_line: 7, condition: String::new(), state: BranchState::Skipped },
        ]
    );
}

/// A chain inside a skipped branch is never evaluated by the compiler: every branch of it is
/// skipped, whatever its condition says.
#[test]
fn Test_A_Chain_Nested_In_A_Skipped_Branch_Should_Be_Skipped_Throughout()
{
    use BranchState::{Compiled, Skipped};
    let source = "#if A\n#if true\nx\n#else\ny\n#endif\n#else\n#if true\nz\n#endif\n#endif\n";

    assert_eq!(States(source, &[]), [Skipped, Skipped, Skipped, Compiled, Compiled]);
}

#[test]
fn Test_A_File_Definition_Should_Change_The_Set_From_Its_Line_On()
{
    let source = "#define LOCAL\n#undef DEBUG\n#if LOCAL && !DEBUG\nclass A {}\n#endif\n";

    let ConditionalReading::Read { definitions, regions } = Read_Conditionals(source, &Symbols(&["DEBUG"]))
    else
    {
        panic!("the fixture's directives nest");
    };

    assert_eq!(
        definitions,
        [
            FileDefinition { line: 1, symbol: "LOCAL".to_owned(), effect: DefinitionEffect::Define },
            FileDefinition { line: 2, symbol: "DEBUG".to_owned(), effect: DefinitionEffect::Undefine },
        ]
    );
    assert_eq!(regions.iter().map(|region| return region.state).collect::<Vec<_>>(), [BranchState::Compiled]);
}

/// A `#define` in a skipped branch is never read, so it defines nothing.
#[test]
fn Test_A_Definition_In_A_Skipped_Branch_Should_Define_Nothing()
{
    let source = "#if false\n#define LOCAL\n#endif\n#if LOCAL\nx\n#endif\n";

    assert_eq!(States(source, &[]), [BranchState::Skipped, BranchState::Skipped]);
}

/// A malformed condition is unevaluated, and so is every later branch of its chain and every
/// branch nested in it; a symbol it may have defined leaves later conditions reading it unknown.
#[test]
fn Test_An_Unevaluable_Condition_Should_Leave_Its_Chain_Unevaluated_And_Guess_Nothing()
{
    use BranchState::{Compiled, Unevaluated};
    let source = "#if (A\n#define LOCAL\n#if B\nx\n#endif\n#else\ny\n#endif\n#if LOCAL\nz\n#endif\n#if true\nw\n#endif\n";

    assert_eq!(States(source, &[]), [Unevaluated, Unevaluated, Unevaluated, Unevaluated, Compiled]);
}

/// Directive-shaped lines inside a verbatim string, a raw string and a delimited comment are text,
/// and one inside a skipped branch is a directive whatever surrounds it.
#[test]
fn Test_A_Directive_Should_Be_Recognized_Only_Where_The_Compiler_Recognizes_One()
{
    let source = "var v = @\"\n#if VERBATIM\n\";\nvar r = \"\"\"\n#if RAW\n\"\"\";\n/*\n#if COMMENT\n*/\n#if false\nstring s = \"\n#endif\n";

    let regions = Regions(source, &[]);

    assert_eq!(regions.len(), 1, "{regions:?}");
    assert_eq!(regions.first().map(|region| return (region.line, region.end_line)), Some((10, 12)));
}

#[test]
fn Test_A_Chain_The_Compiler_Refuses_Should_Be_Refused_For_Its_Own_Reason()
{
    let cases: [(&str, &str); 6] = [
        ("#endif\n", "an #endif with no #if open"),
        ("#else\n#endif\n", "an #else with no #if open"),
        ("#if A\n#else\n#elif B\n#endif\n", "an #elif after its chain's #else"),
        ("#if A\n", "an #if with no #endif"),
        ("class A {}\n#define LATE\n", "#define after the first token"),
        ("#define\n", "#define names no symbol"),
    ];

    for (source, reason) in cases
    {
        let refusal = Refusal(source);
        assert!(refusal.contains(reason), "{source:?}: {refusal}");
    }
}

#[test]
fn Test_A_File_With_No_Directive_Should_Read_As_No_Region()
{
    assert_eq!(Read_Conditionals("class A {}\n", &BTreeSet::new()), ConditionalReading::Read { definitions: Vec::new(), regions: Vec::new() });
}
