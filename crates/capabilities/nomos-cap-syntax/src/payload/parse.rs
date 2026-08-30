//! Reading a payload back, and refusing one that does not say what it claims.

use super::{SyntaxPayload, PayloadRefusal, PayloadRefusalKind, PayloadItem, Observed_Field};

/// Fields in an `item` record, tag included.
const ITEM_FIELDS: usize = 7;

/// Fields in the `unexpanded` header, tag included.
const HEADER_FIELDS: usize = 2;

/// A record read left to right, so a field's place is the order it is asked for rather than a
/// number typed beside a grammar written elsewhere.
///
/// The number and the encoder drift apart in silence. A field inserted into the record
/// renumbers every field after it, and nothing in the language ties `fields.get(4)` to the
/// fifth thing the encoder writes — the reader keeps compiling and starts filling the wrong
/// fields. Asking in order leaves the grammar as the only place the order is stated.
struct Fields<'record, 'text>
{
    fields: &'record [&'text str],
    next: usize,
}

impl<'record, 'text> Fields<'record, 'text>
{
    /// The fields after the tag, which every record spends its first column on.
    fn After_The_Tag(fields: &'record [&'text str]) -> Self
    {
        return Self { fields, next: 1 };
    }

    /// The next field the grammar names, empty where the record is short.
    ///
    /// `Expect_Fields` has already refused a record of the wrong length by the time any
    /// reader asks, so the empty string here is unreachable rather than a quiet default.
    fn Next(&mut self) -> &'text str
    {
        let at = self.next;

        self.next = at.saturating_add(1);
        return self.fields.get(at).copied().unwrap_or_default();
    }
}

/// Reads a payload, or refuses it.
///
/// Never a partial answer. A payload half of which decodes is an index that is silently
/// short, and the caller cannot tell it from a file that declared that much.
///
/// # Errors
///
/// [`PayloadRefusal`] for anything that is not a well-formed payload by the grammar above.
pub fn Parse_Payload(bytes: &[u8]) -> Result<SyntaxPayload, PayloadRefusal>
{
    let Ok(text) = core::str::from_utf8(bytes)
    else
    {
        return Err(PayloadRefusal::Whole(PayloadRefusalKind::NotUtf8));
    };

    let mut read = Reading {
        unexpanded: None,
        items: Vec::new(),
    };

    for (offset, line) in text.lines().enumerate()
    {
        Read_Record(line, offset.saturating_add(1), &mut read)?;
    }

    return read.Complete();
}

/// A payload part-way through being read.
///
/// `unexpanded` is optional here and not on [`SyntaxPayload`], because the header may not
/// have arrived yet — and a payload that never carries one is refused rather than defaulted.
pub(super) struct Reading
{
    unexpanded: Option<u32>,
    items: Vec<PayloadItem>,
}

impl Reading
{
    /// The payload a completed read describes, if it describes one.
    ///
    /// A payload with no header at all is refused rather than read as zero unexpanded
    /// macros: zero is a claim that the provider looked and found none, and nobody looked.
    fn Complete(self) -> Result<SyntaxPayload, PayloadRefusal>
    {
        let Some(unexpanded) = self.unexpanded
        else
        {
            return Err(PayloadRefusal::Whole(PayloadRefusalKind::NoHeader));
        };

        return Ok(SyntaxPayload {
            unexpanded,
            items: self.items,
        });
    }
}

/// One record of a payload.
///
/// `split` yields at least one element for every input, including the empty one, so an
/// absent tag is an empty tag and is refused rather than skipped.
pub(super) fn Read_Record(line: &str, at: usize, read: &mut Reading) -> Result<(), PayloadRefusal>
{
    let fields: Vec<&str> = line.split('\t').collect();
    let tag = fields.first().copied().unwrap_or_default();
    match tag
    {
        "unexpanded" => read.unexpanded = Some(Header_Count(&fields, at, read.unexpanded)?),
        "item" =>
        {
            let item = Read_Item(&fields, at, read.unexpanded)?;
            read.items.push(item);
        }
        other => return Err(Unknown_Record(other, at)),
    }

    return Ok(());
}

/// A tag this build does not know is most likely a newer schema, so it is refused rather than
/// skipped.
fn Unknown_Record(tag: &str, at: usize) -> PayloadRefusal
{
    return PayloadRefusal::At(
        at,
        PayloadRefusalKind::UnknownRecord {
            tag: tag.to_owned(),
        },
    );
}

/// One item record, refused if the header has not arrived yet.
///
/// An item before the header is the same defect as no header at all: the count that says how
/// much of the tree went unread is missing at the moment the items are being believed.
pub(super) fn Read_Item(
    fields: &[&str],
    at: usize,
    unexpanded: Option<u32>,
) -> Result<PayloadItem, PayloadRefusal>
{
    if unexpanded.is_none()
    {
        return Err(PayloadRefusal::Whole(PayloadRefusalKind::NoHeader));
    }
    Expect_Fields("item", fields, ITEM_FIELDS, at)?;

    return Item_Record(fields, at);
}

/// The header's count, refusing a second one.
///
/// A payload carrying two headers does not say which count is its own, and taking either
/// would be inventing an answer the bytes do not give.
pub(super) fn Header_Count(fields: &[&str], at: usize, already: Option<u32>) -> Result<u32, PayloadRefusal>
{
    if already.is_some()
    {
        return Err(PayloadRefusal::At(at, PayloadRefusalKind::RepeatedHeader));
    }
    Expect_Fields("unexpanded", fields, HEADER_FIELDS, at)?;

    let mut record = Fields::After_The_Tag(fields);

    return Parsed_Number(record.Next(), "unexpanded", at);
}

