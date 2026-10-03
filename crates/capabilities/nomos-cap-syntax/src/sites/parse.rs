//! Reading a sites payload back, and refusing one that does not say what it claims.

use super::{KindDecline, SiteKind, SiteRecord, SiteValue, SiteValueType, Site_Kind, SitesPayload, SitesRefusal, SitesRefusalKind};
use crate::payload::Unescape_Field;
use std::collections::BTreeMap;

/// Fields in an `offers` stance, tag included.
const OFFER_FIELDS: usize = 2;
/// Fields in a `declines` stance, tag included.
const DECLINE_FIELDS: usize = 3;
/// Fields every `site` record carries before its kind's own: the tag, the kind and the line.
const SITE_HEAD_FIELDS: usize = 3;

/// Reads a sites payload, or refuses it.
///
/// Never a partial answer, for the reason [`crate::Parse_Payload`] gives: a payload half of which
/// decodes is a file judged on part of what it holds.
///
/// # Errors
///
/// [`SitesRefusal`] for anything that is not a well-formed payload by the grammar in
/// `src/sites.rs`'s documentation -- including a site of a kind nobody declared, a field its
/// kind does not declare, and a declared field it does not carry.
pub fn Parse_Sites_Payload(bytes: &[u8]) -> Result<SitesPayload, SitesRefusal>
{
    let Ok(text) = core::str::from_utf8(bytes)
    else
    {
        return Err(SitesRefusal::Whole(SitesRefusalKind::NotUtf8));
    };

    let mut payload = SitesPayload { offered: Vec::new(), declined: Vec::new(), records: Vec::new() };
    let mut site_lines = Vec::new();
    for (index, record) in Records(text).enumerate()
    {
        let line = index.saturating_add(1);
        if Read_Record(record, line, &mut payload)?
        {
            site_lines.push(line);
        }
    }

    Refuse_Unoffered_Sites(&payload, &site_lines)?;

    return Ok(payload);
}

/// The records of `text`, without the empty piece after the final `\n`.
fn Records(text: &str) -> impl Iterator<Item = &str>
{
    let body = text.strip_suffix('\n').unwrap_or(text);
    return body.split('\n').filter(move |_| return !text.is_empty());
}

/// Reads one record into `payload`, answering whether it was a site.
fn Read_Record(record: &str, line: usize, payload: &mut SitesPayload) -> Result<bool, SitesRefusal>
{
    let fields: Vec<&str> = record.split('\t').collect();

    return match fields.first().copied().unwrap_or_default()
    {
        "offers" => Read_Offer(&fields, line, payload).map(|()| return false),
        "declines" => Read_Decline(&fields, line, payload).map(|()| return false),
        "site" => Read_Site(&fields, line, payload).map(|()| return true),
        tag => Err(SitesRefusal::At(line, SitesRefusalKind::UnknownRecord { tag: tag.to_owned() })),
    };
}

fn Read_Offer(fields: &[&str], line: usize, payload: &mut SitesPayload) -> Result<(), SitesRefusal>
{
    Expect_Fields(fields, OFFER_FIELDS, line)?;
    let kind = Stanced_Kind(fields, line, payload)?;

    payload.offered.push(kind.name.to_owned());
    return Ok(());
}

fn Read_Decline(fields: &[&str], line: usize, payload: &mut SitesPayload) -> Result<(), SitesRefusal>
{
    Expect_Fields(fields, DECLINE_FIELDS, line)?;
    let kind = Stanced_Kind(fields, line, payload)?;
    let reason = Unescape_Field(fields.get(2).copied().unwrap_or_default());
    if reason.is_empty()
    {
        return Err(SitesRefusal::At(line, SitesRefusalKind::UnexplainedDecline { kind: kind.name.to_owned() }));
    }

    payload.declined.push(KindDecline { kind: kind.name.to_owned(), reason });
    return Ok(());
}

fn Read_Site(fields: &[&str], line: usize, payload: &mut SitesPayload) -> Result<(), SitesRefusal>
{
    if fields.len() < SITE_HEAD_FIELDS
    {
        return Err(Wrong_Field_Count(fields, SITE_HEAD_FIELDS, line));
    }
    let kind = Declared_Kind(fields.get(1).copied().unwrap_or_default(), line)?;
    let at = fields.get(2).copied().unwrap_or_default();
    let site_line = at.parse::<usize>().map_err(|error| {
        return SitesRefusal::At(line, SitesRefusalKind::UnreadableLine { value: at.to_owned(), cause: error.to_string() });
    })?;

    let mut values = BTreeMap::new();
    for field in fields.get(SITE_HEAD_FIELDS..).unwrap_or_default()
    {
        Read_Field(kind, field, line, &mut values)?;
    }
    if let Some(missing) = kind.fields.iter().find(|declared| return !values.contains_key(declared.name))
    {
        return Err(SitesRefusal::At(line, SitesRefusalKind::MissingField { kind: kind.name.to_owned(), field: missing.name.to_owned() }));
    }

    payload.records.push(SiteRecord { kind: kind.name.to_owned(), line: site_line, values });
    return Ok(());
}

