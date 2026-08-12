//! The members inside a `struct`, an `enum`, a `trait` or an `impl` block.
//!
//! A public field and a public variant are part of the surface, so a type recorded without
//! them is a surface that under-reports. Reading them means bracket matching over a payload
//! rather than splitting on lines, because a tuple variant's payload can carry a comma
//! inside a generic argument.

use crate::reading::masks::{Identifier_After, Is_Code, Matching_Delimiter};
use crate::reading::text::{Collapsed, Line_End, Next_Line};

/// Which of the two bodies is being read.
///
/// Named rather than a bool. `Members_In(text, masks, range, true)` said nothing at the
/// call site about which body it was reading, and the two read a line differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reading
{
    /// A struct's fields, each carrying its own visibility.
    Fields,
    /// An enum's variants, as public as the enum in front of them.
    Variants,
}

/// The members of an enum or struct body, with whatever each one carries.
///
/// Read line by line to find each member and then by brace matching to capture what it
/// carries, because this workspace writes Allman braces: a struct variant's fields are on
/// the lines *after* its name, so a purely line-wise reader records `NotRelative` and
/// drops `{ profile: String, output: String }`. A payload's types are as much of the
/// surface as the name in front of them.
///
/// [`Reading`] separates the two callers. A struct's fields are public one at a time and
/// a private one is not an export; an enum's variants are as public as the enum.
pub(crate) fn Members_In(
    text: &str,
    masks: &crate::reading::masks::Masks,
    range: (usize, usize),
    reading: Reading,
) -> Vec<(String, String)>
{
    let (start, end) = range;
    let mut found = Vec::new();
    let mut cursor = start;

    while cursor < end
    {
        let read = Read_Member(text, masks, (cursor, end), reading);

        found.extend(read.member);
        cursor = read.next;
    }

    return found;
}

/// One line of a member body: whatever member it declares, and where the next line begins.
struct Line
{
    pub(crate) member: Option<(String, String)>,
    pub(crate) next: usize,
}

/// Reads the line beginning at `range.0`.
fn Read_Member(
    text: &str,
    masks: &crate::reading::masks::Masks,
    range: (usize, usize),
    reading: Reading,
) -> Line
{
    let (cursor, end) = range;
    let line_end = Line_End(text, cursor, end);
    let next_line = Next_Line(text, line_end, end);
    let Some(named) = Named_Member(text, (cursor, line_end), reading)
    else
    {
        return Line {
            member: None,
            next: next_line,
        };
    };
    let Carried { payload, after } = Payload(text, masks, named.after, end);

    return Line {
        member: Some((named.name, payload)),
        next: after.max(next_line).min(end),
    };
}

/// A member's name, and the offset just past it.
pub(crate) struct NamedMember
{
    pub(crate) name: String,
    pub(crate) after: usize,
}

/// The member a line declares, if it declares one.
///
/// A member is a bare identifier at the start of its line. Attributes open with `#`, the
/// closing brace with `}`, and a nested item with a keyword. A struct's fields are public one
/// at a time and a private one is not an export; an enum's variants are as public as the enum
/// in front of them.
pub(crate) fn Named_Member(text: &str, line: (usize, usize), reading: Reading) -> Option<NamedMember>
{
    use crate::reading::recogniser::KEYWORDS;

    let (cursor, line_end) = line;
    let raw = text.get(cursor..line_end).unwrap_or_default();
    let trimmed = raw.trim_start();
    let at = cursor.saturating_add(raw.len().saturating_sub(trimmed.len()));
    let (rest, from) = match reading
    {
        Reading::Fields => (trimmed.strip_prefix("pub ")?, at.saturating_add(4)),
        Reading::Variants => (trimmed, at),
    };
    let (name, after) = Identifier_After(text.as_bytes(), from)?;

    if !rest.starts_with(&name) || KEYWORDS.contains(&name.as_str())
    {
        return None;
    }

    return Some(NamedMember { name, after });
}

/// What a member carries, and where the next one starts.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct Carried
{
    pub(crate) payload: String,
    pub(crate) after: usize,
}

/// What a member carries, and where the next one starts.
fn Payload(
    text: &str,
    masks: &crate::reading::masks::Masks,
    from: usize,
    end: usize,
) -> Carried
{
    let bytes = text.as_bytes();
    let mut cursor = from;
    while cursor < end && bytes.get(cursor).is_some_and(u8::is_ascii_whitespace)
    {
        cursor = cursor.saturating_add(1);
    }
    let opener = bytes.get(cursor).copied();

    // A field's type, up to the comma that ends it. Depth-counted, so
    // `BTreeMap<String, u32>` is one type rather than two fields.
    if opener == Some(b':')
    {
        let close = Comma_At_Depth(bytes, &masks.code, cursor, end);
        let carried = text.get(cursor..close).map(Collapsed).unwrap_or_default();

        return Carried {
            payload: carried,
            after: close.saturating_add(1),
        };
    }

    return Bracketed(text, masks, (cursor, end), opener);
}

/// What a member carries between brackets, and where the next one starts.
///
/// A variant's fields are on the lines after its name, so the payload is taken by matching
/// the bracket rather than by reading to the end of the line.
fn Bracketed(
    text: &str,
    masks: &crate::reading::masks::Masks,
    range: (usize, usize),
    opener: Option<u8>,
) -> Carried
{
    let (cursor, end) = range;
    let bytes = text.as_bytes();
    let matched = match opener
    {
        Some(b'{') => Matching_Delimiter(bytes, &masks.code, cursor, b'{', b'}'),
        Some(b'(') => Matching_Delimiter(bytes, &masks.code, cursor, b'(', b')'),
        _ =>
        {
            return Carried {
                payload: String::new(),
                after: cursor.saturating_add(1),
            };
        }
    };
    let Some(close) = matched
    else
    {
        return Carried {
            payload: String::new(),
            after: end,
        };
    };
    let carried = text.get(cursor..=close).map(Collapsed).unwrap_or_default();
    let separator = if opener == Some(b'{') { " " } else { "" };

    return Carried {
        payload: format!("{separator}{carried}"),
        after: close.saturating_add(1),
    };
}

/// The first comma outside any bracket, or the end of the range.
fn Comma_At_Depth(bytes: &[u8], mask: &[bool], from: usize, end: usize) -> usize
{
    let mut depth = 0_i32;
    let mut cursor = from;

    while cursor < end
    {
        if Is_Code(mask, cursor)
        {
            match bytes.get(cursor).copied()
            {
                Some(b'(' | b'[' | b'<' | b'{') => depth = depth.saturating_add(1),
                Some(b')' | b']' | b'>' | b'}') => depth = depth.saturating_sub(1),
                Some(b',') if depth == 0 => return cursor,
                _ =>
                {}
            }
        }
        cursor = cursor.saturating_add(1);
    }

    return end;
}

