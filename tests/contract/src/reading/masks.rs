//! What every byte of a Rust file is, and the brace matching that depends on knowing.
//!
//! Brace matching without this counts the braces in `format!("{name}")` and desynchronises
//! on the first formatted panic message — of which this workspace has many, because a
//! failing assertion is required to name what it saw. Every scan in this crate reads
//! offsets through these masks rather than reading the text directly, which is what keeps
//! a `;` in a string and a `{` in a comment invisible to all of them.
//!
//! Byte-level and deliberately not a parser. This crate compiles against almost nothing on
//! purpose — an observer that linked the thing it observes would be a participant — so the
//! only thing available to read the workspace with is its text.

/// What each byte of a file is.
pub(crate) struct Masks
{
    /// Ordinary code: not a comment, and not inside a literal.
    pub(crate) code: Vec<bool>,
    /// Inside a line or block comment, the delimiters included.
    pub(crate) comment: Vec<bool>,
}

/// The byte after a block comment opening at `index`.
///
/// Rust nests block comments, so a depth counter rather than a search for the first `*/`.
fn Block_Comment_End(bytes: &[u8], index: usize) -> usize
{
    let mut cursor = index.saturating_add(2);
    let mut depth = 1_u32;
    while cursor < bytes.len() && depth > 0
    {
        let opening = bytes.get(cursor).copied().unwrap_or(0);
        let following = bytes.get(cursor.saturating_add(1)).copied().unwrap_or(0);
        if opening == b'/' && following == b'*'
        {
            depth = depth.saturating_add(1);
            cursor = cursor.saturating_add(2);
        }
        else if opening == b'*' && following == b'/'
        {
            depth = depth.saturating_sub(1);
            cursor = cursor.saturating_add(2);
        }
        else
        {
            cursor = cursor.saturating_add(1);
        }
    }

    return cursor;
}

/// The byte after an ordinary string literal opening at `index`.
fn String_Literal_End(bytes: &[u8], index: usize) -> usize
{
    let mut cursor = index.saturating_add(1);

    while let Some(byte) = bytes.get(cursor).copied()
    {
        if byte == b'\\'
        {
            cursor = cursor.saturating_add(2);
            continue;
        }
        cursor = cursor.saturating_add(1);
        if byte == b'"'
        {
            break;
        }
    }

    return cursor;
}

/// Classifies every byte of a file.
///
/// Brace matching without this counts the braces in `format!("{name}")` and desynchronises
/// on the first formatted panic message — of which this workspace has many, because a
/// failing assertion is required to name what it saw.
pub(crate) fn Scan(text: &str) -> Masks
{
    let bytes = text.as_bytes();
    let mut masks = Masks {
        code: vec![false; bytes.len()],
        comment: vec![false; bytes.len()],
    };
    let mut index = 0_usize;
    while index < bytes.len()
    {
        index = Classified(bytes, &mut masks, index);
    }

    return masks;
}

/// Classifies whatever begins at `index`, and answers where the next thing begins.
fn Classified(bytes: &[u8], masks: &mut Masks, index: usize) -> usize
{
    let current = bytes.get(index).copied().unwrap_or(0);
    let next = bytes.get(index.saturating_add(1)).copied().unwrap_or(0);
    if current == b'/' && next == b'/'
    {
        let end = Line_End(bytes, index);
        Mark(&mut masks.comment, index, end);

        return end;
    }
    if current == b'/' && next == b'*'
    {
        let end = Block_Comment_End(bytes, index);
        Mark(&mut masks.comment, index, end);

        return end;
    }
    if let Some(after) = Raw_String_End(bytes, index)
    {
        return after;
    }
    if current == b'"'
    {
        return String_Literal_End(bytes, index);
    }
    // A `'` that opens no literal is a lifetime, which is ordinary code.
    if current == b'\'' && let Some(after) = Character_Literal_End(bytes, index)
    {
        return after;
    }
    if let Some(slot) = masks.code.get_mut(index)
    {
        *slot = true;
    }

    return index.saturating_add(1);
}

/// The offset of the newline ending the line `index` sits on, or the end of the file.
fn Line_End(bytes: &[u8], index: usize) -> usize
{
    let mut cursor = index;
    while cursor < bytes.len() && bytes.get(cursor).copied() != Some(b'\n')
    {
        cursor = cursor.saturating_add(1);
    }

    return cursor;
}

