//! What a line of Rust *says* about the module tree: the `mod name;` declaration, and the
//! `#[path = "…"]` attribute that redirects where one resolves.
//!
//! Split out of `orphan_modules.rs` because the two answer different questions. This module
//! knows what a declaration looks like; the walk next door knows which collected file a
//! declaration reaches. A reader asking either question should not have to read the other,
//! and the split is also what keeps the walk under the file-size review trigger.
//!
//! Every reader here is a pure function of one line of code. They deliberately do not share
//! the crate's cross-line literal scanner: each is asked about a line the walk has already
//! masked, so a `mod` inside a string never reaches them.

/// The prefix that lets a keyword be spelled as an identifier. It is spelling, not name:
/// `mod r#match;` is backed by `match.rs`.
const RAW_IDENTIFIER_PREFIX: &str = "r#";

/// The declaration keyword this rule resolves, and the attribute that redirects it.
const MODULE_KEYWORD: &str = "mod";
const PATH_ATTRIBUTE_NAME: &str = "path";

/// The two bytes that open an attribute.
const ATTRIBUTE_OPENING_LENGTH: usize = 2;

/// The module name a line declares with `mod name;`, or `None` for any other line.
///
/// The terminating semicolon is required, which is what excludes an inline module: one
/// written with a body is backed by no file and so declares nothing on disk.
pub(super) fn Declared_Module_Name(code: &str) -> Option<&str>
{
    let after_visibility = Without_Visibility(code.trim_start()).trim_start();
    let after_keyword = Without_Module_Keyword(after_visibility)?;
    let after_prefix = after_keyword.strip_prefix(RAW_IDENTIFIER_PREFIX).unwrap_or(after_keyword);
    let (name, remainder) = Leading_Identifier(after_prefix)?;

    if !remainder.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(name);
}

/// `code` with a leading `pub` or `pub(…)` removed, unchanged when it carries neither.
fn Without_Visibility(code: &str) -> &str
{
    let Some(after_visibility) = code.strip_prefix("pub")
    else
    {
        return code;
    };

    if let Some(after_open) = after_visibility.trim_start().strip_prefix('(')
        && let Some(close) = after_open.find(')')
    {
        return after_open.get(close.saturating_add(1)..).unwrap_or("");
    }

    if !after_visibility.starts_with(char::is_whitespace)
    {
        return code;
    }

    return after_visibility;
}

/// `code` past a leading declaration keyword and the whitespace after it, or `None` when it
/// does not start with the keyword — an identifier merely beginning with those letters must
/// not read as one.
fn Without_Module_Keyword(code: &str) -> Option<&str>
{
    let after_keyword = code.strip_prefix(MODULE_KEYWORD)?;

    if !after_keyword.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after_keyword.trim_start());
}

/// The identifier `code` starts with, paired with everything after it.
fn Leading_Identifier(code: &str) -> Option<(&str, &str)>
{
    let first = code.chars().next()?;

    if !first.is_ascii_alphabetic() && first != '_'
    {
        return None;
    }

    let end = code
        .find(|character: char| return !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(code.len());

    return Some((code.get(..end)?, code.get(end..)?));
}

/// The file a path attribute on this line names, or `None` when the line carries no complete
/// attribute.
pub(super) fn Declared_Path_Attribute(code: &str) -> Option<&str>
{
    let opened = code.find("#[").and_then(|start| return code.get(start.saturating_add(ATTRIBUTE_OPENING_LENGTH)..))?;
    let after_name = opened.trim_start().strip_prefix(PATH_ATTRIBUTE_NAME)?;
    let after_equals = after_name.trim_start().strip_prefix('=')?;
    let quoted = after_equals.trim_start().strip_prefix('"')?;
    let end = quoted.find('"')?;
    let after_quote = quoted.get(end.saturating_add(1)..)?;

    if !after_quote.trim_start().starts_with(']')
    {
        return None;
    }

    return quoted.get(..end);
}
