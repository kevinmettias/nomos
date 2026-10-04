//! The Go sites offer through the public API a consumer has: the corpus's own cases for the labeled
//! jump, ported site for site, and the offer checked against the family's contract.
//!
//! Every case of `language-kernels/programming/go/golang/go_labeled_loop_test.go` at code-standards
//! `f0d820729` is here with its source, line for line -- indented with spaces rather than tabs,
//! which Go's grammar does not distinguish -- asserting the whole site: line, keyword, label, the
//! label's line and both truth values, where the corpus test checks some of them. What those tests
//! then ask of `labeledloop.Noise_Labels` is the rule's judgment, and the rule's own tests in
//! `nomos-rules` hold the corpus engine's cases for exactly those shapes: a jump standing inside a
//! break capture keeps its label, one load-bearing jump keeps the name for the others, and two labels
//! sharing a name are two labels.

use nomos_cap_syntax::LabeledJump;
use nomos_lang_go::sites::{Declared_Guarantee, Provider_Offer, Read_Sites, SitesReading};

/// Every jump `source` holds, or a failed test if it did not parse.
fn Jumps_In(source: &str) -> Vec<LabeledJump>
{
    return match Read_Sites(source)
    {
        SitesReading::Parsed(jumps) => jumps,
        SitesReading::Unparseable(failure) => panic!("the case is the corpus's own parseable Go: {failure}"),
    };
}

/// One expected site: where it is, what it names, where its label is written, and the two facts.
struct Expected<'a>
{
    line: usize,
    keyword: &'a str,
    label: &'a str,
    label_line: usize,
    same_target: bool,
    inside_capture: bool,
}

fn Jump(expected: Expected<'_>) -> LabeledJump
{
    return LabeledJump {
        line: expected.line,
        keyword: expected.keyword.to_owned(),
        label: expected.label.to_owned(),
        text: format!("{} {}", expected.keyword, expected.label),
        label_line: expected.label_line,
        has_same_target_unlabeled: expected.same_target,
        is_inside_break_capture: expected.inside_capture,
    };
}

/// The doubly-nested search, which `Test_Go_A_Break_Across_Two_Loops_Is_Load_Bearing` and
/// `Test_record_Records_A_Labeled_Jump` both read.
const TWO_LOOPS: &str = "package p

func search(grid [][]int, needle int) bool {
Outer:
    for _, row := range grid {
        for _, cell := range row {
            if cell == needle {
                break Outer
            }
        }
    }
    return false
}
";

/// `Test_Go_A_Break_Across_Two_Loops_Is_Load_Bearing`: a bare `break` leaves the inner loop.
#[test]
fn Test_A_Break_Across_Two_Loops_Should_Not_Land_Where_A_Bare_Break_Would()
{
    let expected = Expected { line: 8, keyword: "break", label: "Outer", label_line: 4, same_target: false, inside_capture: false };

    assert_eq!(Jumps_In(TWO_LOOPS), vec![Jump(expected)]);
}

/// `Test_record_Records_A_Labeled_Jump`: the labeled break is recorded, targeting `Outer`.
#[test]
fn Test_Record_Should_Record_A_Labeled_Break()
{
    let found = Jumps_In(TWO_LOOPS);

    assert_eq!(found.iter().map(|jump| return jump.label.as_str()).collect::<Vec<_>>(), vec!["Outer"]);
}

/// `Test_Go_A_Label_On_The_Loop_The_Jump_Is_In_Is_Redundant`: `break` alone does exactly this.
#[test]
fn Test_A_Break_Naming_Its_Own_Loop_Should_Land_Where_A_Bare_Break_Would()
{
    let found = Jumps_In(
        "package p

func first(values []int) int {
Loop:
    for _, value := range values {
        if value > 0 {
            break Loop
        }
    }
    return 0
}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 7, keyword: "break", label: "Loop", label_line: 4, same_target: true, inside_capture: false })]);
}