/// Marks a half-open byte range.
fn Mark(mask: &mut [bool], from: usize, to: usize)
{
    let mut index = from;
    while index < to
    {
        if let Some(slot) = mask.get_mut(index)
        {
            *slot = true;
        }
        index = index.saturating_add(1);
    }
}

/// The file with every comment byte replaced by a space.
///
/// Byte for byte, so an offset taken from the original still means the same place here.
/// A comment is blanked whole, so a multi-byte character inside one never loses part of
/// itself and the result stays valid UTF-8.
pub(crate) fn Without_Comments(text: &str, comment: &[bool]) -> String
{
    let blanked: Vec<u8> = text
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if comment.get(index).copied().unwrap_or(false)
            {
                return b' ';
            }
            return *byte;
        })
        .collect();

    return String::from_utf8_lossy(&blanked).into_owned();
}

/// The offset just past a raw string starting at `index`, if one starts there.
///
/// Handles `r"..."`, `r#"..."#` and the `b`-prefixed byte forms. Returns `None` when the
/// `r` is part of an identifier, which is the common case — `for revision in ...`.
fn Raw_String_End(bytes: &[u8], index: usize) -> Option<usize>
{
    let opened = Raw_String_Opening(bytes, index)?;

    return Some(Raw_String_Close(bytes, opened));
}

/// Where a raw string's contents begin, and how many hashes have to close it.
#[derive(Clone, Copy)]
struct RawOpening
{
    from: usize,
    hashes: usize,
}

/// The opening of a raw string at `index`, if one opens there.
fn Raw_String_Opening(bytes: &[u8], index: usize) -> Option<RawOpening>
{
    if index > 0 && Is_Word_Byte(bytes, index.saturating_sub(1))
    {
        return None;
    }
    let mut cursor = index;
    if bytes.get(cursor).copied() == Some(b'b')
    {
        cursor = cursor.saturating_add(1);
    }
    if bytes.get(cursor).copied() != Some(b'r')
    {
        return None;
    }
    cursor = cursor.saturating_add(1);
    let mut hashes = 0_usize;
    while bytes.get(cursor).copied() == Some(b'#')
    {
        hashes = hashes.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }
    if bytes.get(cursor).copied() != Some(b'"')
    {
        return None;
    }

    return Some(RawOpening {
        from: cursor.saturating_add(1),
        hashes,
    });
}

/// The offset just past the quote and hashes that close a raw string.
///
/// The end of the file closes it. An unterminated raw string is a truncated source, and
/// reading the remainder as string content is what keeps the scan from finding code in it.
fn Raw_String_Close(bytes: &[u8], opened: RawOpening) -> usize
{
    let mut cursor = opened.from;
    while cursor < bytes.len()
    {
        if bytes.get(cursor).copied() == Some(b'"') && Hashes_Follow(bytes, cursor, opened.hashes)
        {
            return cursor.saturating_add(opened.hashes).saturating_add(1);
        }
        cursor = cursor.saturating_add(1);
    }

    return bytes.len();
}

/// Whether `wanted` hashes follow the quote at `cursor`.
fn Hashes_Follow(bytes: &[u8], cursor: usize, wanted: usize) -> bool
{
    let mut closing = 0_usize;
    while closing < wanted
        && bytes.get(cursor.saturating_add(closing).saturating_add(1)).copied() == Some(b'#')
    {
        closing = closing.saturating_add(1);
    }

    return closing == wanted;
}

/// The offset just past a character literal starting at `index`, if one starts there.
///
/// Returns `None` for a lifetime. The two are told apart by whether a closing quote
/// follows one character: `'a'` is a literal and `'a` is a lifetime.
fn Character_Literal_End(bytes: &[u8], index: usize) -> Option<usize>
{
    let first = bytes.get(index.saturating_add(1)).copied()?;

    if first == b'\\'
    {
        return Escaped_Literal_End(bytes, index);
    }

    let width = Utf8_Width(first);
    let closing = index.saturating_add(1).saturating_add(width);
    if bytes.get(closing).copied() == Some(b'\'')
    {
        return Some(closing.saturating_add(1));
    }

    return None;
}

