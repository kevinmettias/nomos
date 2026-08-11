//! Which tests cannot run without a corpus, read out of the source.
//!
//! A corpus-gated test returns early when its corpus is absent, and cargo prints `ok`
//! for a test that returned early exactly as it does for one that asserted. That is the
//! same shape as the defect this workspace exists to prevent: the mechanism was present
//! and it did not run, and nothing said so. Counting the gates is what makes the silence
//! measurable.
//!
//! Derived, never declared. A hand-written list of gated tests would be right on the day
//! it was written and wrong the first time somebody added a gate without updating it —
//! and a stale inventory understates the hole, which is the direction that flatters.

use crate::Workspace;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The environment variables that point a test at a corpus.
///
/// Authored here because there is nothing to infer them from: a corpus is a path this
/// repository does not contain, and the only thing naming it is the variable. Adding a
/// fourth corpus without adding it here makes its tests invisible to the count, so the
/// list is short on purpose and belongs next to the scanner that reads it.
///
/// Mirrored by `Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables`.
pub const CORPUS_VARIABLES: &[&str] =
    &["NOMOS_V14_CORPUS", "NOMOS_SPEC_ARCHIVES", "NOMOS_RUST_CORPUS"];

/// One test that reads a corpus.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorpusGate
{
    /// The file holding the test, relative to the workspace root, in forward slashes.
    pub file: String,
    /// The test function's name.
    pub test: String,
    /// Every corpus variable the test reaches, directly or through a helper it calls.
    pub corpora: BTreeSet<String>,
}

/// Every corpus-gated test in the workspace, in a stable order.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Corpus_Gates() -> Vec<CorpusGate>
{
    let workspace = Workspace::Load();
    let root = Workspace::Workspace_Root();
    let mut gates = Vec::new();

    for member in workspace.Members()
    {
        // The crate doing the counting names the variables in order to count them, so
        // scanning itself would report its own inventory as a gate. Nothing here reads a
        // corpus: these are assertions about the workspace, and they must hold on a runner
        // that has none.
        if member.name != "nomos-contract-tests"
        {
            let found = Gates_In_Member(&member.root, &root);
            gates.extend(found);
        }
    }

    gates.sort();
    return gates;
}

/// Every gate one member declares, from both of the roots a crate compiles from.
fn Gates_In_Member(member_root: &Path, root: &Path) -> Vec<CorpusGate>
{
    let mut gates = Vec::new();

    for directory in ["src", "tests"]
    {
        let source_root = member_root.join(directory);
        if source_root.is_dir()
        {
            let found = Gates_Under(&source_root, root);
            gates.extend(found);
        }
    }

    return gates;
}

/// Every gate the files under one source root declare.
fn Gates_Under(source_root: &Path, root: &Path) -> Vec<CorpusGate>
{
    let mut gates = Vec::new();

    for file in Source_Files(source_root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        let relative = Relative_To(root, &file);
        let found = Gates_In(&relative, &text);

        gates.extend(found);
    }

    return gates;
}

/// A path as it reads from the workspace root, in the one spelling this crate compares on.
fn Relative_To(root: &Path, file: &Path) -> String
{
    return file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");
}

/// The gated tests in one file.
///
/// Two passes. The first records which functions name a corpus variable outright; the
/// second propagates that along calls until nothing changes, because the gate is almost
/// never in the test — it is in a `Corpus()` helper the test calls, sometimes through a
/// second helper such as `Both()`.
///
/// Scoped to one file. A helper shared across files would be missed, which is a false
/// negative rather than a false positive: the count would be too low and the guard would
/// notice the next time it was compared.
fn Gates_In(file: &str, text: &str) -> Vec<CorpusGate>
{
    let functions = Functions(text);
    let mut reach = Named_Outright(&functions);

    Propagate(&functions, &mut reach);

    return functions
        .iter()
        .filter(|function| return function.is_test)
        .filter_map(|function| {
            let corpora = reach.get(&function.name)?;
            return Some(CorpusGate {
                file: file.to_owned(),
                test: function.name.clone(),
                corpora: corpora.clone(),
            });
        })
        .collect();
}

