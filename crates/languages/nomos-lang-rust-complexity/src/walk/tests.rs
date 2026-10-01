//! One test per decision form, each against a hand count, and the scoping rules that decide
//! whose count a branch lands in.

use crate::{ComplexityReading, Read_Complexity};
use nomos_cap_complexity::FunctionComplexity;

fn Functions(source: &str) -> Vec<FunctionComplexity>
{
    return match Read_Complexity(source)
    {
        ComplexityReading::Parsed(functions) => functions,
        // Every fixture here is Rust written to parse, so a refusal is a broken fixture rather
        // than a reading the assertions below could compare against anything.
        ComplexityReading::Unparseable(failure) => panic!("fixture must parse: {failure}"),
    };
}

fn Complexity_Of(source: &str, function: &str) -> usize
{
    return Functions(source)
        .into_iter()
        .find(|found| return found.function == function)
        .map_or_else(|| panic!("{function} is not in {source}"), |found| return found.complexity);
}

#[test]
fn Test_A_Function_With_No_Branch_Should_Have_Complexity_One()
{
    assert_eq!(Complexity_Of("fn One() { let x = 1; drop(x); }", "One"), 1);
}

#[test]
fn Test_Each_Decision_Form_Should_Add_What_Its_Paths_Add()
{
    let cases: [(&str, usize); 9] = [
        ("fn F(a: bool) { if a {} }", 2),
        ("fn F(a: bool) { if a {} else if !a {} else {} }", 3),
        ("fn F(a: bool) { while a {} }", 2),
        ("fn F(v: Vec<u8>) { for _ in v {} }", 2),
        ("fn F(a: u8) { match a { 0 => {}, 1 => {}, _ => {} } }", 3),
        ("fn F(a: u8) { match a { n if n > 3 => {}, _ => {} } }", 3),
        ("fn F(a: bool, b: bool) -> bool { a && b || a }", 3),
        ("fn F(a: Option<u8>) -> Option<u8> { let b = a?; Some(b) }", 2),
        ("fn F(a: Option<u8>) { let Some(_b) = a else { return; }; }", 2),
    ];

    for (source, expected) in cases
    {
        assert_eq!(Complexity_Of(source, "F"), expected, "{source}");
    }
}

/// A `loop` has one way in, and a `break` under a condition is already counted by that condition.
#[test]
fn Test_A_Loop_Should_Add_Nothing_Of_Its_Own()
{
    assert_eq!(Complexity_Of("fn F(a: bool) { loop { if a { break; } } }", "F"), 2);
}

#[test]
fn Test_A_Closure_Should_Count_Into_The_Function_That_Contains_It()
{
    assert_eq!(Complexity_Of("fn F(v: Vec<u8>) { let _ = v.iter().filter(|x| if **x > 1 { true } else { false }); }", "F"), 2);
}

#[test]
fn Test_A_Nested_Function_Should_Be_Its_Own_Subject_And_Not_Its_Parents()
{
    let functions = Functions("fn Outer(a: bool) { fn Inner(b: bool) { if b {} if b {} } if a {} }");

    assert_eq!(
        functions,
        vec![
            FunctionComplexity { function: "Outer".to_owned(), line: 1, complexity: 2 },
            FunctionComplexity { function: "Outer::Inner".to_owned(), line: 1, complexity: 3 },
        ]
    );
}

#[test]
fn Test_A_Method_Should_Be_Named_Under_Its_Self_Type_And_Its_Module()
{
    let source = "mod shapes {\n    struct Walker;\n    impl Walker {\n        fn Step(&self) {}\n    }\n}\n";

    assert_eq!(Functions(source), vec![FunctionComplexity { function: "shapes::Walker::Step".to_owned(), line: 4, complexity: 1 }]);
}

#[test]
fn Test_A_Trait_Method_Should_Be_A_Subject_Only_Where_It_Has_A_Body()
{
    let functions = Functions("trait Judged { fn Bare(&self); fn Bodied(&self) {} }");

    assert_eq!(functions, vec![FunctionComplexity { function: "Judged::Bodied".to_owned(), line: 1, complexity: 1 }]);
}

/// A decision outside every function belongs to no subject, so it is lost rather than credited to
/// whichever function happens to follow it.
#[test]
fn Test_A_Decision_Outside_Every_Function_Should_Be_Credited_To_None()
{
    let functions = Functions("const LIMIT: u8 = if true { 1 } else { 2 };\nfn After() {}\n");

    assert_eq!(functions, vec![FunctionComplexity { function: "After".to_owned(), line: 2, complexity: 1 }]);
}
