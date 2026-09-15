//! Reading the two statement shapes all three facade rules are written over.
//!
//! `pub mod <child>;` and `pub use <child>::<item>;`, read line-local over comment-stripped
//! text. Grouped apart from the three rules in [`super`] the way every other reading half in
//! this crate is: none of this has an opinion about what a facade *should* publish, and none
//! of the rules has an opinion about how a statement is spelled.

/// A parsed single-item public re-export.
pub(super) struct ReExport<'a>
{
    /// The path segments, raw-identifier prefixes removed.
    pub(super) path: Vec<&'a str>,
    /// The alias, when the statement renames what it publishes.
    pub(super) alias: Option<&'a str>,
}

/// The path and optional alias of a single-item public re-export. A brace group publishes a
/// surface this port does not model and is not one.
pub(super) fn Re_Export(code: &str) -> Option<ReExport<'_>>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "use")?;
    let rest = after_keyword.strip_prefix("self::").unwrap_or(after_keyword);

    let (path, rest) = Path_Segments(rest)?;
    let trailing = rest.trim_start();

    if let Some(after_as) = Keyword_Body(trailing, "as")
    {
        return Aliased_Re_Export(path, after_as);
    }

    if !trailing.starts_with(';')
    {
        return None;
    }

    return Some(ReExport { path, alias: None });
}

/// The `::`-separated leading path segments of `text`, and whatever follows the last one --
/// a raw-identifier prefix removed from each segment, the same as [`Leading_Identifier`].
fn Path_Segments(text: &str) -> Option<(Vec<&str>, &str)>
{
    let mut path = Vec::new();
    let mut rest = text;

    return loop
           {
        let (segment, after_segment) = Leading_Identifier(rest)?;
        path.push(segment);

        let Some(after_colons) = after_segment.strip_prefix("::")
        else
        {
            break Some((path, after_segment));
        };
        rest = after_colons;
           };
}

/// The re-export `path` renames to whatever leads `after_as` -- the text right after the
/// `as` keyword -- or `None` if that alias does not close on this line.
fn Aliased_Re_Export<'a>(path: Vec<&'a str>, after_as: &'a str) -> Option<ReExport<'a>>
{
    let (alias, after_alias) = Leading_Identifier(after_as)?;
    if !after_alias.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(ReExport { path, alias: Some(alias) });
}

/// `code` with its leading whitespace and its `pub` or restricted-`pub` visibility removed.
/// Restricted visibility counts: `facade-chooses-flattening-or-namespace` says a facade is a
/// boundary even when the boundary is only inside the crate.
pub(super) fn After_Public_Visibility(code: &str) -> Option<&str>
{
    let trimmed = code.trim_start();
    let after_pub = trimmed.strip_prefix("pub")?;

    let after_restriction = if let Some(inside) = after_pub.strip_prefix('(')
                            {
        let close = inside.find(')')?;
        inside.get(close.saturating_add(1)..)?
                            }
    else
                            {
        after_pub
                            };

    if !after_restriction.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after_restriction.trim_start());
}

/// What follows `keyword` in `text`, requiring whitespace after it so `used` is never read
/// as `use`.
pub(super) fn Keyword_Body<'a>(text: &'a str, keyword: &str) -> Option<&'a str>
{
    let after = text.strip_prefix(keyword)?;

    if !after.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after.trim_start());
}

/// `text`'s leading Rust identifier -- any raw-identifier prefix removed, since `r#match`
/// and `match` name one module -- and whatever follows it.
pub(super) fn Leading_Identifier(text: &str) -> Option<(&str, &str)>
{
    let body = text.strip_prefix("r#").unwrap_or(text);
    let end = Leading_Identifier_Byte_Length(body);

    if end == 0
    {
        return None;
    }

    return Some((body.get(..end)?, body.get(end..)?));
}

/// The byte length of the leading Rust identifier characters in `body`: ASCII-alphabetic or
/// `_` first, then ASCII-alphanumeric or `_`.
fn Leading_Identifier_Byte_Length(body: &str) -> usize
{
    let mut end = 0usize;

    for (offset, character) in body.char_indices()
    {
        let position = Identifier_Position_At(offset);
        if !Is_Acceptable_Identifier_Character(character, position)
        {
            break;
        }
        end = offset.saturating_add(character.len_utf8());
    }

    return end;
}

/// Where a character sits in an identifier — the first character allows a narrower set
/// (no digits) than the rest, so a caller cannot silently pass the wrong test the way a
/// bare `bool` invites.
enum IdentifierPosition
{
    First,
    Rest,
}

/// Where the byte at `offset` sits in an identifier: only the first byte is barred from
/// starting with a digit.
fn Identifier_Position_At(offset: usize) -> IdentifierPosition
{
    if offset == 0
    {
        return IdentifierPosition::First;
    }

    return IdentifierPosition::Rest;
}

fn Is_Acceptable_Identifier_Character(character: char, position: IdentifierPosition) -> bool
{
    return match position
           {
        IdentifierPosition::First => character.is_ascii_alphabetic() || character == '_',
        IdentifierPosition::Rest => character.is_ascii_alphanumeric() || character == '_',
           };
}
