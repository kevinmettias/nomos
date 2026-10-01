//! The encoding round-trips, orders by position, and refuses what is not its shape.

use super::*;

fn Value(line: u32, column: u32, is_error: bool, type_name: &str) -> DiscardedValue
{
    return DiscardedValue { line, column, is_error, type_name: type_name.to_owned() };
}

#[test]
fn Test_A_Payload_Should_Round_Trip_In_Position_Order()
{
    let payload = DiscardedValuesPayload { values: vec![Value(13, 5, true, "error"), Value(12, 5, true, "error"), Value(13, 2, false, "int")] };

    let parsed = Parse_Payload(&Encode_Payload(&payload)).expect("its own encoding");

    assert_eq!(parsed.values, [Value(12, 5, true, "error"), Value(13, 2, false, "int"), Value(13, 5, true, "error")]);
}

/// A file that discards nothing is an empty answer, and reads back as one.
#[test]
fn Test_An_Empty_Payload_Should_Round_Trip_As_Empty()
{
    let parsed = Parse_Payload(&Encode_Payload(&DiscardedValuesPayload::default())).expect("an empty answer");

    assert!(parsed.values.is_empty());
}

/// A type with spaces in it survives, and a line of another shape is refused rather than skipped.
#[test]
fn Test_A_Line_Of_Another_Shape_Should_Be_Refused()
{
    let spaced = Parse_Payload(b"value\t3\t2\tother\tfunc(x int) error\n").expect("a func type");

    assert_eq!(spaced.values.first().map(|value| return value.type_name.as_str()), Some("func(x int) error"));
    assert!(Parse_Payload(b"value\t3\t2\tmaybe\tint\n").is_err());
    assert!(Parse_Payload(b"entry\t3\t2\terror\terror\n").is_err());
    assert!(Parse_Payload(b"value\tthree\t2\terror\terror\n").is_err());
}
