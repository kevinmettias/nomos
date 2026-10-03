//! The sites offer through the public API a consumer has: the corpus's own cases for the labeled
//! jump, ported site for site, and the offer checked against the family's contract.
//!
//! Every case of `language-kernels/programming/rust/rustlang/rust_labeled_loop_test.go` at
//! code-standards `f0d820729` is here with its source unchanged, asserting what that test asserts
//! about the projected jump and, because a site is more than the two fields that test checks, the
//! rest of it too: the line, the label's line and the text. What those tests then ask of
//! `labeledloop.Noise_Labels` is the rule's judgment rather than this provider's, and is ported
//! against the rule in `nomos-rules`, over the sites this file pins.

use nomos_cap_syntax::LabeledJump;
use nomos_lang_rust::sites::{Declared_Guarantee, Provider_Offer, Read_Sites, SitesReading};

/// Every jump `source` holds, or a failed test if it did not parse.
fn Jumps_In(source: &str) -> Vec<LabeledJump>
{
    return match Read_Sites(source)
    {
        SitesReading::Parsed(jumps) => jumps,
        SitesReading::Unparseable(failure) => panic!("the case is the corpus's own parseable Rust: {failure}"),
    };
}

fn Jump(line: usize, keyword: &str, label: &str, label_line: usize, has_same_target_unlabeled: bool) -> LabeledJump
{
    return LabeledJump {
        line,
        keyword: keyword.to_owned(),
        label: label.to_owned(),
        text: format!("{keyword} {label}"),
        label_line,
        has_same_target_unlabeled,
        is_inside_break_capture: false,
    };
}

/// `Test_Rust_A_Break_Across_Two_Loops_Is_Load_Bearing`: the doubly-nested search. A bare `break`
/// leaves the inner loop, so only the label reaches the outer one.
#[test]
fn Test_A_Break_Across_Two_Loops_Should_Not_Land_Where_A_Bare_Break_Would()
{
    let found = Jumps_In(
        "
fn search(grid: &[Vec<i32>], needle: i32) -> bool {
    'outer: for row in grid {
        for cell in row {
            if *cell == needle {
                break 'outer;
            }
        }
    }
    false
}
",
    );

    assert_eq!(found, vec![Jump(6, "break", "'outer", 3, false)]);
}

/// `Test_Rust_A_Label_On_The_Loop_The_Jump_Is_In_Is_Redundant`: a `match` arm does not catch a
/// bare `break` in Rust, so the label adds nothing, and Rust has no break-capturing construct to
/// excuse it. The bare `continue` beside it names nothing and is no site.
#[test]
fn Test_A_Jump_Out_Of_A_Match_Arm_Should_Land_Where_A_Bare_Break_Would()
{
    let found = Jumps_In(
        "
fn drain(kinds: &[i32]) {
    'loop_kinds: for kind in kinds {
        match kind {
            0 => break 'loop_kinds,
            _ => continue,
        }
    }
}
",
    );

    assert_eq!(found, vec![Jump(5, "break", "'loop_kinds", 3, true)]);
}

/// `Test_Rust_A_Redundant_Continue_Is_Noise`: the same in the other keyword, which is carried as
/// the language spells it.
#[test]
fn Test_A_Continue_Naming_Its_Own_Loop_Should_Land_Where_A_Bare_Continue_Would()
{
    let found = Jumps_In(
        "
fn skip(values: &[i32]) {
    'each: for value in values {
        if *value < 0 {
            continue 'each;
        }
    }
}
",
    );

    assert_eq!(found, vec![Jump(5, "continue", "'each", 3, true)]);
}

/// `Test_Rust_A_Break_Out_Of_A_Labelled_Block_Is_Not_A_Loop_Jump`: a labeled block abandons no
/// iteration. The walk still tracks it, or the jump would resolve against an outer loop of the same
/// name.
#[test]
fn Test_A_Break_Out_Of_A_Labelled_Block_Should_Be_No_Site()
{
    let found = Jumps_In(
        "
fn classify(kind: i32) -> i32 {
    'blk: {
        if kind == 0 {
            break 'blk;
        }
    }
    kind
}
",
    );

    assert_eq!(found, Vec::new());
}

/// `Test_Rust_The_Extracted_Function_Is_Clean`: no labels, no jumps.
#[test]
fn Test_The_Extracted_Function_Should_Hold_No_Site()
{
    let found = Jumps_In(
        "
fn has_match(row: &[i32], needle: i32) -> bool {
    for cell in row {
        if *cell == needle {
            return true;
        }
    }
    false
}

fn search(grid: &[Vec<i32>], needle: i32) -> bool {
    for row in grid {
        if has_match(row, needle) {
            return true;
        }
    }
    false
}
",
    );

    assert_eq!(found, Vec::new());
}

/// `Test_Rust_An_Unparseable_File_Has_No_Labeled_Jumps`: a file that did not parse was not judged,
/// which is a different answer from a file with no jumps.
#[test]
fn Test_An_Unparseable_File_Should_Be_Unparseable_Rather_Than_Empty()
{
    let reading = Read_Sites("fn f( { 'a: loop { break 'a");

    assert!(matches!(reading, SitesReading::Unparseable(_)), "{reading:?}");
}

/// Beyond the corpus's cases: a labeled block between a jump and its loop is not a loop, so it does
/// not catch a bare `break` -- the jump still lands where a bare one would.
#[test]
fn Test_A_Labelled_Block_Inside_The_Loop_Should_Not_Count_As_The_Innermost_Loop()
{
    let found = Jumps_In("fn f() {\n    'outer: loop {\n        'blk: {\n            break 'outer;\n        }\n    }\n}\n");

    assert_eq!(found, vec![Jump(4, "break", "'outer", 2, true)]);
}

/// Beyond the corpus's cases: a label does not cross a closure or an item. Inside a closure the
/// outer label resolves to nothing, which does not compile and is no site; inside a nested function
/// the jump resolves to that function's own loop.
#[test]
fn Test_A_Label_Should_Not_Resolve_Across_A_Closure_Or_A_Nested_Function()
{
    let found = Jumps_In(
        "fn f() {\n    'a: while true {\n        let g = || loop { break 'a; };\n        fn h() {\n            'a: loop {\n                break 'a;\n            }\n        }\n        break 'a;\n    }\n}\n",
    );

    assert_eq!(found, vec![Jump(6, "break", "'a", 5, true), Jump(9, "break", "'a", 2, true)]);
}

/// What the declared guarantee claims as sound, exercised: every reported jump is written on the
/// line it is reported at, keyword and label both, and its label is written on the line reported
/// for it -- so the walk reports nothing the file does not contain.
#[test]
fn Test_Soundness_Should_Hold_Every_Reported_Jump_Is_Written_On_Its_Line()
{
    let source = "fn f(grid: &[Vec<i32>]) {\n    'rows: for row in grid {\n        'cells: for cell in row {\n            if *cell < 0 { continue 'rows; }\n            if *cell == 0 { break 'cells; }\n        }\n    }\n}\n";
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

/// An offer claiming resolution is refused: the family's ceiling binds this provider too.
#[test]
fn Test_A_Sites_Offer_Claiming_Resolution_Should_Be_Refused()
{
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    let mut registry = nomos_capability::Registry::New();
    registry.Declare(nomos_cap_syntax::Sites_Capability_Contract()).expect("the contract is the first declaration in a fresh registry");
    let resolving = nomos_capability::ProviderOffer {
        guarantee: Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
        ..Provider_Offer()
    };

    assert!(registry.Offer(resolving).is_err());
}
