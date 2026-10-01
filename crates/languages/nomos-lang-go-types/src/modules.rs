//! Which module holds each Go source: the nearest directory at or above it, and within the root,
//! that holds a `go.mod`.
//!
//! The same resolution `nomos-lang-go-lint` makes for `go vet`, for the same measured reason: `go`
//! answers "a module" even in a directory with no `go.mod`, so it cannot be asked which module holds
//! a file, and asking would launch it in a repository with no Go in it. Written again here rather
//! than shared, because a provider names no sibling provider, and the few lines it takes are not
//! worth a crate of their own.

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

#[cfg(test)]
mod tests;
