//! A crate's modules, loaded off disk by following its `mod` declarations.
//!
//! The shape everything else here reads: which modules exist, which are exported, and what
//! each one declared. Loading is kept apart from reading because the two fail differently —
//! a module file that cannot be opened is a surface that under-reports, and under-reporting
//! is the direction that flatters, so this half panics where the reading half returns.

use crate::reading::declaration::items::{Filing, Source, Walk};
use crate::reading::masks::{Scan, Without_Comments};
use crate::reading::source_files::Without_Test_Modules;
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
    for (name, is_public) in children
    {
        let child = Child {
            name,
            exported: exported && is_public,
        };
        Load_Child(&directory, path, child, into);
    }
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
    let text = std::fs::read_to_string(file)
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

/// A child module as its parent declares it: its name, and whether the chain of `mod`
/// declarations from the crate root down to it is public the whole way.
struct Child
{
    name: String,
    exported: bool,
}

/// Loads one child module, whether or not a file backs it.
fn Load_Child(
    directory: &Path,
    path: &[String],
    child: Child,
    into: &mut BTreeMap<Vec<String>, Module>,
)
{
    let Child { name, exported } = child;
    let mut child_path = path.to_vec();
    child_path.push(name.clone());
    // Recorded before the file is read, so that a module whose file is missing still carries
    // its visibility and an inline `mod` block already loaded keeps it.
    let entry = into.entry(child_path.clone()).or_default();
    entry.exported = entry.exported || exported;

    let Some(child_file) = Module_File(directory, &name)
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

/// The file backing a child module, in either of the two layouts.
fn Module_File(directory: &Path, name: &str) -> Option<PathBuf>
{
    return [directory.join(format!("{name}.rs")), directory.join(name).join("mod.rs")]
        .into_iter()
        .find(|candidate| return candidate.is_file());
}
