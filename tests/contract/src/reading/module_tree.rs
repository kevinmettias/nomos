//! A crate's modules, loaded off disk by following its `mod` declarations.
//!
//! The shape everything else here reads: which modules exist, which are exported, and what
//! each one declared. Loading is kept apart from reading because the two fail differently —
//! a module file that cannot be opened is a surface that under-reports, and under-reporting
//! is the direction that flatters, so this half panics where the reading half returns.

use crate::reading::declaration::items::{Filing, Source, Walk};
use crate::reading::masks::{Scan, Without_Comments};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One module of a crate, as its source declares it.
#[derive(Debug, Default)]
pub(crate) struct Module
{
    /// Whether every `mod` from the crate root down to it is `pub`.
    pub(crate) exported: bool,
    /// The public items it declares.
    pub(crate) items: Vec<Item>,
    /// Its `pub use` declarations, as written after `pub use ` and before the `;`.
    pub(crate) re_exports: Vec<String>,
}

/// One public declaration.
#[derive(Clone, Debug)]
pub(crate) struct Item
{
    /// `fn`, `struct`, `enum`, `trait`, `type`, `const`, `static`, `macro`, `impl`,
    /// `struct field` or `enum variant`.
    pub(crate) kind: String,
    /// The name, qualified by its owning type where it has one.
    pub(crate) name: String,
    /// Everything after the name: parameters, return type, field type.
    pub(crate) tail: String,
    /// Anything before the kind keyword that belongs to the declaration, such as `unsafe`
    /// or `async`.
    pub(crate) modifiers: String,
}

/// Reads a module file and every module it declares, into `into`.
pub(crate) fn Load_Module(file: &Path, path: &[String], into: &mut BTreeMap<Vec<String>, Module>)
{
    let cleaned = Readable(file);
    let (mut module, children) = Scanned(&cleaned, path, into);
    let previously_exported = into.get(path).is_some_and(|held| return held.exported);

    module.exported = module.exported || previously_exported;
    let exported = module.exported;
    into.insert(path.to_vec(), module);

    let directory = Module_Directory(file, path);
    let attributed = Path_Attributed_Files(&cleaned, file);
    for (name, is_public) in Children_And_Those_The_Walk_Missed(children, &attributed)
    {
        let child = Child {
            exported: exported && is_public,
            attributed: attributed.get(&name).map(|declared| return declared.file.clone()),
            name,
        };
        Load_Child(&directory, path, child, into);
    }
}

/// The children the walk reported, plus the ones it could not see.
///
/// A `mod` written on the same line as its `#[path]` attribute is invisible to the walk: the
/// line opens with an attribute, so no declaration keyword starts it and the recogniser steps
/// over the whole line. Both spellings are written in this workspace, and reading the
/// attribute alone is not enough to fix the same-line one — a file nothing queues is a file
/// nothing loads — so the modules the walk missed are added here from the attributes.
///
/// A name the walk did report keeps the walk's own visibility, which is the reading that saw
/// the declaration in its context rather than a line at a time.
fn Children_And_Those_The_Walk_Missed(
    children: Vec<(String, bool)>,
    attributed: &BTreeMap<String, Declared>,
) -> Vec<(String, bool)>
{
    let mut complete = children;

    for (name, declared) in attributed
    {
        if !complete.iter().any(|(seen, _)| return seen == name)
        {
            complete.push((name.clone(), declared.public));
        }
    }

    return complete;
}

/// A module a `#[path]` attribute backs: the file it names and how it was declared.
struct Declared
{
    /// What the attribute named, resolved against the declaring file's directory.
    file: PathBuf,
    /// Whether the declaration was plain `pub`. A `pub(crate)` or `pub(super)` module is a
    /// restriction rather than an export and answers false, so it can still be crossed by a
    /// route without carrying the export flag down.
    public: bool,
}