/// `Test_Go_A_Break_Out_Of_A_Switch_Inside_A_Loop_Is_Load_Bearing`: Go's bare `break` is caught by
/// the `switch`, so `break Loop` is the only way out of the loop. The bare `continue` beside it names
/// nothing and is no site.
#[test]
fn Test_A_Break_Out_Of_A_Switch_Should_Not_Land_Where_A_Bare_Break_Would()
{
    let found = Jumps_In(
        "package p

func drain(kinds []int) {
Loop:
    for _, kind := range kinds {
        switch kind {
        case 0:
            break Loop
        default:
            continue
        }
    }
}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 8, keyword: "break", label: "Loop", label_line: 4, same_target: false, inside_capture: true })]);
}

/// `Test_Go_A_Continue_Out_Of_A_Switch_Names_Its_Loop_Deliberately`: a bare `continue` passes
/// through a switch to the loop, and the jump stands inside the switch.
#[test]
fn Test_A_Continue_Out_Of_A_Switch_Should_Reach_The_Loop_From_Inside_A_Break_Capture()
{
    let found = Jumps_In(
        "package p

func drain(kinds []int) {
Loop:
    for _, kind := range kinds {
        switch kind {
        case 0:
            continue Loop
        }
    }
}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 8, keyword: "continue", label: "Loop", label_line: 4, same_target: true, inside_capture: true })]);
}

/// `Test_Go_A_Break_To_A_Labeled_Switch_Is_Not_A_Loop_Jump`: a label on a `switch` is not a label on
/// a loop.
#[test]
fn Test_A_Break_To_A_Labeled_Switch_Should_Be_No_Site()
{
    let found = Jumps_In(
        "package p

func classify(kind int) {
Done:
    switch kind {
    case 0:
        break Done
    }
}
",
    );

    assert_eq!(found, Vec::new());
}

/// `Test_Go_The_Extracted_Function_Is_Clean`: no labels, no jumps.
#[test]
fn Test_The_Extracted_Function_Should_Hold_No_Site()
{
    let found = Jumps_In(
        "package p

func has_Match(row []int, needle int) bool {
    for _, cell := range row {
        if cell == needle {
            return true
        }
    }
    return false
}

func search(grid [][]int, needle int) bool {
    for _, row := range grid {
        if has_Match(row, needle) {
            return true
        }
    }
    return false
}
",
    );

    assert_eq!(found, Vec::new());
}

/// `Test_Go_An_Unparseable_File_Has_No_Labeled_Jumps`: a file that did not parse was not judged.
#[test]
fn Test_An_Unparseable_File_Should_Be_Unparseable_Rather_Than_Empty()
{
    let reading = Read_Sites("package p\nfunc f( {\n");

    assert!(matches!(reading, SitesReading::Unparseable(_)), "{reading:?}");
}

/// `Test_A_Labeled_Loop_In_A_Package_Level_Func_Literal_Is_Still_Found`: a func literal in a `var`
/// declaration is code, and a walk that never opened it would report a clean file.
#[test]
fn Test_A_Labeled_Loop_In_A_Package_Level_Func_Literal_Should_Be_Found()
{
    let found = Jumps_In(
        "package p

var search = func(grid [][]int, needle int) bool {
Outer:
    for _, row := range grid {
        for _, cell := range row {
            if cell == needle {
                break Outer
            }
        }
    }
    return false
}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 8, keyword: "break", label: "Outer", label_line: 4, same_target: false, inside_capture: false })]);
}

/// `Test_A_Labeled_Loop_In_A_Composite_Literal_Is_Still_Found`: the other place Go hides a body.
#[test]
fn Test_A_Labeled_Loop_In_A_Composite_Literal_Should_Be_Found()
{
    let found = Jumps_In(
        "package p

type Handler struct{ Scan func([][]int) }

var handler = Handler{Scan: func(grid [][]int) {
Outer:
    for _, row := range grid {
        for _, cell := range row {
            if cell == 0 {
                break Outer
            }
        }
    }
}}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 10, keyword: "break", label: "Outer", label_line: 6, same_target: false, inside_capture: false })]);
}

