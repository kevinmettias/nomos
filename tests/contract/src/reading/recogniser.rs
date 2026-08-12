//! What kind of item begins at an offset, and where it ends.
//!
//! Separated from the walk that calls it because the two answer different questions. Nothing
//! here records anything: given a position it says what is written there and how far it
//! runs, and a caller decides whether that belongs to the surface.

use crate::reading::masks::{Is_Code, Matching_Brace};
use crate::reading::text::{Collapsed, Line_End};

/// One recognised declaration and where it ends.
pub(crate) struct Recognised
{
    /// The declaration with runs of whitespace collapsed, without the trailing `{` or `;`.
    pub(crate) text: String,
    /// The keyword naming what it declares.
    pub(crate) kind: String,
    /// Whatever preceded the keyword other than visibility.
    pub(crate) modifiers: String,
    /// The visibility as written: `pub`, `pub(crate)`, or empty.
    pub(crate) visibility: String,
    /// The body's byte range, when the declaration has one.
    pub(crate) body: Option<(usize, usize)>,
    /// The offset just past the whole declaration.
    pub(crate) after: usize,
}

/// Every keyword that opens an item this cares about.
pub(crate) const KEYWORDS: &[&str] = &[
    "fn", "struct", "enum", "trait", "type", "const", "static", "mod", "use", "impl", "union",
];

/// Reads a declaration starting at `from`, if one starts there.
pub(crate) fn Head(text: &str, masks: &crate::reading::masks::Masks, from: usize, end: usize) -> Option<Recognised>
{
    let line_end = Line_End(text, from, end);
    let line = text.get(from..line_end)?.trim_start();
    let indent = text.get(from..line_end)?.len().checked_sub(line.len())?;
    let begins = from.checked_add(indent)?;
    let (visibility, modifiers, kind) = Opening(line)?;
    let (terminator, at) = Ends_At(text, masks, (begins, end), &kind)?;
    let cut = Value_Cut(text, masks, (begins, at), &kind);
    let declaration = text.get(begins..cut)?;
    let body = Body_Range(text, masks, terminator, at);
    let after = body.map_or_else(
        || return at.saturating_add(1),
        |(_, close)| return close.saturating_add(1),
    );

    return Some(Recognised {
        text: Collapsed(declaration),
        kind,
        modifiers,
        visibility,
        body,
        after,
    });
}

/// Where a declaration ends: the first `{` or `;` outside parentheses, and which it was.
///
/// `where` clauses and return types cannot introduce either, and a body-bearing item always
/// reaches a brace before a semicolon.
///
/// `use` is the exception and the one that mattered: the braces in `use a::{B, C};` are a
/// list, not a body. Reading them as one truncated every grouped re-export in this workspace
/// to `pub use a::` and left the items behind them out of every snapshot — which is a surface
/// check reporting an empty surface.
fn Ends_At(
    text: &str,
    masks: &crate::reading::masks::Masks,
    range: (usize, usize),
    kind: &str,
) -> Option<(u8, usize)>
{
    let (begins, end) = range;
    if kind != "use"
    {
        return Terminator(text, masks, begins, end);
    }
    let at = Semicolon(text, masks, begins, end)?;

    return Some((b';', at));
}

/// Where a declaration's text stops being its surface.
///
/// A constant's value is not its surface. `SHIPPED` is a fourteen-entry table of
/// `include_str!` calls, and putting it in a snapshot would make every profile file rename
/// read as an API change.
fn Value_Cut(text: &str, masks: &crate::reading::masks::Masks, range: (usize, usize), kind: &str) -> usize
{
    let (begins, at) = range;
    if !matches!(kind, "const" | "static" | "type")
    {
        return at;
    }

    return Assignment(text, masks, begins, at).unwrap_or(at);
}

/// The byte range of a declaration's body, when the terminator opened one.
fn Body_Range(
    text: &str,
    masks: &crate::reading::masks::Masks,
    terminator: u8,
    at: usize,
) -> Option<(usize, usize)>
{
    if terminator != b'{'
    {
        return None;
    }
    let close = Matching_Brace(text.as_bytes(), &masks.code, at)?;

    return Some((at.saturating_add(1), close));
}