/// The functions that name a corpus variable in their own body, which is where the second
/// pass starts from.
fn Named_Outright(functions: &[Function]) -> BTreeMap<String, BTreeSet<String>>
{
    let mut reach: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for function in functions
    {
        for variable in CORPUS_VARIABLES
        {
            if function.body.contains(variable)
            {
                reach
                    .entry(function.name.clone())
                    .or_default()
                    .insert((*variable).to_owned());
            }
        }
    }

    return reach;
}

/// Carries each function's corpus variables along the calls it makes, until a pass discovers
/// nothing new.
fn Propagate(functions: &[Function], reach: &mut BTreeMap<String, BTreeSet<String>>)
{
    loop
    {
        let discovered = Newly_Reached(functions, reach);
        if discovered.is_empty()
        {
            break;
        }

        for (name, variables) in discovered
        {
            reach.entry(name).or_default().extend(variables);
        }
    }
}

/// What each function would gain this pass, for the functions that would gain anything.
fn Newly_Reached(
    functions: &[Function],
    reach: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<(String, BTreeSet<String>)>
{
    let mut discovered = Vec::new();

    for function in functions
    {
        let gained = Gained(function, reach);
        let known = reach.get(&function.name);
        if gained
            .iter()
            .any(|variable| return known.is_none_or(|set| return !set.contains(variable)))
        {
            discovered.push((function.name.clone(), gained));
        }
    }

    return discovered;
}

/// The corpus variables one function reaches through the helpers its body calls.
///
/// A function does not gain from itself: a recursive call would otherwise report a variable
/// as newly reached on every pass and the fixpoint above would never settle.
fn Gained(function: &Function, reach: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String>
{
    let mut gained = BTreeSet::new();

    for (name, variables) in reach
    {
        if name == &function.name
        {
            continue;
        }
        if function.body.contains(&format!("{name}("))
        {
            gained.extend(variables.iter().cloned());
        }
    }

    return gained;
}

/// One function definition, with the text of its body.
struct Function
{
    name: String,
    is_test: bool,
    body: String,
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
fn Functions(text: &str) -> Vec<Function>
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
fn Starts_Keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool
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
            Some(b'{') => depth = depth.saturating_add(1),
            Some(b'}') =>
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

/// Every `.rs` file under a directory.
/// A file's text with every `#[cfg(test)]` item body blanked out.
///
/// `pub(crate)` for `strategies.rs`, which asks whether a crate declares a determinism
/// strategy and must not be answered by one written inside a unit-test module.
/// `nomos-contracts` declares two — `AnalysisKernel` and `AgentHost` — as compile-time
/// examples of the trait it defines, and counting those as domains would have credited the
/// contracts crate with occupying rows of a table it exists to describe.
///
/// The bodies are blanked rather than removed so that byte offsets are unchanged and a
/// caller may still line up the result with the original.
///
/// Brace matching runs over the same masks the rest of this module uses, because a
/// `panic!("{} ...")` inside a test module would otherwise desynchronise the scan and
/// blank the remainder of the file — which would hide real declarations and report a clean
/// result, the failure direction that flatters.
///
/// The search for a body is bounded by the end of the item the attribute is attached to,
/// which is the whole of `Item_Shape_After`'s job. An earlier version asked only for the
/// next open brace anywhere in the file, and for `#[cfg(test)] mod registration;` — the
/// ordinary way to declare a test-only module living in another file — that brace belonged
/// to some later, unrelated item, so everything between them was blanked including that
/// item. `nomos-spec-store` lost its `pub use authoring::{…}` that way and the public
/// surface snapshot went red naming the crate, not the scanner.
pub(crate) fn Without_Test_Modules(text: &str) -> String
{
    let bytes = text.as_bytes();
    let masks = Scan(text);
    // A byte buffer rather than an in-place edit of the `String`: this crate forbids
    // unsafe, so there is no mutable view of a `String`'s bytes to reach for, and blanking
    // to spaces keeps every offset and every line break where it was.
    let mut blanked = bytes.to_vec();
    let mut index = 0_usize;
    while index < bytes.len()
    {
        let Some((from, to)) = Marked_Range(bytes, &masks.code, index)
        else
        {
            index = index.saturating_add(1);
            continue;
        };

        Blank(&mut blanked, from, to);
        index = to.saturating_add(1);
    }

    // Blanking replaces whole bytes of what was valid UTF-8 with ASCII spaces, so the
    // result is still valid UTF-8 — but a multi-byte character partially overwritten would
    // not be, and the lossy conversion is what keeps a scanner bug from becoming a panic in
    // a check that is supposed to report.
    return String::from_utf8_lossy(&blanked).into_owned();
}

/// The byte range one `#[cfg(test)]` item occupies, if the marker begins at `index`.
///
/// A body is blanked from its opening brace, leaving the attribute and the item's signature
/// legible, which is what every caller has always seen. A declaration has no body to blank,
/// so what goes is the declaration itself — from the attribute through its `;` — and nothing
/// beyond it.
///
/// `None` for an offset the marker does not begin at, and for an item this scan cannot bound,
/// which means a truncated source.
fn Marked_Range(bytes: &[u8], mask: &[bool], index: usize) -> Option<(usize, usize)>
{
    if !Is_Code(mask, index) || !Starts_Marker(bytes, index)
    {
        return None;
    }

    let after = index.saturating_add(MARKER.len());
    match Item_Shape_After(bytes, mask, after)?
    {
        ItemShape::Body(open) =>
        {
            let close = Matching_Brace(bytes, mask, open)?;

            return Some((open, close));
        }
        ItemShape::Declaration(end) => return Some((index, end)),
    }
}

/// Overwrites a byte range with spaces, leaving line breaks where they were.
///
/// Keeping the newlines is what lets a caller line a blanked buffer up with the original by
/// line as well as by offset.
fn Blank(bytes: &mut [u8], from: usize, to: usize)
{
    for offset in from..=to
    {
        let Some(slot) = bytes.get_mut(offset)
        else
        {
            continue;
        };

        if *slot != b'\n'
        {
            *slot = b' ';
        }
    }
}

/// The attribute that marks an item as test-only.
const MARKER: &[u8] = b"#[cfg(test)]";

/// Whether a `#[cfg(test)]` attribute begins at an offset.
fn Starts_Marker(bytes: &[u8], index: usize) -> bool
{
    return bytes
        .get(index..index.saturating_add(MARKER.len()))
        .is_some_and(|window| return window == MARKER);
}

/// How the item carrying a `#[cfg(test)]` attribute ends.
///
/// The distinction the scanner needs is not which keyword follows the attribute but
/// whether the item terminates with a body or with a `;`, because that is the only thing
/// that decides how much text belongs to it. `mod x { … }`, `fn f() { … }` and
/// `impl T for U { … }` are one shape; `mod x;`, `use a::b;` and `struct S;` are the
/// other. Reading the keyword instead would mean listing every item form Rust has and
/// re-listing it whenever one is added — and `mod` alone appears in both shapes anyway.
enum ItemShape
{
    /// The item has a body. The offset is its opening brace.
    Body(usize),
    /// The item is a declaration and has no body. The offset is the `;` that ends it.
    Declaration(usize),
}

/// Whichever of `{` or `;` ends the item beginning at `from`, and which one it was.
///
/// This is the bound that `Without_Test_Modules` was missing. Whichever of the two comes
/// first decides the shape, so a declaration can never borrow the body of the item after
/// it: the `;` is reached before that item's brace is.
///
/// Nesting depth counts only `(`/`)` and `[`/`]`, and both delimiters matter. A `;` can sit
/// inside either without ending anything — `fn f(v: [u8; 4]) { … }` has one in an array
/// type, ahead of the brace that really is the body — and a further `#[…]` attribute
/// stacked under the marker opens and closes a bracket of its own before the item even
/// begins. Angle brackets are deliberately not counted: `<` is ambiguous with comparison
/// in Rust's grammar, and it need not be, because generics and where-clauses put no bare
/// `;` or `{` between the attribute and the body. Anything they could carry that looks
/// like one — an array length, a const-generic argument — is already inside `[]` or `()`.
///
/// Only code bytes are read, so a `;` in a string and a `{` in a comment are both invisible
/// to it. `None` means the file ends mid-item, which is a truncated source rather than a
/// declaration, and the caller stops rather than guessing.
fn Item_Shape_After(bytes: &[u8], mask: &[bool], from: usize) -> Option<ItemShape>
{
    let mut cursor = from;
    let mut depth = 0_u32;

    while cursor < bytes.len()
    {
        if Is_Code(mask, cursor)
        {
            match bytes.get(cursor).copied()
            {
                Some(b'(' | b'[') => depth = depth.saturating_add(1),
                Some(b')' | b']') => depth = depth.saturating_sub(1),
                Some(b'{') if depth == 0 => return Some(ItemShape::Body(cursor)),
                Some(b';') if depth == 0 => return Some(ItemShape::Declaration(cursor)),
                _ =>
                {}
            }
        }
        cursor = cursor.saturating_add(1);
    }

    return None;
}

/// Every `.rs` file under a directory that is compiled into the crate proper.
///
/// `pub(crate)` because `strategies.rs` walks the same trees for a different property,
/// and two walkers would eventually disagree about what counts as a source file.
///
/// A file declared `#[cfg(test)] mod tests;` is left out, for exactly the reason
/// [`Without_Test_Modules`] blanks an inline `#[cfg(test)] mod tests { … }`: it is a unit
/// test, and a scanner that reads it is reading examples as though they were the thing
/// they are examples of. Without this, moving a test module out of the file it tests
/// makes the crate look like it constructs facts and declares no strategy — which is what
/// happened to `nomos-rules` the first time its `mirror` module became a directory.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Rust_Files_Under(root);
    let test_only = Test_Only_Modules(&found);

    found.retain(|path| return !test_only.iter().any(|excluded| return path.starts_with(excluded)));

    return found;
}

/// Every `.rs` file under a directory, in whatever order the filesystem answers.
fn Rust_Files_Under(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| return extension == "rs")
            {
                found.push(path);
            }
        }
    }

    return found;
}