/// `Test_A_Closure_Still_Starts_A_Fresh_Label_Stack`: the jump inside the closure resolves against
/// the closure's own loops, never the enclosing ones.
#[test]
fn Test_A_Closure_Should_Start_A_Fresh_Label_Stack()
{
    let found = Jumps_In(
        "package p

var run = func(grid [][]int) {
Outer:
    for range grid {
        go func() {
        Inner:
            for range grid {
                for range grid {
                    break Inner
                }
            }
        }()
    }
}
",
    );

    assert_eq!(found, vec![Jump(Expected { line: 10, keyword: "break", label: "Inner", label_line: 7, same_target: false, inside_capture: false })]);
}

/// `Test_Go_A_Loop_With_One_Load_Bearing_Jump_Keeps_Its_Label`: both jumps are projected, naming the
/// same label by the same line, one where a bare `break` would do and one inside a `switch`.
#[test]
fn Test_A_Loop_With_One_Load_Bearing_Jump_Should_Project_Both_Under_One_Label()
{
    let found = Jumps_In(
        "package p

func drain(kinds []int) {
Loop:
    for _, kind := range kinds {
        if kind < 0 {
            break Loop
        }
        switch kind {
        case 0:
            break Loop
        }
    }
}
",
    );

    assert_eq!(
        found,
        vec![
            Jump(Expected { line: 7, keyword: "break", label: "Loop", label_line: 4, same_target: true, inside_capture: false }),
            Jump(Expected { line: 11, keyword: "break", label: "Loop", label_line: 4, same_target: false, inside_capture: true }),
        ]
    );
}

/// `Test_Go_The_Same_Label_Name_In_Two_Functions_Is_Two_Labels`: a label's identity is the line it
/// is written on -- caseValues' `L` on line 4, caseTypes' on line 15.
#[test]
fn Test_The_Same_Label_Name_In_Two_Functions_Should_Carry_Two_Label_Lines()
{
    let found = Jumps_In(
        "package p

func caseValues(values [][]int) {
L:
    for _, value := range values {
        for _, inner := range value {
            if inner == 0 {
                continue L
            }
        }
    }
}

func caseTypes(types []int) {
L:
    for _, kind := range types {
        if kind == 0 {
            continue L
        }
    }
}
",
    );

    assert_eq!(
        found,
        vec![
            Jump(Expected { line: 8, keyword: "continue", label: "L", label_line: 4, same_target: false, inside_capture: false }),
            Jump(Expected { line: 18, keyword: "continue", label: "L", label_line: 15, same_target: true, inside_capture: false }),
        ]
    );
}

/// What the declared guarantee claims as sound, exercised: every reported jump is written on the line
/// it is reported at, and its label on the line reported for it.
#[test]
fn Test_Soundness_Should_Hold_Every_Reported_Jump_Is_Written_On_Its_Line()
{
    let source = "package p\n\nfunc f(grid [][]int) {\nRows:\n    for _, row := range grid {\n        switch len(row) {\n        case 0:\n            continue Rows\n        }\n        break Rows\n    }\n}\n";
    let lines: Vec<&str> = source.lines().collect();

    let found = Jumps_In(source);

    assert_eq!(found.len(), 2, "{found:?}");
    for jump in found
    {
        let written = lines.get(jump.line.saturating_sub(1)).copied().unwrap_or_default();
        assert!(written.contains(&jump.text), "line {} does not hold `{}`: {written}", jump.line, jump.text);
        let labelled = lines.get(jump.label_line.saturating_sub(1)).copied().unwrap_or_default();
        assert!(labelled.contains(&format!("{}:", jump.label)), "line {} does not label `{}`: {labelled}", jump.label_line, jump.label);
    }
}

#[test]
fn Test_The_Sites_Offer_Should_Be_Accepted_Under_The_Familys_Contract()
{
    let mut registry = nomos_capability::Registry::New();

    let accepted = registry.Declare_And_Offer(nomos_cap_syntax::Sites_Capability_Contract(), Provider_Offer());

    assert_eq!(accepted, Ok(()));
    assert!(nomos_cap_syntax::Sites_Ceiling().Satisfies(&Declared_Guarantee()));
}
