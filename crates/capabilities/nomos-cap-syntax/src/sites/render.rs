//! Writing a sites payload out, byte for byte the same every time.

use super::{SiteRecord, SiteValue, Site_Kind, SitesPayload};
use crate::Escape_Text;
use core::fmt::Write as _;

/// Renders `payload` to the bytes the grammar describes: its offers, then its declines, then its
/// sites in the order given, each site's fields in its kind's declared order.
///
/// It writes what it is handed and judges none of it. A field the kind does not declare is
/// written after the declared ones and a record of an undeclared kind is written in field-name
/// order, so a provider's mistake reaches [`super::Parse_Sites_Payload`] -- which refuses it in
/// the provider's own tests -- instead of being dropped here where nobody would see it.
#[must_use]
pub fn Render_Sites_Payload(payload: &SitesPayload) -> Vec<u8>
{
    let mut rendered = String::new();

    for kind in &payload.offered
    {
        let _ = writeln!(rendered, "offers\t{kind}");
    }
    for decline in &payload.declined
    {
        let _ = writeln!(rendered, "declines\t{}\t{}", decline.kind, Escape_Text(&decline.reason));
    }
    for record in &payload.records
    {
        let _ = writeln!(rendered, "site\t{}\t{}{}", record.kind, record.line, Fields_Of(record));
    }

    return rendered.into_bytes();
}

/// A record's fields, each with the tab before it, in its kind's declared order and then any
/// field the kind does not declare.
fn Fields_Of(record: &SiteRecord) -> String
{
    let declared: Vec<&str> = Site_Kind(&record.kind).map_or_else(Vec::new, |kind| return kind.fields.iter().map(|field| return field.name).collect());
    let undeclared = record.values.keys().map(String::as_str).filter(|name| return !declared.contains(name));

    let mut fields = String::new();
    for name in declared.iter().copied().chain(undeclared)
    {
        if let Some(value) = record.values.get(name)
        {
            let _ = write!(fields, "\t{name}={}", Encode_Value(value));
        }
    }

    return fields;
}

/// One value as its type is written.
fn Encode_Value(value: &SiteValue) -> String
{
    return match value
    {
        SiteValue::Text(text) => Escape_Text(text),
        SiteValue::Integer(integer) => integer.to_string(),
        SiteValue::Truth(truth) => truth.to_string(),
        SiteValue::TextList(items) => items.iter().map(|item| return format!("{};", Escape_Text(item).replace(';', "\\;"))).collect(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{KindDecline, LABELED_JUMP, LabeledJump};

    /// The line the fixture jump is written on.
    const JUMP_LINE: usize = 4;
    /// The line the fixture jump's label is written on.
    const LABEL_LINE: usize = 2;

    #[test]
    fn Test_Render_Should_Write_Stances_Then_Sites_In_Declared_Field_Order()
    {
        let jump = LabeledJump {
            line: JUMP_LINE,
            keyword: "break".to_owned(),
            label: "'outer".to_owned(),
            text: "break 'outer".to_owned(),
            label_line: LABEL_LINE,
            has_same_target_unlabeled: true,
            is_inside_break_capture: false,
        };
        let payload = SitesPayload {
            offered: vec![LABELED_JUMP.name.to_owned()],
            declined: vec![KindDecline { kind: "other".to_owned(), reason: "none\there".to_owned() }],
            records: vec![jump.Record()],
        };

        let rendered = String::from_utf8(Render_Sites_Payload(&payload)).expect("the payload is text");

        assert_eq!(
            rendered,
            "offers\tlabeled-jump\n\
             declines\tother\tnone\\there\n\
             site\tlabeled-jump\t4\tkeyword=break\tlabel='outer\ttext=break 'outer\tlabel_line=2\thas_same_target_unlabeled=true\tis_inside_break_capture=false\n"
        );
    }

    #[test]
    fn Test_Encode_Value_Should_Terminate_Every_List_Item_And_Escape_Its_Separator()
    {
        assert_eq!(Encode_Value(&SiteValue::TextList(Vec::new())), "");
        assert_eq!(Encode_Value(&SiteValue::TextList(vec![String::new()])), ";");
        assert_eq!(Encode_Value(&SiteValue::TextList(vec!["a;b".to_owned(), "c\\".to_owned()])), "a\\;b;c\\\\;");
    }
}