/// The offset just past an escaped character literal such as `'\n'` or `'\u{1F600}'`.
///
/// An escape is at most `'\u{10FFFF}'`, so a quote further out than that closes something
/// else and there is no literal here.
fn Escaped_Literal_End(bytes: &[u8], index: usize) -> Option<usize>
{
    let mut cursor = index.saturating_add(2);
    let limit = index.saturating_add(12);

    while cursor <= limit
    {
        if bytes.get(cursor).copied()? == b'\''
        {
            return Some(cursor.saturating_add(1));
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// How many bytes the character beginning with this byte occupies.
const fn Utf8_Width(first: u8) -> usize
{
    if first < 0x80
    {
        return 1;
    }
    if first >> 5 == 0b110
    {
        return 2;
    }
    if first >> 4 == 0b1110
    {
        return 3;
    }

    return 4;
}

/// Whether a keyword starts at `index` and is a whole word.
pub(crate) fn Starts_Keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool
{
    for (offset, expected) in keyword.iter().enumerate()
    {
        if bytes.get(index.saturating_add(offset)).copied() != Some(*expected)
        {
            return false;
        }
    }
    if index > 0 && Is_Word_Byte(bytes, index.saturating_sub(1))
    {
        return false;
    }

    return !Is_Word_Byte(bytes, index.saturating_add(keyword.len()));
}

/// Whether the byte at an offset could continue an identifier.
///
/// A byte off either end of the slice cannot, so a keyword at the very start or the very end
/// of a file is a whole word.
fn Is_Word_Byte(bytes: &[u8], index: usize) -> bool
{
    let byte = bytes.get(index).copied().unwrap_or(b' ');

    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// The identifier starting at or after `from`, with the offset just past it.
pub(crate) fn Identifier_After(bytes: &[u8], from: usize) -> Option<(String, usize)>
{
    let mut cursor = from;
    while bytes.get(cursor).copied()?.is_ascii_whitespace()
    {
        cursor = cursor.saturating_add(1);
    }
    let start = cursor;
    while Is_Word_Byte(bytes, cursor)
    {
        cursor = cursor.saturating_add(1);
    }
    if cursor == start
    {
        return None;
    }

    let name = bytes
        .get(start..cursor)?
        .iter()
        .map(|byte| return char::from(*byte))
        .collect();

    return Some((name, cursor));
}

/// The first code offset at or after `from` holding `target`.
pub(crate) fn Next_Code_Byte(bytes: &[u8], mask: &[bool], from: usize, target: u8) -> Option<usize>
{
    let mut cursor = from;
    while cursor < bytes.len()
    {
        if Is_Code(mask, cursor) && bytes.get(cursor).copied() == Some(target)
        {
            return Some(cursor);
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// The offset of the brace closing the one at `open`.
pub(crate) fn Matching_Brace(bytes: &[u8], mask: &[bool], open: usize) -> Option<usize>
{
    return Matching_Delimiter(bytes, mask, open, Delimiter { opening: b'{', closing: b'}' });
}

/// The pair of bytes a depth walk counts: the byte that opens a level and the byte that
/// closes it. Grouped rather than passed separately, because the two are never meaningful
/// apart -- an opening byte with no closing byte to balance it counts nothing.
#[derive(Clone, Copy)]
pub(crate) struct Delimiter
{
    pub(crate) opening: u8,
    pub(crate) closing: u8,
}

/// The offset of the `delimiter.closing` byte that closes the `delimiter.opening` one at
/// `open`.
///
/// Braces and parentheses are counted by one depth walk rather than by two, because the
/// walk is the whole content of both: skip anything the mask says is not code, count the
/// pair, and stop where the depth returns to zero. Written twice, the two copies had to be
/// diffed to know they agreed, and a fix to either left the other wrong.
pub(crate) fn Matching_Delimiter(
    bytes: &[u8],
    mask: &[bool],
    open: usize,
    delimiter: Delimiter,
) -> Option<usize>
{
    let mut depth = 0_u32;
    let mut cursor = open;
    while cursor < bytes.len()
    {
        if !Is_Code(mask, cursor)
        {
            cursor = cursor.saturating_add(1);
            continue;
        }

        match bytes.get(cursor).copied()
        {
            Some(byte) if byte == delimiter.opening => depth = depth.saturating_add(1),
            Some(byte) if byte == delimiter.closing =>
            {
                depth = depth.saturating_sub(1);
                if depth == 0
                {
                    return Some(cursor);
                }
            }
            _ =>
            {}
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// Whether an offset is ordinary code.
pub(crate) fn Is_Code(mask: &[bool], index: usize) -> bool
{
    return mask.get(index).copied().unwrap_or(false);
}
