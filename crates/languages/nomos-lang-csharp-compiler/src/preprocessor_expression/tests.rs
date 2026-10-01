//! The grammar, operator by operator and by precedence, and every refusal.

use super::{Definitions, Evaluate};
use std::collections::BTreeSet;

fn Set(symbols: &[&str]) -> BTreeSet<String>
{
    return symbols.iter().map(|symbol| return (*symbol).to_owned()).collect();
}

fn Value(text: &str, defined: &[&str]) -> Option<bool>
{
    let defined = Set(defined);
    let uncertain = BTreeSet::new();
    return Evaluate(text, &Definitions { defined: &defined, uncertain: &uncertain });
}

#[test]
fn Test_Each_Operator_Should_Evaluate_As_The_Specification_Defines_It()
{
    let cases: [(&str, bool); 12] = [
        ("DEBUG", true),
        ("RELEASE", false),
        ("true", true),
        ("false", false),
        ("!RELEASE", true),
        ("DEBUG && RELEASE", false),
        ("DEBUG || RELEASE", true),
        ("DEBUG == true", true),
        ("RELEASE != false", false),
        ("(DEBUG || RELEASE) && !RELEASE", true),
        ("DEBUG // trailing comment", true),
        ("  DEBUG  ", true),
    ];

    for (text, expected) in cases
    {
        assert_eq!(Value(text, &["DEBUG"]), Some(expected), "{text}");
    }
}

/// `!` binds tighter than `==`, which binds tighter than `&&`, which binds tighter than `||`.
#[test]
fn Test_Operators_Should_Bind_In_The_Specifications_Order()
{
    assert_eq!(Value("A || B && C", &["A"]), Some(true), "|| is loosest: A || (B && C)");
    assert_eq!(Value("!A == B", &["A"]), Some(true), "(!A) == B, both false");
    assert_eq!(Value("A && B == C", &["A"]), Some(true), "A && (B == C), B and C both false");
}

#[test]
fn Test_What_Is_Not_A_Preprocessing_Expression_Should_Be_Refused()
{
    for text in ["", "1", "(DEBUG", "DEBUG)", "DEBUG &&", "DEBUG & RELEASE", "/* c */ DEBUG", "DEBUG RELEASE", "!"]
    {
        assert_eq!(Value(text, &["DEBUG"]), None, "{text:?}");
    }
}

/// A symbol a `#define` in an unevaluated branch may have changed has no known value, and an
/// expression reading it has none either.
#[test]
fn Test_An_Uncertain_Symbol_Should_Leave_The_Expression_Unevaluated()
{
    let defined = Set(&["DEBUG"]);
    let uncertain = Set(&["LOCAL"]);
    let definitions = Definitions { defined: &defined, uncertain: &uncertain };

    assert_eq!(Evaluate("DEBUG && LOCAL", &definitions), None);
    assert_eq!(Evaluate("DEBUG", &definitions), Some(true));
}
