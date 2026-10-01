//! The codec's own tests: a round trip, the stable bytes, and every refusal by its reason.

use super::*;

#[test]
fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
{
    let payload = Sample();
    let encoded = Encode_Payload(&payload);
    let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

    assert_eq!(decoded, payload);
}

#[test]
fn Test_Encode_Payload_Should_Write_Every_Descriptor_Property_Before_Any_Function()
{
    let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("UTF-8 by construction");
    let tags: Vec<&str> = rendered.lines().filter_map(|line| return line.split('\t').next()).collect();

    assert_eq!(tags.iter().filter(|tag| return **tag == "descriptor").count(), 10, "{rendered}");
    assert_eq!(tags.iter().rev().take(2).copied().collect::<Vec<&str>>(), ["function", "function"], "{rendered}");
    assert!(rendered.ends_with("function\tParse\t3\t1\nfunction\tWalker::Step\t9\t7\n"), "{rendered}");
    assert!(!rendered.contains('\r'), "line endings must not be local");
}

/// MET-007 is only checkable against a descriptor that says a sum means nothing, so the one this
/// capability publishes must say so, and say which way is worse.
#[test]
fn Test_The_Complexity_Descriptor_Should_Be_Not_Aggregable_And_Higher_Is_Worse()
{
    let descriptor = Complexity_Descriptor();

    assert_eq!(descriptor.aggregation, Aggregation::NotAggregable);
    assert_eq!(descriptor.directionality, Directionality::HigherIsWorse);
}

/// A property left out is refused, one at a time, for each of the ten -- so no property of the
/// descriptor is optional on the wire.
#[test]
fn Test_A_Payload_Missing_Any_Descriptor_Property_Should_Be_Refused()
{
    let encoded = String::from_utf8(Encode_Payload(&Sample())).expect("UTF-8 by construction");
    let descriptor_lines: Vec<&str> = encoded.lines().filter(|line| return line.starts_with("descriptor\t")).collect();
    assert_eq!(descriptor_lines.len(), 10);

    for omitted in &descriptor_lines
    {
        let kept: String = encoded.lines().filter(|line| return line != omitted).map(|line| return format!("{line}\n")).collect();
        let error = Parse_Payload(kept.as_bytes()).expect_err("a descriptor with a property missing must be refused");
        assert!(error.reason.starts_with("no descriptor line declares"), "{omitted:?}: {}", error.reason);
    }
}

#[test]
fn Test_A_Property_Declared_Twice_Should_Be_Refused()
{
    let mut encoded = Encode_Payload(&Sample());
    encoded.extend_from_slice(b"descriptor\taggregation\tsum\n");

    let error = Parse_Payload(&encoded).expect_err("two answers to one property must be refused");

    assert!(error.reason.contains("declared twice"), "{}", error.reason);
}

#[test]
fn Test_An_Unrecognized_Label_Or_Property_Should_Be_Refused()
{
    let mut unknown_label = Without_Line("descriptor\taggregation\t");
    unknown_label.push_str("descriptor\taggregation\taverage\n");
    let mut unknown_property = Encode_Text();
    unknown_property.push_str("descriptor\tprecision\thigh\n");

    let label = Parse_Payload(unknown_label.as_bytes()).expect_err("an aggregation this schema does not name");
    let property = Parse_Payload(unknown_property.as_bytes()).expect_err("a property MET-006 does not name");

    assert!(label.reason.contains("unrecognized descriptor label"), "{}", label.reason);
    assert!(property.reason.contains("names no MET-006 property"), "{}", property.reason);
}

#[test]
fn Test_A_Malformed_Function_Line_Should_Be_Refused_For_Its_Own_Reason()
{
    let cases: [(&str, &str); 4] = [
        ("function\tF\t3\n", "does not have exactly three fields"),
        ("function\tF\t3\t1\t9\n", "does not have exactly three fields"),
        ("function\tF\tthree\t1\n", "line is not a count"),
        ("function\tF\t3\t-1\n", "complexity is not a count"),
    ];

    for (line, reason) in cases
    {
        let mut text = Encode_Text();
        text.push_str(line);
        let error = Parse_Payload(text.as_bytes()).expect_err("a malformed function line");
        assert!(error.reason.contains(reason), "{line:?}: {}", error.reason);
    }
}

#[test]
fn Test_A_Line_With_No_Known_Tag_Should_Be_Refused()
{
    let mut text = Encode_Text();
    text.push_str("site\tF\tx\tempty\n");

    let error = Parse_Payload(text.as_bytes()).expect_err("a line of another schema");

    assert!(error.reason.contains("neither a descriptor nor a function"), "{}", error.reason);
}

/// An empty byte string carries no descriptor, so it is refused rather than read as a file with
/// no functions: a file with none still answers with the terms its absence is read under.
#[test]
fn Test_An_Empty_Payload_Should_Be_Refused_Rather_Than_Read_As_No_Functions()
{
    let error = Parse_Payload(&[]).expect_err("no descriptor at all");

    assert!(error.reason.starts_with("no descriptor line declares"), "{}", error.reason);
}

#[test]
fn Test_A_Descriptor_With_No_Function_Should_Round_Trip_To_No_Functions()
{
    let payload = ComplexityPayload { descriptor: Complexity_Descriptor(), functions: Vec::new() };

    let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("a file defining no function");

    assert_eq!(decoded, payload);
}

#[test]
fn Test_Every_Label_Should_Round_Trip_Through_Its_Parser()
{
    for aggregation in [Aggregation::NotAggregable, Aggregation::Sum, Aggregation::Maximum]
    {
        assert_eq!(Aggregation::From_Label(aggregation.Label()), Some(aggregation));
    }
    for directionality in [Directionality::HigherIsWorse, Directionality::LowerIsWorse]
    {
        assert_eq!(Directionality::From_Label(directionality.Label()), Some(directionality));
    }
    assert_eq!(Aggregation::From_Label("average"), None);
    assert_eq!(Directionality::From_Label("better"), None);
}

fn Encode_Text() -> String
{
    return String::from_utf8(Encode_Payload(&Sample())).expect("UTF-8 by construction");
}

fn Without_Line(prefix: &str) -> String
{
    return Encode_Text().lines().filter(|line| return !line.starts_with(prefix)).map(|line| return format!("{line}\n")).collect();
}

fn Sample() -> ComplexityPayload
{
    return ComplexityPayload {
        descriptor: Complexity_Descriptor(),
        functions: vec![
            FunctionComplexity { function: "Parse".to_owned(), line: 3, complexity: 1 },
            FunctionComplexity { function: "Walker::Step".to_owned(), line: 9, complexity: 7 },
        ],
    };
}
