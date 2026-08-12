//! Whether a declaration is exported, and how its name splits from its tail.
//!
//! `pub` is the whole question and it is not a keyword match: `pub(crate)`, `pub(super)`
//! and `pub(in …)` all begin with the same three bytes and none of them is public.

use crate::reading::declaration::items::{Filing, Source, Walk};
use std::collections::BTreeMap;
use crate::reading::module_tree::Module;

/// Whether a declaration carries `pub`.
///
/// Named rather than a bool. Both call sites below passed it among four or more
/// positional arguments, where `true` says nothing about what it is true of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Exported
{
    Yes,
    No,
}

impl Exported
{
    /// Whether a declaration's visibility is `pub`.
    pub(crate) const fn Of(is_public: bool) -> Self
    {
        if is_public
        {
            return Self::Yes;
        }

        return Self::No;
    }
}

/// An inline module as it was declared: its name, whether the declaration exports it, and
/// the byte range of its body.
#[derive(Clone, Copy)]
pub(crate) struct Inline<'a>
{
    pub(crate) name: &'a str,
    pub(crate) exported: Exported,
    pub(crate) body: (usize, usize),
}

/// An inline `mod name { … }`, loaded as its own module.
pub(crate) fn Load_Inline(
    inline: Inline<'_>,
    source: Source<'_>,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
)
{
    let Inline { name, exported, body } = inline;
    let mut child_path = path.to_vec();
    child_path.push(name.to_owned());

    let parent_exported = into.get(path).is_some_and(|held| return held.exported) || path.is_empty();
    let mut child = Module {
        exported: parent_exported && exported == Exported::Yes,
        ..Module::default()
    };
    let mut grandchildren = Vec::new();

    Walk(source, body, None, &mut Filing {
        module: &mut child,
        children: &mut grandchildren,
        path: &child_path,
        into,
    });

    into.insert(child_path, child);
}

/// The name a declaration gives, taken from just after its keyword.
pub(crate) fn Named(declaration: &str, keyword: &str) -> Option<String>
{
    use crate::reading::masks::Identifier_After;

    let at = Keyword_At(declaration, keyword)?;
    let from = at.checked_add(keyword.len())?;
    let (name, _) = Identifier_After(declaration.as_bytes(), from)?;

    return (!name.is_empty()).then_some(name);
}

/// Everything a declaration says after its name.
pub(crate) fn Tail(declaration: &str, keyword: &str, name: &str) -> String
{
    let Some(at) = Keyword_At(declaration, keyword)
    else
    {
        return String::new();
    };
    let Some(after) = declaration.get(at..)
    else
    {
        return String::new();
    };
    let Some(past_name) = after.split_once(name).map(|(_, rest)| return rest)
    else
    {
        return String::new();
    };

    return past_name.trim_end().to_owned();
}

/// Where a keyword appears as a whole word.
pub(crate) fn Keyword_At(declaration: &str, keyword: &str) -> Option<usize>
{
    let mut from = 0_usize;

    while let Some(offset) = declaration.get(from..)?.find(keyword)
    {
        let at = from.checked_add(offset)?;
        if Whole_Word(declaration, at, keyword.len())
        {
            return Some(at);
        }
        from = at.saturating_add(1);
    }

    return None;
}

/// Whether the `length` bytes at `at` are bounded by something that cannot continue an
/// identifier, which is what makes an occurrence the word rather than part of one.
fn Whole_Word(declaration: &str, at: usize, length: usize) -> bool
{
    let bytes = declaration.as_bytes();
    let before = at
        .checked_sub(1)
        .and_then(|index| return bytes.get(index).copied())
        .unwrap_or(b' ');
    let after = bytes.get(at.saturating_add(length)).copied().unwrap_or(b' ');

    return !before.is_ascii_alphanumeric()
        && before != b'_'
        && !after.is_ascii_alphanumeric()
        && after != b'_';
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Declaration_Should_Split_Into_A_Name_And_A_Tail()
    {
        let declaration = "pub fn Build(store: &Store, profile: &Profile) -> Result<Output, E>";

        assert_eq!(Named(declaration, "fn"), Some("Build".to_owned()));
        assert_eq!(
            Tail(declaration, "fn", "Build"),
            "(store: &Store, profile: &Profile) -> Result<Output, E>".to_owned()
        );
    }
}
