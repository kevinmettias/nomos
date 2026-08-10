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
        // corpus: these are assertions about the workspace, and they must hold on a
        // runner that has none.
        if member.name == "nomos-contract-tests"
        {
            continue;
        }

        for directory in ["src", "tests"]
        {
            let source_root = member.root.join(directory);
            if !source_root.is_dir()
            {
                continue;
            }

            for file in Source_Files(&source_root)
            {
                let Ok(text) = std::fs::read_to_string(&file)
                else
                {
                    continue;
                };

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .display()
                    .to_string()
                    .replace('\\', "/");

                gates.extend(Gates_In(&relative, &text));
            }
        }
    }

    gates.sort();
    return gates;
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
    let mut reach: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for function in &functions
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

    loop
    {
        let mut discovered: Vec<(String, BTreeSet<String>)> = Vec::new();

        for function in &functions
        {
            let mut gained = BTreeSet::new();
            for (name, variables) in &reach
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

            let known = reach.get(&function.name);
            if gained
                .iter()
                .any(|variable| return known.is_none_or(|set| return !set.contains(variable)))
            {
                discovered.push((function.name.clone(), gained));
            }
        }

        if discovered.is_empty()
        {
            break;
        }

        for (name, variables) in discovered
        {
            reach.entry(name).or_default().extend(variables);
        }
    }

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
fn Functions(text: &str) -> Vec<Function>
{
    let bytes = text.as_bytes();
    let masks = Scan(text);
    let mask = &masks.code;
    // Bodies are read with comments blanked. A variable has to be named in a string to
    // reach `var_os`, so string contents are kept — but a doc comment saying "opt-in by
    // NOMOS_V14_CORPUS" describes a gate rather than being one, and every gate in this
    // workspace carries exactly that sentence.
    let cleaned = Without_Comments(text, &masks.comment);
    let mut found = Vec::new();
    let mut index = 0_usize;

    while index < bytes.len()
    {
        if !Is_Code(mask, index)
            || !Starts_Keyword(bytes, index, b"fn")
            || !Only_Modifiers_Before(text, index)
        {
            index = index.saturating_add(1);
            continue;
        }

        let Some((name, after_name)) = Identifier_After(bytes, index.saturating_add(2))
        else
        {
            index = index.saturating_add(1);
            continue;
        };

        // The first brace after the name opens the body. A signature cannot contain one:
        // generics, argument types and return types are all brace-free in Rust.
        let Some(open) = Next_Code_Byte(bytes, mask, after_name, b'{')
        else
        {
            break;
        };

        let Some(close) = Matching_Brace(bytes, mask, open)
        else
        {
            break;
        };

        found.push(Function {
            name,
            is_test: Carries_Test_Attribute(text, index),
            body: cleaned.get(open..=close).unwrap_or_default().to_owned(),
        });

        // Resume inside the body rather than past it, so a nested definition is seen.
        index = open.saturating_add(1);
    }

    return found;
}

/// Whether the attribute block immediately above an offset contains `#[test]`.
///
/// Walks back over blank lines, comments and attributes and stops at the first line that
/// is none of those, so the attributes of an earlier item cannot be borrowed by a later
/// one.
fn Carries_Test_Attribute(text: &str, offset: usize) -> bool
{
    let Some(prefix) = text.get(..offset)
    else
    {
        return false;
    };

    // Everything above the line the `fn` sits on. Cutting at the line start rather than
    // dropping the last element of `lines()`: a prefix ending in a newline has no final
    // empty element, so dropping one would discard the attribute itself.
    let line_start = prefix.rfind('\n').map_or(0, |at| return at.saturating_add(1));
    let Some(above) = text.get(..line_start)
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
        // A multi-line attribute such as `#[cfg_attr(\n    ...\n)]` ends on `)]`.
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("#[")
            || trimmed.ends_with(']')
        {
            continue;
        }
        return false;
    }

    return false;
}

/// What each byte of a file is.
pub(crate) struct Masks
{
    /// Ordinary code: not a comment, and not inside a literal.
    pub(crate) code: Vec<bool>,
    /// Inside a line or block comment, the delimiters included.
    pub(crate) comment: Vec<bool>,
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
        let current = bytes.get(index).copied().unwrap_or(0);
        let next = bytes.get(index.saturating_add(1)).copied().unwrap_or(0);

        if current == b'/' && next == b'/'
        {
            let start = index;
            while index < bytes.len() && bytes.get(index).copied() != Some(b'\n')
            {
                index = index.saturating_add(1);
            }
            Mark(&mut masks.comment, start, index);
            continue;
        }

