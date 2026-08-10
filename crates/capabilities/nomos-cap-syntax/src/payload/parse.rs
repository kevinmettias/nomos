//! Reading a payload back, and refusing one that does not say what it claims.

use super::{SyntaxPayload, PayloadRefusal, PayloadItem, Observed};

/// Fields in an `item` record, tag included.
const ITEM_FIELDS: usize = 7;

/// Fields in the `unexpanded` header, tag included.
const HEADER_FIELDS: usize = 2;

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
        return Err(PayloadRefusal::NotUtf8);
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
            return Err(PayloadRefusal::NoHeader);
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
        "unexpanded" => read.unexpanded = Some(Header(&fields, at, read.unexpanded)?),
        "item" =>
        {
            let item = Read_Item(&fields, at, read.unexpanded)?;
            read.items.push(item);
        }
        other =>
        {
            return Err(PayloadRefusal::UnknownRecord {
                tag: other.to_owned(),
                line: at,
            });
        }
    }

    return Ok(());
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
        return Err(PayloadRefusal::NoHeader);
    }
    Expect_Fields("item", fields, ITEM_FIELDS, at)?;

    return Item(fields, at);
}

/// The header's count, refusing a second one.
///
/// A payload carrying two headers does not say which count is its own, and taking either
/// would be inventing an answer the bytes do not give.
pub(super) fn Header(fields: &[&str], at: usize, already: Option<u32>) -> Result<u32, PayloadRefusal>
{
    if already.is_some()
    {
        return Err(PayloadRefusal::RepeatedHeader { line: at });
    }
    Expect_Fields("unexpanded", fields, HEADER_FIELDS, at)?;

    return Number(fields.get(1).copied().unwrap_or_default(), "unexpanded", at);
}

/// One item record, with both observation-bearing fields read as observations.
pub(super) fn Item(fields: &[&str], at: usize) -> Result<PayloadItem, PayloadRefusal>
{
    return Ok(PayloadItem {
        ordinal: Number(fields.get(1).copied().unwrap_or_default(), "ordinal", at)?,
        kind: fields.get(2).copied().unwrap_or_default().to_owned(),
        visibility: fields.get(3).copied().unwrap_or_default().to_owned(),
        qualified_name: fields.get(4).copied().unwrap_or_default().to_owned(),
        documentation: Observed(
            fields.get(5).copied().unwrap_or_default(),
            "documentation",
            at,
        )?,
        shape: Observed(fields.get(6).copied().unwrap_or_default(), "shape", at)?,
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

    return Err(PayloadRefusal::WrongFieldCount {
        tag: tag.to_owned(),
        expected,
        found: fields.len(),
        line,
    });
}

/// Reads a field that must be a number.
pub(super) fn Number(value: &str, field: &'static str, line: usize) -> Result<u32, PayloadRefusal>
{
    return value.parse::<u32>().map_err(|_| {
        return PayloadRefusal::UnreadableNumber {
            field,
            value: value.to_owned(),
            line,
        };
    });
}
