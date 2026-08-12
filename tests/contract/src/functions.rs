//! Every function a file defines, with the text of its body.
//!
//! What a definition *is* rather than what any one caller wants to know about it. The
//! corpus-gate scan asks which functions name a variable and which call which; nothing
//! about that question belongs to the business of telling a definition from a use, which is
//! the whole of what this module does.

use crate::masks::{
    Identifier_After, Is_Code, Matching_Brace, Next_Code_Byte, Scan, Starts_Keyword,
    Without_Comments,
};

/// One function definition, with the text of its body.
pub(crate) struct Function
{
    pub(crate) name: String,
    pub(crate) is_test: bool,
    pub(crate) body: String,
}

/// Every function defined in a file, including ones nested inside another function body.
///
/// A nested helper is found in its own right and also remains part of its parent's body,
/// which is what makes a test that inlines its own corpus lookup count as gated.
///
/// Bodies are read with comments blanked. A variable has to be named in a string to reach
/// `var_os`, so string contents are kept — but a doc comment saying "opt-in by
/// `NOMOS_V14_CORPUS`" describes a gate rather than being one, and every gate in this
/// workspace carries exactly that sentence.
pub(crate) fn Functions(text: &str) -> Vec<Function>
{
    let masks = Scan(text);
    let cleaned = Without_Comments(text, &masks.comment);
    let mut found = Vec::new();
    let mut index = 0_usize;
    while index < text.len()
    {
        let Some(defined) = Definition_At(text, &masks.code, index)
        else
        {
            index = index.saturating_add(1);
            continue;
        };
        found.push(Function {
            name: defined.name,
            is_test: Carries_Test_Attribute(text, index),
            body: cleaned.get(defined.open..=defined.close).unwrap_or_default().to_owned(),
        });
        // Resume inside the body rather than past it, so a nested definition is seen.
        index = defined.open.saturating_add(1);
    }

    return found;
}

/// One `fn` definition: its name, and the offsets of the braces around its body.
struct Definition
{
    name: String,
    open: usize,
    close: usize,
}

/// The function definition starting at exactly this offset, if one does.
///
/// `None` also covers a definition this scan cannot bound — no opening brace after the name,
/// or no brace matching it — which means a truncated source. The caller steps past and keeps
/// looking rather than stopping, so one malformed item cannot hide every item after it.
fn Definition_At(text: &str, mask: &[bool], index: usize) -> Option<Definition>
{
    let bytes = text.as_bytes();

    if !Is_Code(mask, index)
        || !Starts_Keyword(bytes, index, b"fn")
        || !Only_Modifiers_Before(text, index)
    {
        return None;
    }

    // The first brace after the name opens the body. A signature cannot contain one:
    // generics, argument types and return types are all brace-free in Rust.
    let (name, after_name) = Identifier_After(bytes, index.saturating_add(2))?;
    let open = Next_Code_Byte(bytes, mask, after_name, b'{')?;
    let close = Matching_Brace(bytes, mask, open)?;

    return Some(Definition { name, open, close });
}

/// Whether the attribute block immediately above an offset contains `#[test]`.
///
/// Walks back over blank lines, comments and attributes and stops at the first line that
/// is none of those, so the attributes of an earlier item cannot be borrowed by a later
/// one.
fn Carries_Test_Attribute(text: &str, offset: usize) -> bool
{
    let Some(above) = Lines_Above(text, offset)
    else
    {
        return false;
    };

    for line in above.lines().rev()
    {
        let trimmed = line.trim();
        if trimmed == "#[test]"
        {
            return true;
        }
        if !Is_Attribute_Furniture(trimmed)
        {
            return false;
        }
    }

    return false;
}

/// Everything above the line an offset sits on.
///
/// Cut at the line start rather than by dropping the last element of `lines()`: a prefix
/// ending in a newline has no final empty element, so dropping one would discard the
/// attribute itself.
fn Lines_Above(text: &str, offset: usize) -> Option<&str>
{
    let prefix = text.get(..offset)?;
    let line_start = prefix.rfind('\n').map_or(0, |at| return at.saturating_add(1));

    return text.get(..line_start);
}

/// Whether a line belongs to the attribute block above an item rather than ending it.
///
/// Blank lines, comments and attributes all belong to it. A multi-line attribute such as
/// `#[cfg_attr(\n    ...\n)]` ends on `)]`, which is why a trailing `]` counts.
fn Is_Attribute_Furniture(line: &str) -> bool
{
    return line.is_empty()
        || line.starts_with("//")
        || line.starts_with("#[")
        || line.ends_with(']');
}

/// Whether everything before `offset` on its line is whitespace or an item modifier.
///
/// This is what separates a definition from a use: `fn` also appears in `impl Fn()`,
/// `Box<dyn Fn(u32)>` and a `where` clause, none of which start a line.
fn Only_Modifiers_Before(text: &str, offset: usize) -> bool
{
    let Some(prefix) = text.get(..offset)
    else
    {
        return false;
    };

    let line_start = prefix.rfind('\n').map_or(0, |at| return at.saturating_add(1));
    let Some(before) = text.get(line_start..offset)
    else
    {
        return false;
    };

    return before.split_whitespace().all(|token| {
        return matches!(
            token,
            "pub" | "pub(crate)" | "pub(super)" | "async" | "const" | "unsafe" | "extern" | "\"C\""
        );
    });
}
