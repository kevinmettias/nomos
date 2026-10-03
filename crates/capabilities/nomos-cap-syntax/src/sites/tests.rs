//! The grammar held against itself: what the writer writes the reader reads back, and every way a
//! payload can fail to be this schema is refused rather than read.

use super::*;

/// The line the fixture jump is written on.
const JUMP_LINE: usize = 6;
/// The line the fixture jump's label is written on.
const LABEL_LINE: usize = 3;

/// The one well-formed site record every refusal below is a mutation of.
const SITE: &str = "site\tlabeled-jump\t6\tkeyword=break\tlabel='outer\ttext=break 'outer\tlabel_line=3\thas_same_target_unlabeled=false\tis_inside_break_capture=false";

fn Jump() -> LabeledJump
{
    return LabeledJump {
        line: JUMP_LINE,
        keyword: "break".to_owned(),
        label: "'outer".to_owned(),
        text: "break 'outer".to_owned(),
        label_line: LABEL_LINE,
        has_same_target_unlabeled: false,
        is_inside_break_capture: false,
    };
}

fn Offering(records: &[&str]) -> String
{
    let mut payload = "offers\tlabeled-jump\n".to_owned();
    for record in records
    {
        payload.push_str(record);
        payload.push('\n');
    }
    return payload;
}

fn Refusal_Of(payload: &str) -> SitesRefusalKind
{
    return Parse_Sites_Payload(payload.as_bytes()).expect_err("the payload is not this schema").kind;
}

#[test]
fn Test_A_Rendered_Payload_Should_Read_Back_Exactly()
{
    let payload = SitesPayload { offered: vec![LABELED_JUMP.name.to_owned()], declined: Vec::new(), records: vec![Jump().Record()] };

    let read = Parse_Sites_Payload(&Render_Sites_Payload(&payload)).expect("the writer writes this schema");

    assert_eq!(read, payload);
    assert_eq!(LabeledJump::All_In(&read), Some(vec![Jump()]));
}

#[test]
fn Test_A_Decline_Should_Read_Back_With_Its_Reason()
{
    let payload = SitesPayload {
        offered: Vec::new(),
        declined: vec![KindDecline { kind: LABELED_JUMP.name.to_owned(), reason: "the language cannot name a loop".to_owned() }],
        records: Vec::new(),
    };

    let read = Parse_Sites_Payload(&Render_Sites_Payload(&payload)).expect("the writer writes this schema");

    assert_eq!(read.Stance(LABELED_JUMP.name), KindStance::Declined("the language cannot name a loop"));
}

/// An offered kind with no sites is a file with none of the construct -- a real, clean answer.
#[test]
fn Test_An_Offer_With_No_Sites_Should_Be_A_Clean_Answer()
{
    let read = Parse_Sites_Payload(Offering(&[]).as_bytes()).expect("an offer alone is a payload");

    assert_eq!(read.Stance(LABELED_JUMP.name), KindStance::Offered);
    assert!(read.records.is_empty());
}

#[test]
fn Test_The_Fields_Of_A_Site_May_Come_In_Any_Order()
{
    let reordered = "site\tlabeled-jump\t6\tis_inside_break_capture=false\thas_same_target_unlabeled=false\tlabel_line=3\ttext=break 'outer\tlabel='outer\tkeyword=break";

    let read = Parse_Sites_Payload(Offering(&[reordered]).as_bytes()).expect("field order is not the grammar's");

    assert_eq!(LabeledJump::All_In(&read), Some(vec![Jump()]));
}

/// The empty payload is every declared kind unanswered -- a gap a rule reports, never a pass -- so
/// it is read rather than refused; sites with no stance offering their kind are still refused.
#[test]
fn Test_The_Empty_Payload_Should_Leave_Every_Kind_Unanswered()
{
    let empty = Parse_Sites_Payload(b"").expect("the empty payload states no stance, which is a payload");

    assert_eq!(empty.Stance(LABELED_JUMP.name), KindStance::Unanswered);
    assert_eq!(Refusal_Of(&format!("{SITE}\n")), SitesRefusalKind::UnofferedSite { kind: "labeled-jump".to_owned() });
    assert_eq!(Parse_Sites_Payload(&[0xff, 0xfe]).expect_err("not UTF-8").kind, SitesRefusalKind::NotUtf8);
}