/// One item record, with both observation-bearing fields read as observations.
pub(super) fn Item_Record(fields: &[&str], at: usize) -> Result<PayloadItem, PayloadRefusal>
{
    let mut record = Fields::After_The_Tag(fields);

    return Ok(PayloadItem {
        ordinal: Parsed_Number(record.Next(), "ordinal", at)?,
        kind: record.Next().to_owned(),
        visibility: record.Next().to_owned(),
        qualified_name: record.Next().to_owned(),
        documentation: Observed_Field(record.Next(), "documentation", at)?,
        shape: Observed_Field(record.Next(), "shape", at)?,
    });
}

/// Refuses a record whose field count is not the one the grammar states.
pub(super) fn Expect_Fields(
    tag: &str,
    fields: &[&str],
    expected: usize,
    line: usize,
) -> Result<(), PayloadRefusal>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(PayloadRefusal::At(
        line,
        PayloadRefusalKind::WrongFieldCount {
            tag: tag.to_owned(),
            expected,
            found: fields.len(),
        },
    ));
}

/// Reads a field that must be a number.
pub(super) fn Parsed_Number(value: &str, field: &'static str, line: usize) -> Result<u32, PayloadRefusal>
{
    return value.parse::<u32>().map_err(|cause| {
        return PayloadRefusal::At(
            line,
            PayloadRefusalKind::UnreadableNumber {
                field,
                value: value.to_owned(),
                cause: cause.to_string(),
            },
        );
    });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Observation;

    #[test]
    fn Test_Parse_Payload_Should_Read_A_Header_With_No_Items()
    {
        let payload = Parse_Payload(b"unexpanded\t0\n").expect("a header alone is well formed");

        assert_eq!(payload.unexpanded, 0);
        assert!(payload.items.is_empty());
    }

    #[test]
    fn Test_Parsed_Number_Should_Refuse_A_Value_That_Is_Not_A_Number()
    {
        assert_eq!(Parsed_Number("3", "ordinal", 1), Ok(3));

        let refused = Parsed_Number("many", "ordinal", 1).expect_err("not a number");
        assert!(
            matches!(
                refused,
                PayloadRefusal {
                    kind: PayloadRefusalKind::UnreadableNumber { field: "ordinal", .. },
                    line: Some(1),
                }
            ),
            "{refused:?}"
        );
    }

    #[test]
    fn Test_Expect_Fields_Should_Refuse_A_Record_Of_The_Wrong_Length()
    {
        assert_eq!(Expect_Fields("item", &["item", "0"], 2, 3), Ok(()));

        let refused = Expect_Fields("item", &["item", "0"], 7, 3).expect_err("two fields where seven are expected");
        assert!(
            matches!(
                refused,
                PayloadRefusal {
                    kind: PayloadRefusalKind::WrongFieldCount { ref tag, expected: 7, found: 2 },
                    line: Some(3),
                } if tag == "item"
            ),
            "{refused:?}"
        );
    }

    #[test]
    fn Test_Header_Count_Should_Refuse_A_Second_Header()
    {
        let fields: Vec<&str> = "unexpanded\t2".split('\t').collect();
        assert_eq!(Header_Count(&fields, 1, None), Ok(2));

        let refused = Header_Count(&fields, 2, Some(2)).expect_err("a header already arrived");
        assert!(
            matches!(
                refused,
                PayloadRefusal {
                    kind: PayloadRefusalKind::RepeatedHeader,
                    line: Some(2),
                }
            ),
            "{refused:?}"
        );
    }

    #[test]
    fn Test_Item_Record_Should_Read_Every_Field_In_Order()
    {
        let fields: Vec<&str> = "item\t0\tConstant\tPublic\tTABLES\t+doc\t+slice".split('\t').collect();
        let item = Item_Record(&fields, 1).expect("seven well-formed fields");

        assert_eq!(item.ordinal, 0);
        assert_eq!(item.kind, "Constant");
        assert_eq!(item.visibility, "Public");
        assert_eq!(item.qualified_name, "TABLES");
        assert_eq!(item.documentation, Observation::Present("doc".to_owned()));
        assert_eq!(item.shape, Observation::Present("slice".to_owned()));
    }

    #[test]
    fn Test_Read_Item_Should_Refuse_An_Item_Before_The_Header_Arrives()
    {
        let fields: Vec<&str> = "item\t0\tConstant\tPublic\tTABLES\t.\t.".split('\t').collect();
        let refused = Read_Item(&fields, 1, None).expect_err("no header has arrived yet");

        assert!(
            matches!(refused, PayloadRefusal { kind: PayloadRefusalKind::NoHeader, line: None }),
            "{refused:?}"
        );
    }

    #[test]
    fn Test_Read_Record_Should_Refuse_An_Unknown_Tag()
    {
        let mut read = Reading {
            unexpanded: Some(0),
            items: Vec::new(),
        };
        let refused = Read_Record("region\t0\t3", 2, &mut read).expect_err("this build does not know `region`");

        assert!(
            matches!(
                refused,
                PayloadRefusal {
                    kind: PayloadRefusalKind::UnknownRecord { ref tag },
                    line: Some(2),
                } if tag == "region"
            ),
            "{refused:?}"
        );
    }
}
