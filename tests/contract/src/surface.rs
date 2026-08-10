//! What each crate exports, read out of its own source.
//!
//! Five anti-leak assertions were named for this workspace. Three are in `boundaries.rs`,
//! module reachability shipped, and this is the fifth: the surface each crate presents to
//! everything above it, written down so that widening it is a decision somebody committed
//! rather than a review question somebody answered in passing.
//!
//! # Why this and not `cargo public-api`
//!
//! Recorded in `OD-GATE-002`. In short: the tool is not installed on this machine and is
//! on no CI runner, it needs a nested `cargo` invocation that would block on the target
//! directory lock held by the very `cargo test` calling it, and a check that cannot run
//! where the pull requests are is the defect `P9-PUBLIC-API` was opened about. What is
//! given up is rustdoc's resolution — that record says exactly what.
//!
//! # What this sees
//!
//! Items declared `pub` in a module reachable from the crate root by a chain of
//! `pub mod`, and items pulled to a reachable module by `pub use`. Associated items of
//! `impl` and `pub trait` blocks, public struct fields and enum variants are part of the
//! surface and are emitted. `pub(crate)`, `pub(super)` and `pub(in …)` are not public and
//! are not.
//!
//! A re-exported name this resolver cannot follow is reported by [`Surface::unresolved`]
//! rather than dropped, because a re-export nobody could follow is the one place a leak
//! would hide. One line per *name*, not per `pub use`: a list whose other names resolve is
//! exactly where a dropped one hides best — `OD-GATE-002` version 2.

