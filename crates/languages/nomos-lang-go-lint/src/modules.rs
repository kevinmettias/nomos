//! Which module holds each Go source: the nearest directory at or above it, and within the root,
//! that holds a `go.mod`.
//!
//! Read from the filesystem rather than asked of `go`, measured before this was written: `go list
//! -m -json` in a directory with no `go.mod` answers exit 0 with a module named
//! `command-line-arguments`, so asking `go` which module a directory is in cannot tell "no module"
//! apart from "a module", and asking at all would launch `go` in a repository with no Go in it.

use nomos_platform::FileSystem;
use std::collections::BTreeMap;
use std::path::Path;

/// Each source grouped under the module that holds it, and the sources no module holds.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Modules
{
    /// Module directory, relative to the root and empty for the root, to the sources it holds.
    pub(crate) holding: BTreeMap<String, Vec<String>>,
    /// Sources with no `go.mod` at or above them within the root.
    pub(crate) outside: Vec<String>,
}

/// Groups `files` -- relative to `root`, with forward slashes -- by the module holding each.
pub(crate) fn Group_By_Module<Fs: FileSystem>(root: &Path, files: &[&str], filesystem: &Fs) -> Modules
{
    let mut modules = Modules::default();
    for file in files
    {
        match Module_Directory(root, file, filesystem)
        {
            Some(directory) => modules.holding.entry(directory).or_default().push((*file).to_owned()),
            None => modules.outside.push((*file).to_owned()),
        }
    }

    return modules;
}

/// The directory of the nearest `go.mod` at or above `file`, no higher than `root`.
fn Module_Directory<Fs: FileSystem>(root: &Path, file: &str, filesystem: &Fs) -> Option<String>
{
    let mut directory = Parent(file);
    // Climbs one directory a turn; ends at the first one holding a go.mod, or at the root.
    loop
    {
        if filesystem.Exists(&root.join(&directory).join("go.mod"))
        {
            return Some(directory);
        }
        if directory.is_empty()
        {
            return None;
        }
        directory = Parent(&directory);
    }
}

/// `path`'s parent directory, relative, empty at the top.
fn Parent(path: &str) -> String
{
    return path.rsplit_once('/').map_or_else(String::new, |(parent, _)| return parent.to_owned());
}

/// The module path `root`/`directory`/`go.mod` declares on its `module` line, or `None` when the
/// file cannot be read or declares none -- a label for the fact, so its absence is not a failure.
pub(crate) fn Module_Path<Fs: FileSystem>(root: &Path, directory: &str, filesystem: &Fs) -> Option<String>
{
    let text = filesystem.Read_To_String(&root.join(directory).join("go.mod")).ok()?;
    return text.lines().find_map(|line| {
        let line = line.split("//").next().unwrap_or_default().trim();
        let declared = line.strip_prefix("module")?;
        if !declared.starts_with(char::is_whitespace)
        {
            return None;
        }
        let path = declared.trim().trim_matches('"').trim_matches('`');
        return (!path.is_empty()).then(|| return path.to_owned());
    });
}

#[cfg(test)]
mod tests;