/// Every `mod` in this file that a `#[path]` attribute backs, as the file it names.
///
/// Resolved against the directory the *declaring* file sits in, which is what the language
/// does and is not the directory this module's other children resolve against:
/// `src/words.rs` declaring `#[path = "words/fact_context.rs"] mod provider;` means
/// `src/words/fact_context.rs`, while its plain `mod guarantee;` means the same thing by
/// way of `src/words/`. Reading one base for both would resolve every attribute one level
/// too deep.
fn Path_Attributed_Files(cleaned: &str, file: &Path) -> BTreeMap<String, Declared>
{
    let beside = file.parent().unwrap_or(Path::new("."));

    return Path_Attributed_Modules(cleaned)
        .into_iter()
        .map(|(name, (relative, public))| {
            return (
                name,
                Declared {
                    file: beside.join(relative),
                    public,
                },
            );
        })
        .collect();
}

/// A module file's text, with test module bodies and then comments blanked in place.
///
/// Blanked rather than removed so every byte offset still means the same place. Comments have
/// to go: this workspace documents almost every item, and a doc comment reading "/// Returns
/// the pub fn's answer" is prose that a scanner looking for declarations would otherwise take
/// at its word.
///
/// # Panics
///
/// Panics if the file cannot be read.
fn Readable(file: &Path) -> String
{
    use crate::reading::source_files::Without_Test_Modules;

    let text = std::fs::read_to_string(file)
        // rust-panic: allow: this path was produced by resolving a `mod` declaration the walk
        // already read, so a file that will not open means the module tree and the tree on disk
        // disagree. Falling back to empty text would drop a whole module out of every surface
        // snapshot below, and a snapshot missing a module compares equal to one nobody removed
        // anything from.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", file.display()));
    let without_tests = Without_Test_Modules(&text);

    return Without_Comments(&without_tests, &Scan(&without_tests).comment);
}

/// One module's own declarations, and the child modules it names.
fn Scanned(
    cleaned: &str,
    path: &[String],
    into: &mut BTreeMap<Vec<String>, Module>,
) -> (Module, Vec<(String, bool)>)
{
    let masks = Scan(cleaned);
    let mut module = Module {
        // The root is exported by definition; a child's flag is set by its parent below.
        exported: path.is_empty(),
        ..Module::default()
    };
    let mut children: Vec<(String, bool)> = Vec::new();
    Walk(
        Source {
            text: cleaned,
            masks: &masks,
        },
        (0, cleaned.len()),
        None,
        &mut Filing {
            module: &mut module,
            children: &mut children,
            path,
            into,
        },
    );

    return (module, children);
}

/// A child module as its parent declares it: its name, whether the chain of `mod`
/// declarations from the crate root down to it is public the whole way, and the file a
/// `#[path]` attribute names for it if one does.
struct Child
{
    name: String,
    exported: bool,
    /// What `#[path]` names, already resolved. `None` for the ordinary case, where the two
    /// layouts [`Module_File`] knows are the whole answer.
    attributed: Option<PathBuf>,
}