use crate::gates::{
    Identifier_After, Is_Code, Matching_Brace, Scan, Without_Comments, Without_Test_Modules,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One crate's exported surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Surface
{
    /// The package name.
    pub package: String,
    /// Every exported declaration, sorted and deduplicated.
    pub declarations: Vec<String>,
    /// Re-exported names this resolver could not follow to a declaration in this crate.
    ///
    /// Not an error. `pub use rusqlite::Connection` is a real re-export of somebody
    /// else's type and there is nothing in this crate to resolve it to. Reported so the
    /// list is visible rather than silently short.
    ///
    /// One entry per name a `pub use` gives, so a name that does not resolve is reported
    /// whether or not the names beside it do.
    pub unresolved: Vec<String>,
}

/// The identifier form of a package name, as a path would spell it.
#[must_use]
pub fn Crate_Identifier(package: &str) -> String
{
    return package.replace('-', "_");
}

/// The exported surface of the library in `root`, or [`None`] if it has no library.
///
/// # Panics
///
/// Panics if a module file named by a `mod` declaration cannot be read. A surface
/// assembled from the modules that happened to open is a surface that under-reports, and
/// under-reporting is the direction that flatters.
#[must_use]
pub fn Public_Surface(package: &str, root: &Path) -> Option<Surface>
{
    let entry = root.join("src/lib.rs");
    if !entry.is_file()
    {
        return None;
    }

    let mut modules = BTreeMap::new();
    Load_Module(&entry, &[], &mut modules);

    let identifier = Crate_Identifier(package);
    let mut declarations = BTreeSet::new();
    let mut unresolved = BTreeSet::new();

    for (path, module) in &modules
    {
        if !module.exported
        {
            continue;
        }

        let prefix = Prefixed(&identifier, path);
        for item in &module.items
        {
            Emit(item, &prefix, &mut declarations);
        }

        for re_export in &module.re_exports
        {
            Emit_Re_Export(
                re_export,
                path,
                &identifier,
                &modules,
                &mut declarations,
                &mut unresolved,
            );
        }
    }

    return Some(Surface {
        package: package.to_owned(),
        declarations: declarations.into_iter().collect(),
        unresolved: unresolved.into_iter().collect(),
    });
}

/// The path a declaration in `module` is written under.
fn Prefixed(identifier: &str, module: &[String]) -> String
{
    if module.is_empty()
    {
        return identifier.to_owned();
    }

    return format!("{identifier}::{}", module.join("::"));
}

/// One module of a crate, as its source declares it.
#[derive(Debug, Default)]
struct Module
{
    /// Whether every `mod` from the crate root down to it is `pub`.
    exported: bool,
    /// The public items it declares.
    items: Vec<Item>,
    /// Its `pub use` declarations, as written after `pub use ` and before the `;`.
    re_exports: Vec<String>,
}

/// One public declaration.
#[derive(Clone, Debug)]
struct Item
{
    /// `fn`, `struct`, `enum`, `trait`, `type`, `const`, `static`, `macro`, `impl`,
    /// `struct field` or `enum variant`.
    kind: String,
    /// The name, qualified by its owning type where it has one.
    name: String,
    /// Everything after the name: parameters, return type, field type.
    tail: String,
    /// Anything before the kind keyword that belongs to the declaration, such as `unsafe`
    /// or `async`.
    modifiers: String,
}

/// Reads a module file and every module it declares, into `into`.
fn Load_Module(file: &Path, path: &[String], into: &mut BTreeMap<Vec<String>, Module>)
{
    let text = std::fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", file.display()));
    // Test module bodies and then comments, both blanked in place so every byte offset
    // still means the same place. Comments have to go: this workspace documents almost
    // every item, and a doc comment reading "/// Returns the pub fn's answer" is prose
    // that a scanner looking for declarations would otherwise take at its word.
    let without_tests = Without_Test_Modules(&text);
    let cleaned = Without_Comments(&without_tests, &Scan(&without_tests).comment);
    let masks = Scan(&cleaned);

    let mut module = Module {
        // The root is exported by definition; a child's flag is set by its parent below.
        exported: path.is_empty(),
        ..Module::default()
    };
    let mut children: Vec<(String, bool)> = Vec::new();

    Walk(
        &cleaned,
        &masks,
        (0, cleaned.len()),
        None,
        &mut module,
        &mut children,
        path,
        into,
    );

    let previously_exported = into.get(path).is_some_and(|held| return held.exported);
    module.exported = module.exported || previously_exported;
    let exported = module.exported;
    into.insert(path.to_vec(), module);

    let directory = Module_Directory(file, path);
    for (name, is_public) in children
    {
        let mut child_path = path.to_vec();
        child_path.push(name.clone());

        // Recorded before the file is read, so that a module whose file is missing still
        // carries its visibility and an inline `mod` block already loaded keeps it.
        let entry = into.entry(child_path.clone()).or_default();
        entry.exported = entry.exported || (exported && is_public);

        let Some(child_file) = Module_File(&directory, &name)
        else
        {
            continue;
        };

        let inherited = into.get(&child_path).is_some_and(|held| return held.exported);
        Load_Module(&child_file, &child_path, into);
        if inherited && let Some(held) = into.get_mut(&child_path)
        {
            held.exported = true;
        }
    }
}

/// The directory a module's children live in.
///
/// `src/lib.rs` and `src/thing/mod.rs` own the directory they sit in; `src/thing.rs` owns
/// `src/thing/`.
fn Module_Directory(file: &Path, path: &[String]) -> PathBuf
{
    let parent = file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let stem = file.file_stem().and_then(std::ffi::OsStr::to_str).unwrap_or_default();

    if path.is_empty() || stem == "mod"
    {
        return parent;
    }

    return parent.join(stem);
}

/// The file backing a child module, in either of the two layouts.
fn Module_File(directory: &Path, name: &str) -> Option<PathBuf>
{
    return [directory.join(format!("{name}.rs")), directory.join(name).join("mod.rs")]
        .into_iter()
        .find(|candidate| return candidate.is_file());
}

/// What the walker is currently inside.
#[derive(Clone, Copy)]
enum Inside<'a>
{
    /// An inherent `impl Type` block, carrying the type's name.
    ///
    /// Its members need their own `pub`. `Workspace::Name_From_Package_Id` is an inherent
    /// helper with no visibility of its own and is not an export, and a scanner that took
    /// every member of every `impl` for public would list it.
    Implementation(&'a str),
    /// An `impl Trait for Type` block, carrying the type's name.
    ///
    /// Its members are as public as the trait, so they carry no `pub` of their own.
    Conformance(&'a str),
    /// A `pub trait` block, carrying the trait's name.
    Declaration(&'a str),
    /// A `pub struct` body, carrying the struct's name.
    Fields(&'a str),
    /// A `pub enum` body, carrying the enum's name.
    Variants(&'a str),
}

/// Collects the public items in a byte range.
#[allow(clippy::too_many_arguments)]
fn Walk(
    text: &str,
    masks: &crate::gates::Masks,
    range: (usize, usize),
    inside: Option<Inside<'_>>,
    module: &mut Module,
    children: &mut Vec<(String, bool)>,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
)
{
    let (start, end) = range;
    let mut cursor = start;

    while cursor < end
    {
        let line_end = Line_End(text, cursor, end);

        if let Some(Inside::Variants(owner)) = inside
        {
            for (name, payload) in Members_In(text, masks, (cursor, end), false)
            {
                module.items.push(Item {
                    kind: "enum variant".to_owned(),
                    name: format!("{owner}::{name}"),
                    tail: payload,
                    modifiers: String::new(),
                });
            }
            return;
        }

        if let Some(Inside::Fields(owner)) = inside
        {
            for (name, declared) in Members_In(text, masks, (cursor, end), true)
            {
                module.items.push(Item {
                    kind: "struct field".to_owned(),
                    name: format!("{owner}::{name}"),
                    tail: declared,
                    modifiers: String::new(),
                });
            }
            return;
        }

        let Some(head) = Head(text, masks, cursor, end)
        else
        {
            cursor = Next_Line(text, line_end, end);
            continue;
        };

        Record(&head, inside, module, children, path, into, text, masks);
        cursor = Next_Line(text, head.after, end);
    }
}

/// One recognised declaration and where it ends.
struct Recognised
{
    /// The declaration with runs of whitespace collapsed, without the trailing `{` or `;`.
    text: String,
    /// The keyword naming what it declares.
    kind: String,
    /// Whatever preceded the keyword other than visibility.
    modifiers: String,
    /// The visibility as written: `pub`, `pub(crate)`, or empty.
    visibility: String,
    /// The body's byte range, when the declaration has one.
    body: Option<(usize, usize)>,
    /// The offset just past the whole declaration.
    after: usize,
}

/// Every keyword that opens an item this cares about.
const KEYWORDS: &[&str] = &[
    "fn", "struct", "enum", "trait", "type", "const", "static", "mod", "use", "impl", "union",
];

/// Reads a declaration starting at `from`, if one starts there.
fn Head(text: &str, masks: &crate::gates::Masks, from: usize, end: usize) -> Option<Recognised>
{
    let line_end = Line_End(text, from, end);
    let line = text.get(from..line_end)?.trim_start();
    let indent = text.get(from..line_end)?.len().checked_sub(line.len())?;
    let begins = from.checked_add(indent)?;

    let (visibility, modifiers, kind) = Opening(line)?;

    // A declaration ends at the first `{` or `;` outside parentheses. `where` clauses and
    // return types cannot introduce either, and a body-bearing item always reaches a
    // brace before a semicolon.
    //
    // `use` is the exception and the one that mattered: the braces in `use a::{B, C};` are
    // a list, not a body. Reading them as one truncated every grouped re-export in this
    // workspace to `pub use a::` and left the items behind them out of every snapshot —
    // which is a surface check reporting an empty surface.
    let (terminator, at) = if kind == "use"
    {
        Semicolon(text, masks, begins, end).map(|at| return (b';', at))?
    }
    else
    {
        Terminator(text, masks, begins, end)?
    };

    // A constant's value is not its surface. `SHIPPED` is a fourteen-entry table of
    // `include_str!` calls, and putting it in a snapshot would make every profile file
    // rename read as an API change.
    let cut = if matches!(kind.as_str(), "const" | "static" | "type")
    {
        Assignment(text, masks, begins, at).unwrap_or(at)
    }
    else
    {
        at
    };
    let declaration = text.get(begins..cut)?;

    let body = (terminator == b'{')
        .then(|| return Matching_Brace(text.as_bytes(), &masks.code, at))
        .flatten()
        .map(|close| return (at.saturating_add(1), close));

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

/// Splits a line's leading tokens into visibility, other modifiers and the item keyword.
fn Opening(line: &str) -> Option<(String, String, String)>
{
    let mut visibility = String::new();
    let mut modifiers: Vec<&str> = Vec::new();
    let mut rest = line;

    if let Some(after) = rest.strip_prefix("pub")
    {
        if let Some(restricted) = after.strip_prefix('(')
        {
            let close = restricted.find(')')?;
            visibility = format!("pub({})", restricted.get(..close)?);
            rest = restricted.get(close.saturating_add(1)..)?;
        }
        else if after.starts_with(char::is_whitespace)
        {
            "pub".clone_into(&mut visibility);
            rest = after;
        }
        else
        {
            return None;
        }
    }

    let words: Vec<&str> = rest
        .split_whitespace()
        .map(|token| return token.split(['<', '(', '!', ':']).next().unwrap_or(token))
        .collect();

    for (index, word) in words.iter().enumerate()
    {
        // `const` is the one ambiguous word: `const NAME: T = …` declares an item and
        // `const fn` qualifies one. Reading it as the item keyword turned every
        // `pub const fn` in this workspace into a const named `fn`.
        if *word == "const" && words.get(index.saturating_add(1)) == Some(&"fn")
        {
            modifiers.push("const");
            continue;
        }
        if KEYWORDS.contains(word)
        {
            return Some(((visibility), modifiers.join(" "), (*word).to_owned()));
        }
        if matches!(*word, "unsafe" | "async" | "extern" | "default")
        {
            modifiers.push(word);
            continue;
        }
        return None;
    }

    return None;
}

/// The `=` that starts a declaration's value, if it has one.
fn Assignment(text: &str, masks: &crate::gates::Masks, from: usize, end: usize) -> Option<usize>
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
fn Semicolon(text: &str, masks: &crate::gates::Masks, from: usize, end: usize) -> Option<usize>
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
    masks: &crate::gates::Masks,
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

/// Files one recognised declaration, and descends into it where that is meaningful.
#[allow(clippy::too_many_arguments)]
fn Record(
    head: &Recognised,
    inside: Option<Inside<'_>>,
    module: &mut Module,
    children: &mut Vec<(String, bool)>,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
    text: &str,
    masks: &crate::gates::Masks,
)
{
    // A trait's items and a trait implementation's items are as public as the trait, so
    // they carry no `pub` of their own. An inherent `impl` is not like that.
    let associated = matches!(inside, Some(Inside::Conformance(_) | Inside::Declaration(_)));
    let public = head.visibility == "pub" || (associated && head.visibility.is_empty());

    if head.kind == "mod"
    {
        if let Some(name) = Named(&head.text, "mod")
        {
            match head.body
            {
                Some(body) => Load_Inline(&name, head.visibility == "pub", body, path, into, text, masks),
                None => children.push((name, head.visibility == "pub")),
            }
        }
        return;
    }

    if head.kind == "use"
    {
        if head.visibility == "pub"
            && let Some(target) = head.text.split_once("use ").map(|(_, rest)| return rest)
        {
            module.re_exports.push(target.trim().to_owned());
        }
        return;
    }

    if head.kind == "impl"
    {
        Record_Impl(head, module, path, into, text, masks);
        return;
    }

    if !public
    {
        return;
    }

    let Some(name) = Named(&head.text, &head.kind)
    else
    {
        return;
    };
    let owner = match inside
    {
        Some(
            Inside::Implementation(owner)
            | Inside::Conformance(owner)
            | Inside::Declaration(owner)
            | Inside::Fields(owner),
        ) => format!("{owner}::"),
        _ => String::new(),
    };

    module.items.push(Item {
        kind: head.kind.clone(),
        name: format!("{owner}{name}"),
        tail: Tail(&head.text, &head.kind, &name),
        modifiers: head.modifiers.clone(),
    });

    Descend(head, &name, module, children, path, into, text, masks);
}

/// Walks into the body of a declaration whose members are part of the surface.
#[allow(clippy::too_many_arguments)]
fn Descend(
    head: &Recognised,
    name: &str,
    module: &mut Module,
    children: &mut Vec<(String, bool)>,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
    text: &str,
    masks: &crate::gates::Masks,
)
{
    let Some(body) = head.body
    else
    {
        return;
    };

    let inside = match head.kind.as_str()
    {
        "trait" => Inside::Declaration(name),
        "struct" | "union" => Inside::Fields(name),
        "enum" => Inside::Variants(name),
        _ => return,
    };

    Walk(text, masks, body, Some(inside), module, children, path, into);
}

/// An `impl` block: the header is surface when it implements a trait, and its `pub`
/// members are surface either way.
fn Record_Impl(
    head: &Recognised,
    module: &mut Module,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
    text: &str,
    masks: &crate::gates::Masks,
)
{
    let Some((subject, implemented)) = Implemented(&head.text)
    else
    {
        return;
    };

    let inside = if let Some(trait_name) = implemented
    {
        module.items.push(Item {
            kind: "impl".to_owned(),
            name: subject.clone(),
            tail: format!(" implements {trait_name}"),
            modifiers: String::new(),
        });

        Inside::Conformance(&subject)
    }
    else
    {
        Inside::Implementation(&subject)
    };

    let Some(body) = head.body
    else
    {
        return;
    };

    let mut ignored = Vec::new();
    Walk(text, masks, body, Some(inside), module, &mut ignored, path, into);
}

/// The type an `impl` header is about, and the trait it implements if it implements one.
///
/// The subject loses its generic arguments — `impl<T: Read> Drain<T>` is `Drain` — because
/// it is only being used to prefix the members below it, and those carry their own types.
/// The trait keeps everything: `From<StoreError>` and `From<rusqlite::Error>` are two
/// different implementations, and reducing both to `From` collapses them into one line
/// that a snapshot then reports as unchanged when one of them is deleted.
fn Implemented(head: &str) -> Option<(String, Option<String>)>
{
    let rest = head.strip_prefix("impl")?.trim_start();
    let rest = Without_Generics(rest);
    let body = rest.split(" where ").next().unwrap_or(&rest).trim().to_owned();

    if let Some((implemented, subject)) = body.split_once(" for ")
    {
        return Some((Bare(subject.trim()), Some(implemented.trim().to_owned())));
    }

    return Some((Bare(body.trim()), None));
}

/// A type expression without its generic arguments.
fn Bare(text: &str) -> String
{
    return text
        .split(['<', ' '])
        .next()
        .unwrap_or(text)
        .trim()
        .to_owned();
}

/// A header with a leading `<…>` generic parameter list removed.
fn Without_Generics(text: &str) -> String
{
    if !text.starts_with('<')
    {
        return text.to_owned();
    }

    let mut depth = 0_i32;
    for (offset, character) in text.char_indices()
    {
        match character
        {
            '<' => depth = depth.saturating_add(1),
            '>' =>
            {
                depth = depth.saturating_sub(1);
                if depth == 0
                {
                    return text
                        .get(offset.saturating_add(1)..)
                        .unwrap_or_default()
                        .trim()
                        .to_owned();
                }
            }
            _ =>
            {}
        }
    }

    return text.to_owned();
}

/// An inline `mod name { … }`, loaded as its own module.
#[allow(clippy::too_many_arguments)]
fn Load_Inline(
    name: &str,
    is_public: bool,
    body: (usize, usize),
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
    text: &str,
    masks: &crate::gates::Masks,
)
{
    let mut child_path = path.to_vec();
    child_path.push(name.to_owned());

    let parent_exported = into.get(path).is_some_and(|held| return held.exported) || path.is_empty();
    let mut child = Module {
        exported: parent_exported && is_public,
        ..Module::default()
    };
    let mut grandchildren = Vec::new();

    Walk(text, masks, body, None, &mut child, &mut grandchildren, &child_path, into);

    into.insert(child_path, child);
}

/// The name a declaration gives, taken from just after its keyword.
fn Named(declaration: &str, keyword: &str) -> Option<String>
{
    let at = Keyword_At(declaration, keyword)?;
    let from = at.checked_add(keyword.len())?;
    let (name, _) = Identifier_After(declaration.as_bytes(), from)?;

    return (!name.is_empty()).then_some(name);
}

/// Everything a declaration says after its name.
fn Tail(declaration: &str, keyword: &str, name: &str) -> String
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
fn Keyword_At(declaration: &str, keyword: &str) -> Option<usize>
{
    let mut from = 0_usize;

    while let Some(offset) = declaration.get(from..)?.find(keyword)
    {
        let at = from.checked_add(offset)?;
        let before = at
            .checked_sub(1)
            .and_then(|index| return declaration.as_bytes().get(index).copied())
            .unwrap_or(b' ');
        let after = declaration
            .as_bytes()
            .get(at.saturating_add(keyword.len()))
            .copied()
            .unwrap_or(b' ');

        if !before.is_ascii_alphanumeric()
            && before != b'_'
            && !after.is_ascii_alphanumeric()
            && after != b'_'
        {
            return Some(at);
        }

        from = at.saturating_add(1);
    }

    return None;
}

/// The members of an enum or struct body, with whatever each one carries.
///
/// Read line by line to find each member and then by brace matching to capture what it
/// carries, because this workspace writes Allman braces: a struct variant's fields are on
/// the lines *after* its name, so a purely line-wise reader records `NotRelative` and
/// drops `{ profile: String, output: String }`. A payload's types are as much of the
/// surface as the name in front of them.
///
/// `wants_visibility` separates the two callers. A struct's fields are public one at a
/// time and a private one is not an export; an enum's variants are as public as the enum.
fn Members_In(
    text: &str,
    masks: &crate::gates::Masks,
    range: (usize, usize),
    wants_visibility: bool,
) -> Vec<(String, String)>
{
    let (start, end) = range;
    let mut found = Vec::new();
    let mut cursor = start;

    while cursor < end
    {
        let line_end = Line_End(text, cursor, end);
        let raw = text.get(cursor..line_end).unwrap_or_default();
        let line = raw.trim_start();
        let at = cursor.saturating_add(raw.len().saturating_sub(line.len()));

        let declared = if wants_visibility
        {
            line.strip_prefix("pub ").map(|rest| {
                return (rest, at.saturating_add(4));
            })
        }
        else
        {
            Some((line, at))
        };

        let Some((rest, from)) = declared
        else
        {
            cursor = Next_Line(text, line_end, end);
            continue;
        };

        let Some((name, after_name)) = Identifier_After(text.as_bytes(), from)
        else
        {
            cursor = Next_Line(text, line_end, end);
            continue;
        };

        // A member is a bare identifier at the start of its line. Attributes open with
        // `#`, the closing brace with `}`, and a nested item with a keyword.
        if !rest.starts_with(&name) || KEYWORDS.contains(&name.as_str())
        {
            cursor = Next_Line(text, line_end, end);
            continue;
        }

        let (payload, after) = Payload(text, masks, after_name, end);
        found.push((name, payload));
        cursor = after.max(Next_Line(text, line_end, end)).min(end);
    }

    return found;
}

/// What a member carries, and where the next one starts.
fn Payload(
    text: &str,
    masks: &crate::gates::Masks,
    from: usize,
    end: usize,
) -> (String, usize)
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

        return (carried, close.saturating_add(1));
    }

    let close = match opener
    {
        Some(b'{') => Matching_Brace(bytes, &masks.code, cursor),
        Some(b'(') => Matching_Parenthesis(bytes, &masks.code, cursor),
        _ => return (String::new(), cursor.saturating_add(1)),
    };

    let Some(close) = close
    else
    {
        return (String::new(), end);
    };

    let carried = text
        .get(cursor..=close)
        .map(Collapsed)
        .unwrap_or_default();
    let separator = if opener == Some(b'{') { " " } else { "" };

    return (format!("{separator}{carried}"), close.saturating_add(1));
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

/// The offset of the parenthesis closing the one at `open`.
fn Matching_Parenthesis(bytes: &[u8], mask: &[bool], open: usize) -> Option<usize>
{
    let mut depth = 0_u32;
    let mut cursor = open;

    while cursor < bytes.len()
    {
        if Is_Code(mask, cursor)
        {
            match bytes.get(cursor).copied()
            {
                Some(b'(') => depth = depth.saturating_add(1),
                Some(b')') =>
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

/// A declaration rendered as one line of a snapshot.
fn Emit(item: &Item, prefix: &str, into: &mut BTreeSet<String>)
{
    let modifiers = if item.modifiers.is_empty()
    {
        String::new()
    }
    else
    {
        format!("{} ", item.modifiers)
    };

    into.insert(
        format!("pub {modifiers}{} {prefix}::{}{}", item.kind, item.name, item.tail)
            .trim_end()
            .to_owned(),
    );
}

/// Resolves a `pub use` to the declarations it exports, or records that it could not.
///
/// One name at a time, and that is the whole of `OD-GATE-002` version 2. This used to ask
/// whether the *declaration* had resolved to anything: one flag for the whole list, set by
/// the first name that landed. So `pub use corpus::{Corpus, SourceFile, Subject_Of_Path,
/// Walk}` reported nothing unresolved because three of its four names are declared in this
/// crate, and the fourth — re-exported from `nomos-model`, which this per-crate resolver
/// cannot see into — was neither emitted nor reported. It was dropped, silently, from
/// `tests/contract/surface/nomos-integration-tests.txt` while the crate still exported it.
///
/// That is worse than the false negatives `OD-GATE-002` writes down, because the record
/// promises the opposite for the case it did name: a glob resolves to nothing and is
/// reported rather than dropped. A glob was only ever reported because it is the sole name
/// in its declaration, so the promise held by accident and failed the moment a name that
/// resolves stood beside one that does not.
fn Emit_Re_Export(
    target: &str,
    module: &[String],
    identifier: &str,
    modules: &BTreeMap<Vec<String>, Module>,
    into: &mut BTreeSet<String>,
    unresolved: &mut BTreeSet<String>,
)
{
    // The re-exporting module, not the declaring one. `pub use build::Build` in a lib.rs
    // makes the item `nomos_spec_project::Build`, and recording it under the private
    // module it happens to be written in would describe a path no caller can name.
    let prefix = Prefixed(identifier, module);

    for (route, declared, exported_as) in Named_Imports(target)
    {
        let exported = Locate(&route, &declared, module, modules);
        if exported.is_empty()
        {
            unresolved.insert(Unfollowed(&route, &declared, &exported_as));
            continue;
        }

        for item in exported
        {
            Emit(&Renamed(&item, &declared, &exported_as), &prefix, into);
        }
    }
}

/// One name a re-export could not be followed to, spelled as a `pub use` of its own.
///
/// A grouped list is split rather than reported whole. Half a list resolving and half not
/// is the case this exists for, and one line naming all four names would say that four
/// items are outside this crate when one is — which is the over-reporting mirror of the
/// defect above, and just as unreadable.
///
/// The route is the one this crate wrote, not the one the target eventually lives at.
/// `pub use corpus::Subject_Of_Path` is what the source says and what a reader has to go
/// and look at; following it to `nomos_model` is the resolution this reader is recorded as
/// not doing.
fn Unfollowed(route: &[String], declared: &str, exported_as: &str) -> String
{
    let path = if route.is_empty()
    {
        declared.to_owned()
    }
    else
    {
        format!("{}::{declared}", route.join("::"))
    };

    if declared == exported_as
    {
        return format!("pub use {path}");
    }

    return format!("pub use {path} as {exported_as}");
}

/// An item under the name a re-export gives it.
///
/// `pub use profile::{Section as ProfileSection}` exports a type that is *called*
/// `ProfileSection` by every caller. Recording the declared name would put two unrelated
/// `Section` types on one line of the snapshot, and this crate has exactly that pair —
/// `profile::Section` and `projection::Section`, one of them aliased. Their fields merged.
fn Renamed(item: &Item, declared: &str, exported_as: &str) -> Item
{
    if declared == exported_as
    {
        return item.clone();
    }

    let name = item
        .name
        .strip_prefix(declared)
        .map_or_else(|| return item.name.clone(), |rest| return format!("{exported_as}{rest}"));

    return Item {
        name,
        ..item.clone()
    };
}

/// Every `(module route, declared name, exported name)` a use declaration names.
///
/// `a::b::{X, Y as Z}` is two names under the route `a::b`. A glob is returned as the
/// name `*`, which resolves to nothing and is therefore reported.
fn Named_Imports(target: &str) -> Vec<(Vec<String>, String, String)>
{
    let cleaned = target.trim().trim_end_matches(';').trim();

    let (route, names) = match cleaned.split_once("::{")
    {
        Some((route, list)) =>
        {
            let list = list.trim_end_matches('}');
            (
                route.to_owned(),
                list.split(',').map(str::trim).filter(|name| return !name.is_empty()).map(str::to_owned).collect::<Vec<String>>(),
            )
        }
        None => match cleaned.rsplit_once("::")
        {
            Some((route, name)) => (route.to_owned(), vec![name.trim().to_owned()]),
            None => (String::new(), vec![cleaned.to_owned()]),
        },
    };

    let segments: Vec<String> = route
        .split("::")
        .map(str::trim)
        .filter(|segment| return !segment.is_empty())
        .map(str::to_owned)
        .collect();

    return names
        .into_iter()
        .map(|name| {
            let (declared, exported_as) = name
                .split_once(" as ")
                .map_or((name.as_str(), name.as_str()), |(left, right)| {
                    return (left.trim(), right.trim());
                });

            return (segments.clone(), declared.to_owned(), exported_as.to_owned());
        })
        .collect();
}

/// How far a chain of re-exports is followed before giving up.
///
/// A bound rather than a visited set: the chains here are two links at most, and a
/// counter cannot be defeated by a cycle that renames on every hop.
const CHAIN_LIMIT: u32 = 8;

/// Every declaration a name reaches under a route, trying the routes Rust would try.
///
/// More than one, because re-exporting a type re-exports what is written on it. `pub use
/// catalogue::Catalogue` exports the struct and every `pub fn` in its `impl` block, and a
/// snapshot holding only the struct would not notice a method being added to it.
fn Locate(
    route: &[String],
    name: &str,
    from: &[String],
    modules: &BTreeMap<Vec<String>, Module>,
) -> Vec<Item>
{
    return Located(route, name, from, modules, 0);
}

/// [`Locate`], carrying how many re-exports have already been followed.
fn Located(
    route: &[String],
    name: &str,
    from: &[String],
    modules: &BTreeMap<Vec<String>, Module>,
    depth: u32,
) -> Vec<Item>
{
    let mut trimmed = route.to_vec();
    let anchored_at_root = matches!(trimmed.first().map(String::as_str), Some("crate"));
    if anchored_at_root || matches!(trimmed.first().map(String::as_str), Some("self"))
    {
        trimmed.remove(0);
    }

    let mut candidates: Vec<Vec<String>> = Vec::new();
    if anchored_at_root
    {
        candidates.push(trimmed.clone());
    }
    else
    {
        // Uniform paths: a bare route is tried against the current module first and then
        // against the crate root, which is what `pub use build::Build` in a lib.rs means.
        let mut local = from.to_vec();
        local.extend(trimmed.iter().cloned());
        candidates.push(local);
        candidates.push(trimmed.clone());
        // `super::` and a parent's sibling, spelled without an anchor.
        if let Some((_, parent)) = from.split_last()
        {
            let mut beside = parent.to_vec();
            beside.extend(trimmed.iter().cloned());
            candidates.push(beside);
        }
    }

    let owned = format!("{name}::");

    for candidate in candidates
    {
        let Some(module) = modules.get(&candidate)
        else
        {
            continue;
        };

        let found: Vec<Item> = module
            .items
            .iter()
            .filter(|item| return item.name == name || item.name.starts_with(&owned))
            .cloned()
            .collect();

        if !found.is_empty()
        {
            return found;
        }

        // The module names it and did not declare it: it re-exported it from somewhere
        // further down. `nomos-contracts` is built this way — `src/determinism/mod.rs`
        // gathers four types from four files and the crate root re-exports the gathering
        // — and stopping at the first hop left the whole of that crate's determinism
        // vocabulary out of its snapshot while reporting the re-export as unresolvable.
        if depth < CHAIN_LIMIT
            && let Some(reached) = Through_A_Re_Export(module, name, &candidate, modules, depth)
        {
            return reached;
        }
    }

    return Vec::new();
}

/// Follows a module's own `pub use` declarations looking for `name`.
fn Through_A_Re_Export(
    module: &Module,
    name: &str,
    at: &[String],
    modules: &BTreeMap<Vec<String>, Module>,
    depth: u32,
) -> Option<Vec<Item>>
{
    for re_export in &module.re_exports
    {
        for (route, declared, exported_as) in Named_Imports(re_export)
        {
            if exported_as != name
            {
                continue;
            }

            let found = Located(&route, &declared, at, modules, depth.saturating_add(1));
            if !found.is_empty()
            {
                return Some(
                    found
                        .iter()
                        .map(|item| return Renamed(item, &declared, name))
                        .collect(),
                );
            }
        }
    }

    return None;
}

/// The end of the line starting at `from`, bounded by `end`.
fn Line_End(text: &str, from: usize, end: usize) -> usize
{
    return text
        .get(from..end)
        .and_then(|rest| return rest.find('\n'))
        .map_or(end, |at| return from.saturating_add(at));
}

/// The start of the line after `from`, bounded by `end`.
fn Next_Line(text: &str, from: usize, end: usize) -> usize
{
    let line_end = Line_End(text, from.min(end), end);

    return line_end.saturating_add(1).max(from.saturating_add(1));
}

/// Collapses every run of whitespace to a single space.
fn Collapsed(text: &str) -> String
{
    return text.split_whitespace().collect::<Vec<&str>>().join(" ");
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

    #[test]
    fn Test_A_Use_List_Should_Name_Every_Item_In_It()
    {
        let imports = Named_Imports("build::{Build, Check, SIDECAR_SUFFIX};");

        assert_eq!(imports.len(), 3);
        assert_eq!(
            imports.first().map(|(route, declared, _)| {
                return (route.clone(), declared.clone());
            }),
            Some((vec!["build".to_owned()], "Build".to_owned()))
        );
    }

    /// The alias is what callers write, and two crates here declare a `Section`.
    #[test]
    fn Test_An_Aliased_Import_Should_Keep_Both_Names()
    {
        let imports = Named_Imports("profile::{Section as ProfileSection};");

        assert_eq!(
            imports.first().map(|(_, declared, exported)| {
                return (declared.clone(), exported.clone());
            }),
            Some(("Section".to_owned(), "ProfileSection".to_owned()))
        );
    }

    #[test]
    fn Test_An_Impl_Header_Should_Lose_Its_Generics_And_Keep_Its_Trait()
    {
        assert_eq!(
            Implemented("impl<T: Read + Send> Reader for Drain<T>"),
            Some(("Drain".to_owned(), Some("Reader".to_owned())))
        );
        assert_eq!(Implemented("impl Catalogue"), Some(("Catalogue".to_owned(), None)));
        // Two implementations of one trait must not reduce to one line.
        assert_ne!(
            Implemented("impl From<StoreError> for ProjectError"),
            Implemented("impl From<rusqlite::Error> for ProjectError")
        );
    }

    /// A name that could not be followed is spelled as the `pub use` a reader would go and
    /// look at, one line per name.
    ///
    /// The route is the one the source wrote. `corpus::Subject_Of_Path` is where the search
    /// starts and where somebody checking the report has to begin; `nomos_model`, where the
    /// item is actually declared, is the hop this reader is recorded as not taking.
    #[test]
    fn Test_An_Unfollowed_Name_Should_Be_Spelled_As_Its_Own_Use()
    {
        assert_eq!(
            Unfollowed(&["corpus".to_owned()], "Subject_Of_Path", "Subject_Of_Path"),
            "pub use corpus::Subject_Of_Path"
        );
        // An alias is what callers write, so the report has to carry it or a reader
        // searching for the name they use finds nothing.
        assert_eq!(
            Unfollowed(&["profile".to_owned()], "Section", "ProfileSection"),
            "pub use profile::Section as ProfileSection"
        );
        // A whole crate re-exported by name has no route in front of it.
        assert_eq!(Unfollowed(&[], "serde", "serde"), "pub use serde");
    }

    /// A glob keeps the spelling `OD-GATE-002` version 1 promised for it.
    ///
    /// It resolved to nothing then and resolves to nothing now; what changed is that it is
    /// no longer the only shape that gets reported. If this line moved, the record's one
    /// worked example would have been broken by the repair that generalised it.
    #[test]
    fn Test_A_Glob_Should_Still_Be_Reported_In_Its_Written_Form()
    {
        let imports = Named_Imports("module::*");
        let (route, declared, exported_as) =
            imports.first().expect("a glob is one name").clone();

        assert_eq!(declared, "*");
        assert_eq!(Unfollowed(&route, &declared, &exported_as), "pub use module::*");
    }

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
