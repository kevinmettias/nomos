//! What this module promises, exercised.

use super::*;
use crate::payload_refusal::PayloadRefusal;
use crate::payload_item::PayloadItem;
use crate::syntax_payload::SyntaxPayload;

const SAMPLE: &str = "unexpanded\t2\n\
                      item\t0\tModule\tPrivate\ttests\t.\t.\n\
                      item\t1\tConstant\tPublic\tTABLES\t+Mirrored by `Test_X`.\t+slice\n\
                      item\t2\tFunction\tNotApplicable\tJudged::Two\t-\t-\n";

#[test]
fn Test_A_Well_Formed_Payload_Should_Decode_To_Its_Records()
{
    let payload = Parse_Payload(SAMPLE.as_bytes()).expect("this is the grammar");

    assert_eq!(payload.unexpanded, 2);
    assert_eq!(payload.items.len(), 3);

    let list = payload.items.get(1).expect("three items");
    assert_eq!(list.kind, "Constant");
    assert_eq!(list.Own_Name(), "TABLES");
    assert!(list.Is_Public());
    assert_eq!(list.documentation.Value(), Some("Mirrored by `Test_X`."));
    assert_eq!(list.shape.Value(), Some(SLICE));
}

/// The distinction v2 exists for, asserted as bytes rather than as a description.
#[test]
fn Test_Not_Observed_And_Absent_Should_Be_Different_Bytes()
{
    assert_ne!(Observation::NotObserved.Encode(), Observation::Absent.Encode());

    let payload = Parse_Payload(SAMPLE.as_bytes()).expect("the grammar");

    let looked = payload.items.first().expect("three items");
    assert_eq!(looked.documentation, Observation::Absent);
    assert!(looked.documentation.Was_Observed(), "this provider read doc comments");

    let blind = payload.items.get(2).expect("three items");
    assert_eq!(blind.documentation, Observation::NotObserved);
    assert!(!blind.documentation.Was_Observed());

    // Both answer `None` to "what is the text", which is exactly why the caller that
    // must not confuse them has a second question to ask.
    assert_eq!(looked.documentation.Value(), blind.documentation.Value());
}

/// A member belongs to the record it follows, and two impls of one type are told apart
/// by nothing else.
#[test]
fn Test_A_Member_Should_Be_Enclosed_By_The_Record_It_Follows()
{
    let payload = Parse_Payload(
        b"unexpanded\t0\n\
          item\t0\tImplementation\tNotApplicable\tTable\t.\t+trait\n\
          item\t1\tFunction\tNotApplicable\tTable::All\t.\t+fn/1\n\
          item\t2\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
          item\t3\tFunction\tPublic\tTable::All\t.\t+fn/0\n",
    )
    .expect("well formed");

    let under_trait = payload.Enclosing(1).expect("the first All is enclosed");
    assert_eq!(under_trait.shape.Value(), Some(TRAIT));

    let under_inherent = payload.Enclosing(3).expect("the second All is enclosed");
    assert_eq!(under_inherent.shape.Value(), Some(INHERENT));

    assert!(payload.Enclosing(0).is_none(), "a top-level item encloses nothing");
}

#[test]
fn Test_A_Function_Shape_Should_Carry_Its_Arity()
{
    assert_eq!(Function_Shape(0), "fn/0");
    assert_eq!(
        Function_Arity(&Observation::Present(Function_Shape(2))),
        Some(2)
    );
    assert_eq!(Function_Arity(&Observation::NotObserved), None);
    assert_eq!(Function_Arity(&Observation::Present(SLICE.to_owned())), None);
}

/// Documentation is prose and arrives with newlines and tabs in it. One field, one
/// record, and the text a consumer matches against is the text the author wrote.
#[test]
fn Test_Documentation_Should_Survive_The_Field_It_Travels_In()
{
    let prose = "Mirrored by `Test_X`.\n\nA second\tparagraph with a \\ in it.";
    let payload = SyntaxPayload {
        unexpanded: 0,
        items: vec![PayloadItem {
            ordinal: 0,
            kind: "Constant".to_owned(),
            visibility: PUBLIC.to_owned(),
            qualified_name: "TABLES".to_owned(),
            documentation: Observation::Present(prose.to_owned()),
            shape: Observation::Present(SLICE.to_owned()),
        }],
    };

    let rendered = Render_Payload(&payload);
    assert_eq!(
        rendered.iter().filter(|byte| return **byte == b'\n').count(),
        2,
        "the newlines in the prose must not become records"
    );

    let read = Parse_Payload(&rendered).expect("what this module wrote");
    assert_eq!(read.items.first().expect("one item").documentation.Value(), Some(prose));
}