/// Reads one `name=value` field of a site of `kind` into `values`.
fn Read_Field(kind: &SiteKind, field: &str, line: usize, values: &mut BTreeMap<String, SiteValue>) -> Result<(), SitesRefusal>
{
    let refuse = |refusal| return SitesRefusal::At(line, refusal);
    let Some((name, raw)) = field.split_once('=')
    else
    {
        return Err(refuse(SitesRefusalKind::MalformedField { field: field.to_owned() }));
    };
    let Some(declared) = kind.Field(name)
    else
    {
        return Err(refuse(SitesRefusalKind::UndeclaredField { kind: kind.name.to_owned(), field: name.to_owned() }));
    };
    if values.contains_key(name)
    {
        return Err(refuse(SitesRefusalKind::RepeatedField { kind: kind.name.to_owned(), field: name.to_owned() }));
    }
    let Some(value) = Decode_Value(raw, declared.value)
    else
    {
        return Err(refuse(SitesRefusalKind::UnreadableValue { field: name.to_owned(), value: raw.to_owned(), expected: declared.value }));
    };

    values.insert(name.to_owned(), value);
    return Ok(());
}

/// One value, read as the type its field declares, or `None` if it is not that type.
fn Decode_Value(raw: &str, declared: SiteValueType) -> Option<SiteValue>
{
    return match declared
    {
        SiteValueType::Text => Some(SiteValue::Text(Unescape_Field(raw))),
        SiteValueType::Integer => raw.parse::<i64>().ok().map(SiteValue::Integer),
        SiteValueType::Truth => match raw
        {
            "true" => Some(SiteValue::Truth(true)),
            "false" => Some(SiteValue::Truth(false)),
            _ => None,
        },
        SiteValueType::TextList => Split_List(raw).map(SiteValue::TextList),
    };
}

/// A list of text read back: every item ends at an unescaped `;`, and a value whose last item has
/// no `;` after it is not a list.
fn Split_List(raw: &str) -> Option<Vec<String>>
{
    let mut items = Vec::new();
    let mut item = String::new();
    let mut characters = raw.chars();

    while let Some(character) = characters.next()
    {
        match character
        {
            ';' => items.push(Unescape_Field(&core::mem::take(&mut item))),
            '\\' => Push_List_Escape(&mut item, characters.next()),
            other => item.push(other),
        }
    }

    return item.is_empty().then_some(items);
}

/// Keeps an escape for [`Unescape_Field`] to read, except `\;`, which only a list item has and
/// which stands for the separator itself.
fn Push_List_Escape(item: &mut String, escaped: Option<char>)
{
    match escaped
    {
        Some(';') => item.push(';'),
        Some(other) =>
        {
            item.push('\\');
            item.push(other);
        }
        None => item.push('\\'),
    }
}

/// The declared kind a stance names, refused if no declaration names it or the payload already
/// states a stance on it.
fn Stanced_Kind(fields: &[&str], line: usize, payload: &SitesPayload) -> Result<&'static SiteKind, SitesRefusal>
{
    let kind = Declared_Kind(fields.get(1).copied().unwrap_or_default(), line)?;
    let stanced = payload.offered.iter().any(|offered| return offered == kind.name) || payload.declined.iter().any(|decline| return decline.kind == kind.name);
    if stanced
    {
        return Err(SitesRefusal::At(line, SitesRefusalKind::RepeatedStance { kind: kind.name.to_owned() }));
    }

    return Ok(kind);
}

fn Declared_Kind(name: &str, line: usize) -> Result<&'static SiteKind, SitesRefusal>
{
    return Site_Kind(name).ok_or_else(|| return SitesRefusal::At(line, SitesRefusalKind::UndeclaredKind { kind: name.to_owned() }));
}

/// Refuses the first site whose kind the payload does not offer.
fn Refuse_Unoffered_Sites(payload: &SitesPayload, site_lines: &[usize]) -> Result<(), SitesRefusal>
{
    for (record, line) in payload.records.iter().zip(site_lines)
    {
        if !payload.offered.contains(&record.kind)
        {
            return Err(SitesRefusal::At(*line, SitesRefusalKind::UnofferedSite { kind: record.kind.clone() }));
        }
    }

    return Ok(());
}

fn Expect_Fields(fields: &[&str], expected: usize, line: usize) -> Result<(), SitesRefusal>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(Wrong_Field_Count(fields, expected, line));
}

fn Wrong_Field_Count(fields: &[&str], expected: usize, line: usize) -> SitesRefusal
{
    let tag = fields.first().copied().unwrap_or_default().to_owned();
    return SitesRefusal::At(line, SitesRefusalKind::WrongFieldCount { tag, expected, found: fields.len() });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Split_List_Should_Tell_The_Empty_List_From_One_Empty_Item()
    {
        assert_eq!(Split_List(""), Some(Vec::new()));
        assert_eq!(Split_List(";"), Some(vec![String::new()]));
    }

    #[test]
    fn Test_Split_List_Should_Read_An_Escaped_Separator_And_An_Escaped_Backslash()
    {
        assert_eq!(Split_List(r"a\;b;c\\;"), Some(vec!["a;b".to_owned(), "c\\".to_owned()]));
    }

    #[test]
    fn Test_Split_List_Should_Refuse_An_Unterminated_Item()
    {
        assert_eq!(Split_List("a;b"), None);
        assert_eq!(Decode_Value("a;b", SiteValueType::TextList), None);
    }
}