/// Loads one child module, whether or not a file backs it.
fn Load_Child(
    directory: &Path,
    path: &[String],
    child: Child,
    into: &mut BTreeMap<Vec<String>, Module>,
)
{
    let Child { name, exported, attributed } = child;
    let mut child_path = path.to_vec();
    child_path.push(name.clone());
    // Recorded before the file is read, so that a module whose file is missing still carries
    // its visibility and an inline `mod` block already loaded keeps it.
    let entry = into.entry(child_path.clone()).or_default();
    entry.exported = entry.exported || exported;

    // The attribute first, because it is what the language reads: a `#[path]` module is
    // named for what it *is* rather than for where it lives, so neither layout below finds
    // it and the module used to be registered under its name holding nothing. An attribute
    // naming a file that is not there falls through rather than being taken at its word,
    // which keeps an unresolvable declaration reported the way it already was.
    let Some(child_file) = Backing_File(attributed, directory, &name)
    else
    {
        return;
    };
    let inherited = into.get(&child_path).is_some_and(|held| return held.exported);

    Load_Module(&child_file, &child_path, into);
    if inherited && let Some(held) = into.get_mut(&child_path)
    {
        held.exported = true;
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

/// The file backing a child module: what `#[path]` named, or either of the two layouts.
fn Backing_File(attributed: Option<PathBuf>, directory: &Path, name: &str) -> Option<PathBuf>
{
    if let Some(named) = attributed.filter(|candidate| return candidate.is_file())
    {
        return Some(named);
    }

    return Module_File(directory, name);
}

/// The file backing a child module, in either of the two layouts.
fn Module_File(directory: &Path, name: &str) -> Option<PathBuf>
{
    return [directory.join(format!("{name}.rs")), directory.join(name).join("mod.rs")]
        .into_iter()
        .find(|candidate| return candidate.is_file());
}

/// Every `mod NAME;` a `#[path]` attribute backs, as the relative file the attribute names.
///
/// Read line by line rather than through [`crate::reading::recogniser`], because an
/// attribute is not a declaration: the recogniser sees no keyword on that line and the walk
/// steps over it, which is exactly how the attribute came to be invisible.
///
/// Only a file-backed `mod` counts. An inline `mod name { … }` already has its body here and
/// an attribute on one means nothing.
fn Path_Attributed_Modules(cleaned: &str) -> BTreeMap<String, (String, bool)>
{
    let mut attributed = BTreeMap::new();
    let lines: Vec<&str> = cleaned.lines().collect();

    for (at, line) in lines.iter().enumerate()
    {
        let Some(attribute) = Path_Attribute(line)
        else
        {
            continue;
        };
        if let Some((name, public)) = Module_Named_After(attribute.rest, &lines, at)
        {
            attributed.insert(name, (attribute.relative, public));
        }
    }

    return attributed;
}

/// One `#[path = "…"]` attribute: the file it names, and whatever follows it on its own line.
///
/// The remainder is carried because both spellings are written in this workspace — the
/// attribute alone above its `mod`, and the attribute and the `mod` together on one line —
/// and a reader that looked only at the following lines resolves the first and silently
/// misses the second.
struct Attribute<'a>
{
    /// The file the attribute names, relative to the declaring file's directory.
    relative: String,
    /// The rest of the attribute's own line, where a same-line `mod` sits.
    rest: &'a str,
}

/// The `#[path = "…"]` attribute a line opens with, if it is one.
fn Path_Attribute(line: &str) -> Option<Attribute<'_>>
{
    let trimmed = line.trim_start();
    if !trimmed.starts_with("#[path")
    {
        return None;
    }
    let (_, quoted) = trimmed.split_once('"')?;
    let (value, after) = quoted.split_once('"')?;

    return Some(Attribute {
        relative: value.to_owned(),
        rest: after.trim_start().strip_prefix(']').unwrap_or(after),
    });
}

/// The name of the first file-backed `mod` at or after the attribute, skipping what is neither.
///
/// The attribute's own remainder is tried first, then the lines below it. Attributes stack, so
/// a `#[cfg(test)]` sitting between the attribute and its `mod` is skipped rather than ending
/// the search.
fn Module_Named_After(rest: &str, lines: &[&str], at: usize) -> Option<(String, bool)>
{
    let following = lines.iter().skip(at.saturating_add(1)).copied();

    for line in std::iter::once(rest).chain(following)
    {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#')
        {
            continue;
        }

        return Module_Declared_By(trimmed);
    }

    return None;
}

/// The module a `mod NAME;` line declares — its name and whether it is `pub` — and nothing
/// for any other line.
fn Module_Declared_By(trimmed: &str) -> Option<(String, bool)>
{
    let (without_visibility, public) = Visibility_Stripped(trimmed);
    let named = without_visibility.strip_prefix("mod ")?;
    let (name, _) = named.split_once(';')?;

    return Some((name.trim().to_owned(), public));
}

/// A declaration with its visibility removed, and whether that visibility was plain `pub`.
///
/// `pub(crate)` and `pub(super)` strip like `pub` and answer false. They are restrictions
/// rather than exports, so the module exists and a route may cross it while the export flag
/// stops there — which is the same answer the walk gives for them.
fn Visibility_Stripped(trimmed: &str) -> (&str, bool)
{
    if let Some(restricted) = trimmed.strip_prefix("pub(")
    {
        let after = restricted.split_once(')').map_or(restricted, |(_, rest)| return rest);

        return (after.trim_start(), false);
    }

    return match trimmed.strip_prefix("pub ")
    {
        Some(exported) => (exported.trim_start(), true),
        None => (trimmed, false),
    };
}