/// A file that declares nothing is a real answer, and the shortest well-formed payload.
#[test]
fn Test_A_File_That_Declares_Nothing_Should_Decode_To_No_Items()
{
    let payload = Parse_Payload(b"unexpanded\t0\n").expect("a header alone is well formed");

    assert_eq!(payload.unexpanded, 0);
    assert!(payload.items.is_empty());
}

/// The negative controls. None of these may arrive at a caller as an empty payload: a
/// reader that decodes unreadable bytes to "no items" makes a subject that was never
/// read indistinguishable from a subject that declared nothing.
#[test]
fn Test_A_Payload_This_Build_Cannot_Read_Should_Not_Decode_To_No_Items()
{
    assert_eq!(Parse_Payload(&[0xFF, 0xFE]), Err(PayloadRefusal::NotUtf8));
    assert_eq!(Parse_Payload(b""), Err(PayloadRefusal::NoHeader));
    assert_eq!(
        Parse_Payload(b"item\t0\tFunction\tPublic\tOne\t.\t.\n"),
        Err(PayloadRefusal::NoHeader)
    );
    assert_eq!(
        Parse_Payload(b"unexpanded\t0\nunexpanded\t1\n"),
        Err(PayloadRefusal::RepeatedHeader { line: 2 })
    );

    let unknown = Parse_Payload(b"unexpanded\t0\nregion\t0\t3\n").expect_err("unknown tag");
    assert!(
        matches!(unknown, PayloadRefusal::UnknownRecord { ref tag, line: 2 } if tag == "region"),
        "{unknown:?}"
    );

    // A v1 record, which is this schema's likeliest wrong input rather than a
    // hypothetical one: five fields where seven are written.
    let previous = Parse_Payload(b"unexpanded\t0\nitem\t0\tFunction\tPublic\tOne\n")
        .expect_err("the previous schema is not this one");
    assert!(
        matches!(
            previous,
            PayloadRefusal::WrongFieldCount {
                found: 5,
                expected: 7,
                line: 2,
                ..
            }
        ),
        "{previous:?}"
    );

    let unnumbered = Parse_Payload(b"unexpanded\tmany\n").expect_err("a count is a number");
    assert!(
        matches!(unnumbered, PayloadRefusal::UnreadableNumber { field: "unexpanded", .. }),
        "{unnumbered:?}"
    );

    let unmarked = Parse_Payload(b"unexpanded\t0\nitem\t0\tConstant\tPublic\tA\tnone\t.\n")
        .expect_err("an observation has three spellings and this is not one");
    assert!(
        matches!(
            unmarked,
            PayloadRefusal::UnreadableObservation {
                field: "documentation",
                line: 2,
                ..
            }
        ),
        "{unmarked:?}"
    );
}

/// Every refusal says where and what, because the text reaches somebody who has to act.
#[test]
fn Test_A_Refusal_Should_Say_What_It_Refused()
{
    let described = Parse_Payload(b"unexpanded\t0\nregion\t0\n")
        .expect_err("an unknown tag refuses")
        .Describe();

    assert!(described.contains("region"), "{described}");
    assert!(described.contains('2'), "{described}");
}

/// The grammar against itself. This is the only thing [`Render_Payload`] is for.
#[test]
fn Test_A_Decoded_Payload_Should_Render_Back_To_The_Bytes_It_Came_From()
{
    let payload = Parse_Payload(SAMPLE.as_bytes()).expect("the grammar");
    let rendered = Render_Payload(&payload);

    assert_eq!(String::from_utf8(rendered.clone()).as_deref(), Ok(SAMPLE));
    assert!(!rendered.contains(&b'\r'), "line endings must not be local");
}
