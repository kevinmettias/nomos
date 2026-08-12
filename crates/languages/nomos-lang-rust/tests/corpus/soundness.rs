//! Checking a reported name against the file it was read from.
//!
//! Soundness is a membership question — does this name occur as an identifier in that
//! source — so the identifiers are gathered once per file rather than scanned per item.
//! Over this corpus that is the difference between a test that runs and one nobody waits
//! for.

use nomos_lang_rust::{Read_Source, Reading, SyntaxItem};
use std::collections::BTreeSet;
use std::path::Path;

/// Name segments that are the provider declining to name something.
///
/// `*` is a glob import and `_` is a type with no single head. Neither is a claim that an
/// identifier of that spelling appears in the file, so neither is checked against one.
const NAMELESS: &[&str] = &["*", "_"];

/// Every identifier-shaped token in a source file.
///
/// `#` is part of a token because a raw identifier is spelled `r#match`, and the provider
/// reports it that way — the name as written, which is the whole stance of a syntactic
/// reader. The corpus is what surfaced this: `xvpe-collections` declares `mod r#match;`,
/// and a tokenizer that split on `#` reported the provider unsound for saying exactly
/// what the file says.
fn Identifiers(source: &str) -> BTreeSet<&str>
{
    return source
        .split(|character: char| {
            return !character.is_alphanumeric() && character != '_' && character != '#';
        })
        .filter(|token| return !token.is_empty())
        .collect();
}

/// How many names one file's facts were checked against its own identifiers.
///
/// `None` for a file this walk could not read or the provider would not parse. Those are
/// counted elsewhere and neither is a soundness failure.
pub(crate) fn Names_Checked(path: &Path) -> Option<u64>
{
    let Ok(source) = std::fs::read_to_string(path)
    else
    {
        return None;
    };
    let Reading::Parsed(facts) = Read_Source(&source)
    else
    {
        return None;
    };
    let identifiers = Identifiers(&source);
    let mut checked = 0_u64;

    for item in &facts.items
    {
        let named = Assert_Names_Occur(path, item, &identifiers);

        checked = checked.saturating_add(named);
    }

    return Some(checked);
}

/// Every name segment one item reports occurs as an identifier in the file it came from.
///
/// `*` for a glob import and `_` for a type with no single head are this provider saying it
/// has no name, not names it claims to have found.
fn Assert_Names_Occur(path: &Path, item: &SyntaxItem, identifiers: &BTreeSet<&str>) -> u64
{
    let named = item
        .name
        .split("::")
        .filter(|segment| return !segment.is_empty() && !NAMELESS.contains(segment));
    let mut checked = 0_u64;

    for segment in named
    {
        assert!(
            identifiers.contains(segment),
            "{} reports `{}` ({}) and `{segment}` is not an identifier in that file",
            path.display(),
            item.Qualified_Name(),
            item.kind
        );
        checked = checked.saturating_add(1);
    }

    return checked;
}
