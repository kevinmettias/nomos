//! Reading a surface, and where its snapshot lives.
//!
//! The three modules beside this one compare a surface, interrogate one, and rewrite one, so
//! all three need to be able to produce one and to say where its file is.

use nomos_contract_tests::{Public_Surface, Surface, Workspace};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the committed snapshots live, relative to this crate.
const SNAPSHOTS: &str = "surface";

/// Naming crates in this rewrites *their* snapshots instead of comparing against them.
///
/// The value is a list of package names, separated by commas or whitespace. It is never a
/// flag: there is no value meaning "all", and a value naming nothing is refused.
pub(crate) const BLESS: &str = "NOMOS_SURFACE_BLESS";

pub(crate) fn Snapshot_Directory() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOTS);
}

/// A snapshot's file inside a directory of snapshots.
///
/// The directory is a parameter so that the bless path can be exercised against a scratch
/// directory. A test that blessed into `surface/` to prove blessing is well-behaved would
/// be committing the very act it is checking for.
pub(crate) fn Snapshot_Path(directory: &Path, package: &str) -> PathBuf
{
    return directory.join(format!("{package}.txt"));
}

/// A snapshot's text: the declarations, then the re-exports nothing here could follow.
pub(crate) fn Rendered(surface: &Surface) -> String
{
    let mut text = String::new();

    for declaration in &surface.declarations
    {
        text.push_str(declaration);
        text.push('\n');
    }

    if !surface.unresolved.is_empty()
    {
        text.push_str("\n# re-exports of items declared outside this crate\n");
        for line in &surface.unresolved
        {
            text.push_str(line);
            text.push('\n');
        }
    }

    return text;
}

/// Every workspace member that has a library, with the surface its source declares.
pub(crate) fn Surfaces() -> Vec<Surface>
{
    let workspace = Workspace::Load();
    let mut found = Vec::new();

    for member in workspace.Members()
    {
        if let Some(surface) = Public_Surface(&member.name, &member.root)
        {
            found.push(surface);
        }
    }

    found.sort_by(|left, right| return left.package.cmp(&right.package));
    return found;
}

/// One workspace member's public surface, read from its own source.
pub(crate) fn Surface_Named(package: &str) -> Surface
{
    let workspace = Workspace::Load();
    let member = workspace
        .Get(package)
        .unwrap_or_else(|| panic!("{package} is a workspace member"));

    return Public_Surface(&member.name, &member.root)
        .unwrap_or_else(|| panic!("{package} has a library"));
}

/// Whether any declaration in a surface carries a needle.
pub(crate) fn Says(surface: &Surface, needle: &str) -> bool
{
    return surface
        .declarations
        .iter()
        .any(|declaration| return declaration.contains(needle));
}

/// Every package this reader produced a public surface for.
pub(crate) fn Snapshotted_Packages() -> BTreeSet<String>
{
    return Surfaces()
        .into_iter()
        .map(|surface| return surface.package)
        .collect();
}
