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
use crate::reading::routes::{Emit, Emit_Re_Export, Emitting, Prefixed, Tree};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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
    use crate::reading::module_tree::Load_Module;

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
    let mut emitting = Emitting {
        into: &mut declarations,
        unresolved: &mut unresolved,
    };
    for path in modules.keys()
    {
        let tree = Tree {
            from: path,
            modules: &modules,
        };
        Emit_Module(&identifier, tree, &mut emitting);
    }

    return Some(Surface {
        package: package.to_owned(),
        declarations: declarations.into_iter().collect(),
        unresolved: unresolved.into_iter().collect(),
    });
}

/// Everything one exported module contributes: its own declarations, then its re-exports.
///
/// A module that is not exported contributes nothing, and one the tree does not hold cannot
/// be asked.
fn Emit_Module(identifier: &str, tree: Tree<'_>, emitting: &mut Emitting<'_>)
{
    let Some(module) = tree.modules.get(tree.from).filter(|held| return held.exported)
    else
    {
        return;
    };
    let prefix = Prefixed(identifier, tree.from);

    for item in &module.items
    {
        Emit(item, &prefix, emitting.into);
    }
    for re_export in &module.re_exports
    {
        Emit_Re_Export(re_export, identifier, tree, emitting);
    }
}