/// Every path a `#[cfg(test)] mod <name>;` declaration points at.
///
/// Both spellings, because Rust accepts either: `<name>.rs` beside the declaring file, or
/// `<name>/` beneath it. The declaration is read rather than the file name, so a module
/// that happens to be called `tests` and is compiled into the crate proper is still read.
fn Test_Only_Modules(files: &[PathBuf]) -> Vec<PathBuf>
{
    let mut excluded = Vec::new();
    for file in files
    {
        excluded.extend(Test_Only_In(file));
    }

    return excluded;
}

/// The paths the `#[cfg(test)] mod <name>;` declarations in one file point at.
fn Test_Only_In(file: &Path) -> Vec<PathBuf>
{
    let mut excluded = Vec::new();
    let Ok(text) = std::fs::read_to_string(file)
    else
    {
        return excluded;
    };
    let Some(home) = file.parent()
    else
    {
        return excluded;
    };
    let mut gated = false;
    for line in text.lines()
    {
        let line = line.trim();
        if let Some(name) = Declared_Module(line).filter(|_| return gated)
        {
            excluded.push(home.join(format!("{name}.rs")));
            excluded.push(home.join(name));
        }
        gated = line == "#[cfg(test)]";
    }

    return excluded;
}

/// The module name a `mod <name>;` line declares, if the line is one.
fn Declared_Module(line: &str) -> Option<&str>
{
    let name = line.strip_prefix("mod ")?.strip_suffix(';')?.trim();

    return name
        .chars()
        .all(|character| return character.is_ascii_alphanumeric() || character == '_')
        .then_some(name);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Test_Calling_A_Gated_Helper_Should_Be_Gated()
    {
        let source = r#"
fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    return Some(root);
}

#[test]
fn Test_Something()
{
    let Some(root) = Corpus() else { return };
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
        assert_eq!(gates.first().map(|gate| return gate.test.clone()), Some("Test_Something".to_owned()));
    }

    #[test]
    fn Test_A_Gate_Reached_Through_Two_Helpers_Should_Still_Be_Found()
    {
        let source = r#"
fn Archives() -> Option<PathBuf>
{
    return std::env::var_os("NOMOS_SPEC_ARCHIVES").map(PathBuf::from);
}

fn Both() -> Option<PathBuf>
{
    return Archives();
}

#[test]
fn Test_Compares_Two_Revisions()
{
    let Some(root) = Both() else { return };
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
    }

    /// The case that makes naive brace matching wrong. `panic!("{}", ...)` closes a brace
    /// the scanner never opened, and every function after it is misattributed.
    #[test]
    fn Test_A_Brace_Inside_A_String_Should_Not_End_A_Body()
    {
        let source = r#"
#[test]
fn Test_Reads_A_Corpus()
{
    let root = std::env::var_os("NOMOS_V14_CORPUS");
    panic!("} unbalanced {");
}

#[test]
fn Test_Reads_Nothing()
{
    assert!(true);
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
        assert_eq!(
            gates.first().map(|gate| return gate.test.clone()),
            Some("Test_Reads_A_Corpus".to_owned())
        );
    }

    /// A doc comment naming a variable is documentation, not a gate.
    #[test]
    fn Test_A_Variable_Named_Only_In_A_Comment_Should_Not_Gate()
    {
        let source = r"
/// Opt-in by NOMOS_V14_CORPUS.
#[test]
fn Test_Runs_Always()
{
    assert!(true); // NOMOS_SPEC_ARCHIVES is not read here
}
";

        assert!(Gates_In("example.rs", source).is_empty());
    }

    #[test]
    fn Test_An_Ungated_Test_Should_Not_Be_Reported()
    {
        let source = r"
#[test]
fn Test_Plain()
{
    assert_eq!(1 + 1, 2);
}
";

        assert!(Gates_In("example.rs", source).is_empty());
    }

    /// A test module with a body still loses the whole body, which is the behaviour every
    /// caller of `Without_Test_Modules` has always depended on.
    #[test]
    fn Test_A_Braced_Test_Item_Should_Still_Lose_Its_Whole_Body()
    {
        let source = r"
#[cfg(test)]
mod tests
{
    pub fn Helper_Inside_Tests() {}
}

pub use authoring::{Claimed};
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Helper_Inside_Tests"),
            "the body of a braced #[cfg(test)] item survived: {blanked}"
        );
        assert!(
            blanked.contains("pub use authoring::{Claimed};"),
            "blanking a braced test module reached past it: {blanked}"
        );
        assert_eq!(blanked.len(), source.len());
    }

    /// The defect, in the shape it was found in. `#[cfg(test)] mod registration;` carries no
    /// braces of its own, so an unbounded search for a body takes the braces of whatever
    /// item comes next — in `nomos-spec-store` that was the `pub use authoring::{…}` line,
    /// and the public surface snapshot reported eleven exports missing without ever
    /// mentioning the scanner.
    #[test]
    fn Test_A_Braceless_Test_Declaration_Should_Not_Eat_The_Item_After_It()
    {
        let source = r"
mod record;
#[cfg(test)]
mod registration;
mod rows;

pub use authoring::{
    BlockChange, ClaimedRecord, CommitReport,
};
";

        let blanked = Without_Test_Modules(source);
        assert!(
            blanked.contains("pub use authoring::{"),
            "pub use authoring::{{…}} was eaten by the brace-less #[cfg(test)] mod \
             registration; above it; what survived was: {blanked}"
        );
        for export in ["BlockChange", "ClaimedRecord", "CommitReport"]
        {
            assert!(
                blanked.contains(export),
                "the export {export} was eaten by the brace-less #[cfg(test)] mod \
                 registration; above it; what survived was: {blanked}"
            );
        }
        assert!(
            blanked.contains("mod rows;"),
            "mod rows; was eaten by the brace-less #[cfg(test)] mod registration; above it; \
             what survived was: {blanked}"
        );
    }

    /// A brace-less declaration blanks itself, so nothing downstream can read the module
    /// name back out and count it as a declared file.
    #[test]
    fn Test_A_Braceless_Test_Declaration_Should_Blank_Itself()
    {
        let source = r"
mod record;
#[cfg(test)]
mod registration;
mod rows;
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("registration"),
            "a brace-less #[cfg(test)] declaration was left in place: {blanked}"
        );
        assert!(!blanked.contains("#[cfg(test)]"), "the marker was left in place: {blanked}");
        assert!(blanked.contains("mod record;"), "the item above it was blanked: {blanked}");
        assert_eq!(blanked.len(), source.len());
    }

    /// The shapes are decided by `;` versus `{`, not by the keyword, so every brace-less
    /// item form has to behave the same way. Each of these is followed by a braced item that
    /// must survive.
    #[test]
    fn Test_Every_Braceless_Item_Form_Should_Blank_Only_Itself()
    {
        for declaration in [
            "mod registration;",
            "use super::Helper;",
            "struct Marker;",
            "type Alias = Vec<u8>;",
            "static LIMIT: [u8; 4] = [0; 4];",
        ]
        {
            let source = format!("\n#[cfg(test)]\n{declaration}\n\npub fn Survivor() {{}}\n");
            let blanked = Without_Test_Modules(&source);
            assert!(
                blanked.contains("pub fn Survivor()"),
                "`{declaration}` ate the item after it: {blanked}"
            );
            assert!(
                !blanked.contains("#[cfg(test)]"),
                "`{declaration}` was not blanked: {blanked}"
            );
        }
    }

    /// Generics, a where-clause and an array-typed parameter all put punctuation between the
    /// attribute and the body without ending the item. The `;` inside `[u8; 4]` is the one
    /// that would fool a scanner reading for the first `;` anywhere.
    #[test]
    fn Test_A_Body_Behind_Generics_And_An_Array_Type_Should_Still_Be_Blanked()
    {
        let source = r"
#[cfg(test)]
fn Helper<T>(value: [u8; 4]) -> Result<T, ()>
where
    T: Default,
{
    let Eaten_Marker = value;
    return Ok(T::default());
}

pub fn Survivor() {}
";

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Eaten_Marker"),
            "the body was not blanked, so the `;` in [u8; 4] was read as the item's end: {blanked}"
        );
        assert!(blanked.contains("pub fn Survivor()"), "blanking reached past the body: {blanked}");
    }

    /// The shape is decided over code bytes only. A `;` in a comment between the attribute
    /// and the body would otherwise turn a braced item into a declaration and leave the body
    /// in the text — the failure direction that flatters, because it reports more surface
    /// than the crate has.
    #[test]
    fn Test_A_Semicolon_In_A_Comment_Should_Not_End_An_Item()
    {
        let source = r#"
#[cfg(test)]
// a declaration would have ended here;
mod tests
{
    const NOTE: &str = "and here;";
    fn Inside_The_Body() {}
}

pub fn Survivor() {}
"#;

        let blanked = Without_Test_Modules(source);
        assert!(
            !blanked.contains("Inside_The_Body"),
            "a `;` in a comment was read as the end of a braced item: {blanked}"
        );
        assert!(blanked.contains("pub fn Survivor()"), "blanking reached past the body: {blanked}");
    }
}