        if current == b'/' && next == b'*'
        {
            // Rust nests block comments, so a depth counter rather than a search for the
            // first `*/`.
            let start = index;
            let mut depth = 1_u32;
            index = index.saturating_add(2);
            while index < bytes.len() && depth > 0
            {
                let opening = bytes.get(index).copied().unwrap_or(0);
                let following = bytes.get(index.saturating_add(1)).copied().unwrap_or(0);
                if opening == b'/' && following == b'*'
                {
                    depth = depth.saturating_add(1);
                    index = index.saturating_add(2);
                }
                else if opening == b'*' && following == b'/'
                {
                    depth = depth.saturating_sub(1);
                    index = index.saturating_add(2);
                }
                else
                {
                    index = index.saturating_add(1);
                }
            }
            Mark(&mut masks.comment, start, index);
            continue;
        }

        if let Some(after) = Raw_String_End(bytes, index)
        {
            index = after;
            continue;
        }

        if current == b'"'
        {
            index = index.saturating_add(1);
            while let Some(byte) = bytes.get(index).copied()
            {
                if byte == b'\\'
                {
                    index = index.saturating_add(2);
                    continue;
                }
                index = index.saturating_add(1);
                if byte == b'"'
                {
                    break;
                }
            }
            continue;
        }

        if current == b'\''
        {
            if let Some(after) = Character_Literal_End(bytes, index)
            {
                index = after;
                continue;
            }
            // Otherwise a lifetime, which is ordinary code.
        }

        if let Some(slot) = masks.code.get_mut(index)
        {
            *slot = true;
        }
        index = index.saturating_add(1);
    }

    return masks;
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
    if index > 0
    {
        let previous = bytes.get(index.saturating_sub(1)).copied().unwrap_or(0);
        if previous.is_ascii_alphanumeric() || previous == b'_'
        {
            return None;
        }
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
    cursor = cursor.saturating_add(1);

    while cursor < bytes.len()
    {
        if bytes.get(cursor).copied() == Some(b'"')
        {
            let mut closing = 0_usize;
            while closing < hashes
                && bytes.get(cursor.saturating_add(closing).saturating_add(1)).copied()
                    == Some(b'#')
            {
                closing = closing.saturating_add(1);
            }
            if closing == hashes
            {
                return Some(cursor.saturating_add(hashes).saturating_add(1));
            }
        }
        cursor = cursor.saturating_add(1);
    }

    return Some(bytes.len());
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
        // An escape is at most `'\u{10FFFF}'`; anything longer is not a literal.
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

    let width = if first < 0x80
    {
        1_usize
    }
    else if first >> 5 == 0b110
    {
        2_usize
    }
    else if first >> 4 == 0b1110
    {
        3_usize
    }
    else
    {
        4_usize
    };

    let closing = index.saturating_add(1).saturating_add(width);
    if bytes.get(closing).copied() == Some(b'\'')
    {
        return Some(closing.saturating_add(1));
    }

    return None;
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

    if index > 0
    {
        let previous = bytes.get(index.saturating_sub(1)).copied().unwrap_or(0);
        if previous.is_ascii_alphanumeric() || previous == b'_'
        {
            return false;
        }
    }

    let following = bytes
        .get(index.saturating_add(keyword.len()))
        .copied()
        .unwrap_or(b' ');

    return !(following.is_ascii_alphanumeric() || following == b'_');
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
    while let Some(byte) = bytes.get(cursor).copied()
    {
        if !(byte.is_ascii_alphanumeric() || byte == b'_')
        {
            break;
        }
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
        if Is_Code(mask, cursor)
        {
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
        if !Is_Code(&masks.code, index) || !Starts_Marker(bytes, index)
        {
            index = index.saturating_add(1);
            continue;
        }

        let Some(open) = Next_Code_Byte(bytes, &masks.code, index, b'{')
        else
        {
            break;
        };

        let Some(close) = Matching_Brace(bytes, &masks.code, open)
        else
        {
            break;
        };

        for offset in open..=close
        {
            if let Some(slot) = blanked.get_mut(offset)
            {
                if *slot != b'\n'
                {
                    *slot = b' ';
                }
            }
        }

        index = close.saturating_add(1);
    }

    // Blanking replaces whole bytes of what was valid UTF-8 with ASCII spaces, so the
    // result is still valid UTF-8 — but a multi-byte character partially overwritten would
    // not be, and the lossy conversion is what keeps a scanner bug from becoming a panic in
    // a check that is supposed to report.
    return String::from_utf8_lossy(&blanked).into_owned();
}

/// Whether a `#[cfg(test)]` attribute begins at an offset.
fn Starts_Marker(bytes: &[u8], index: usize) -> bool
{
    const MARKER: &[u8] = b"#[cfg(test)]";

    return bytes
        .get(index..index.saturating_add(MARKER.len()))
        .is_some_and(|window| return window == MARKER);
}

/// Every `.rs` file under a directory, recursively.
///
/// `pub(crate)` because `strategies.rs` walks the same trees for a different property,
/// and two walkers would eventually disagree about what counts as a source file.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
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
}