#[test]
fn Test_A_Kind_No_Declaration_Names_Should_Be_Refused_In_A_Stance_And_In_A_Site()
{
    assert_eq!(Refusal_Of("offers\tlabelled-jump\n"), SitesRefusalKind::UndeclaredKind { kind: "labelled-jump".to_owned() });

    let site = SITE.replacen("labeled-jump", "nested-call", 1);
    assert_eq!(Refusal_Of(&Offering(&[&site])), SitesRefusalKind::UndeclaredKind { kind: "nested-call".to_owned() });
}

#[test]
fn Test_A_Field_The_Kind_Does_Not_Declare_Should_Be_Refused()
{
    let site = format!("{SITE}\tdepth=2");

    assert_eq!(Refusal_Of(&Offering(&[&site])), SitesRefusalKind::UndeclaredField { kind: "labeled-jump".to_owned(), field: "depth".to_owned() });
}

#[test]
fn Test_A_Declared_Field_The_Site_Lacks_Should_Be_Refused()
{
    let site = SITE.replace("\tlabel_line=3", "");

    assert_eq!(Refusal_Of(&Offering(&[&site])), SitesRefusalKind::MissingField { kind: "labeled-jump".to_owned(), field: "label_line".to_owned() });
}

#[test]
fn Test_A_Field_Carried_Twice_Should_Be_Refused()
{
    let site = format!("{SITE}\tlabel_line=4");

    assert_eq!(Refusal_Of(&Offering(&[&site])), SitesRefusalKind::RepeatedField { kind: "labeled-jump".to_owned(), field: "label_line".to_owned() });
}

#[test]
fn Test_A_Value_Of_The_Wrong_Type_Should_Be_Refused()
{
    let integer = SITE.replace("label_line=3", "label_line=three");
    assert_eq!(
        Refusal_Of(&Offering(&[&integer])),
        SitesRefusalKind::UnreadableValue { field: "label_line".to_owned(), value: "three".to_owned(), expected: SiteValueType::Integer }
    );

    let truth = SITE.replace("is_inside_break_capture=false", "is_inside_break_capture=no");
    assert_eq!(
        Refusal_Of(&Offering(&[&truth])),
        SitesRefusalKind::UnreadableValue { field: "is_inside_break_capture".to_owned(), value: "no".to_owned(), expected: SiteValueType::Truth }
    );
}

#[test]
fn Test_A_Malformed_Field_And_An_Unreadable_Line_Should_Be_Refused()
{
    let malformed = format!("{SITE}\tdepth");
    assert_eq!(Refusal_Of(&Offering(&[&malformed])), SitesRefusalKind::MalformedField { field: "depth".to_owned() });

    let line = SITE.replacen("\t6\t", "\tsix\t", 1);
    assert!(matches!(Refusal_Of(&Offering(&[&line])), SitesRefusalKind::UnreadableLine { value, .. } if value == "six"));

    assert!(matches!(Refusal_Of(&Offering(&["site\tlabeled-jump"])), SitesRefusalKind::WrongFieldCount { found: 2, .. }));
}

/// A site of a kind the payload declines, or never names, is a record nobody vouched for.
#[test]
fn Test_A_Site_Of_A_Kind_The_Payload_Does_Not_Offer_Should_Be_Refused()
{
    let declining = format!("declines\tlabeled-jump\tno loops have names here\n{SITE}\n");

    let refusal = Parse_Sites_Payload(declining.as_bytes()).expect_err("the kind is declined");

    assert_eq!(refusal.kind, SitesRefusalKind::UnofferedSite { kind: "labeled-jump".to_owned() });
    assert_eq!(refusal.line, Some(2));
}

#[test]
fn Test_A_Second_Stance_And_A_Decline_Without_A_Reason_Should_Be_Refused()
{
    assert_eq!(Refusal_Of("offers\tlabeled-jump\noffers\tlabeled-jump\n"), SitesRefusalKind::RepeatedStance { kind: "labeled-jump".to_owned() });
    assert_eq!(Refusal_Of("offers\tlabeled-jump\ndeclines\tlabeled-jump\twhy\n"), SitesRefusalKind::RepeatedStance { kind: "labeled-jump".to_owned() });
    assert_eq!(Refusal_Of("declines\tlabeled-jump\t\n"), SitesRefusalKind::UnexplainedDecline { kind: "labeled-jump".to_owned() });
    assert!(matches!(Refusal_Of("offers\tlabeled-jump\textra\n"), SitesRefusalKind::WrongFieldCount { found: 3, .. }));
    assert_eq!(Refusal_Of("region\t1\n"), SitesRefusalKind::UnknownRecord { tag: "region".to_owned() });
}