/// Splits a line's leading tokens into visibility, other modifiers and the item keyword.
pub(crate) fn Opening(line: &str) -> Option<(String, String, String)>
{
    let (visibility, rest) = Visibility(line)?;
    let words: Vec<&str> = rest
        .split_whitespace()
        .map(|token| return token.split(['<', '(', '!', ':']).next().unwrap_or(token))
        .collect();
    let mut modifiers: Vec<&str> = Vec::new();

    for (index, word) in words.iter().enumerate()
    {
        match Classify(&words, index, word)
        {
            Word::Keyword => return Some((visibility, modifiers.join(" "), (*word).to_owned())),
            Word::Modifier => modifiers.push(word),
            Word::Other => return None,
        }
    }

    return None;
}

/// The visibility a line opens with, and everything after it.
///
/// `None` for a `pub` glued to something else — `public`, `pubs` — which declares nothing
/// this reader is looking for.
pub(crate) fn Visibility(line: &str) -> Option<(String, &str)>
{
    let Some(after) = line.strip_prefix("pub")
    else
    {
        return Some((String::new(), line));
    };
    if let Some(restricted) = after.strip_prefix('(')
    {
        let close = restricted.find(')')?;
        let visibility = format!("pub({})", restricted.get(..close)?);

        return Some((visibility, restricted.get(close.saturating_add(1)..)?));
    }
    if after.starts_with(char::is_whitespace)
    {
        return Some(("pub".to_owned(), after));
    }

    return None;
}

/// What a leading word is: the keyword naming the item, a modifier in front of it, or
/// something that means this line declares nothing.
enum Word
{
    Keyword,
    Modifier,
    Other,
}

/// `const` is the one ambiguous word: `const NAME: T = …` declares an item and `const fn`
/// qualifies one. Reading it as the item keyword turned every `pub const fn` in this
/// workspace into a const named `fn`.
fn Classify(words: &[&str], index: usize, word: &str) -> Word
{
    if word == "const" && words.get(index.saturating_add(1)) == Some(&"fn")
    {
        return Word::Modifier;
    }
    if KEYWORDS.contains(&word)
    {
        return Word::Keyword;
    }
    if matches!(word, "unsafe" | "async" | "extern" | "default")
    {
        return Word::Modifier;
    }

    return Word::Other;
}

/// The `=` that starts a declaration's value, if it has one.
fn Assignment(text: &str, masks: &crate::reading::masks::Masks, from: usize, end: usize) -> Option<usize>
{
    let bytes = text.as_bytes();
    let mut cursor = from;

    while cursor < end
    {
        if Is_Code(&masks.code, cursor)
            && bytes.get(cursor).copied() == Some(b'=')
            // Not `==`, `>=`, `=>`. A declaration head holds none of them, and a
            // comparison would only appear in a value this is trying to cut away.
            && bytes.get(cursor.saturating_add(1)).copied() != Some(b'=')
            && !matches!(bytes.get(cursor.saturating_sub(1)).copied(), Some(b'=' | b'>' | b'<' | b'!'))
        {
            return Some(cursor);
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// The first `;` in code, whatever nests around it.
fn Semicolon(text: &str, masks: &crate::reading::masks::Masks, from: usize, end: usize) -> Option<usize>
{
    let bytes = text.as_bytes();
    let mut cursor = from;

    while cursor < end
    {
        if Is_Code(&masks.code, cursor) && bytes.get(cursor).copied() == Some(b';')
        {
            return Some(cursor);
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// The first `{` or `;` at parenthesis depth zero, with its offset.
fn Terminator(
    text: &str,
    masks: &crate::reading::masks::Masks,
    from: usize,
    end: usize,
) -> Option<(u8, usize)>
{
    let bytes = text.as_bytes();
    let mut depth = 0_i32;
    let mut cursor = from;

    while cursor < end
    {
        if Is_Code(&masks.code, cursor)
        {
            match bytes.get(cursor).copied()
            {
                Some(b'(' | b'[') => depth = depth.saturating_add(1),
                Some(b')' | b']') => depth = depth.saturating_sub(1),
                Some(found @ (b'{' | b';')) if depth == 0 => return Some((found, cursor)),
                _ =>
                {}
            }
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Restricted_Visibility_Should_Not_Be_Public()
    {
        assert_eq!(
            Opening("pub(crate) fn Thing()"),
            Some(("pub(crate)".to_owned(), String::new(), "fn".to_owned()))
        );
        assert_eq!(
            Opening("pub fn Thing()"),
            Some(("pub".to_owned(), String::new(), "fn".to_owned()))
        );
        assert_eq!(Opening("let public = 1;"), None);
    }
}
