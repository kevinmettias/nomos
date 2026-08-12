//! Following a `pub use` to the declaration it re-exports.
//!
//! The half of the surface that is not written where it is declared. A re-exported name
//! this resolver cannot follow is *reported* rather than dropped — a re-export nobody could
//! follow is the one place a leak would hide, and a list whose other names resolve is
//! exactly where a dropped one hides best.

use crate::module_tree::{Item, Module};
use std::collections::{BTreeMap, BTreeSet};

/// A declaration rendered as one line of a snapshot.
/// The path a declaration in `module` is written under.
pub(crate) fn Prefixed(identifier: &str, module: &[String]) -> String
{
    if module.is_empty()
    {
        return identifier.to_owned();
    }

    return format!("{identifier}::{}", module.join("::"));
}

pub(crate) fn Emit(item: &Item, prefix: &str, into: &mut BTreeSet<String>)
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
pub(crate) fn Emit_Re_Export(target: &str, identifier: &str, tree: Tree<'_>, emitting: &mut Emitting<'_>)
{
    let module = tree.from;
    let modules = tree.modules;
    let Emitting { into, unresolved } = emitting;
    // The re-exporting module, not the declaring one. `pub use build::Build` in a lib.rs
    // makes the item `nomos_spec_project::Build`, and recording it under the private
    // module it happens to be written in would describe a path no caller can name.
    let prefix = Prefixed(identifier, module);

    for (route, declared, exported_as) in Named_Imports(target)
    {
        let exported = Locate(&route, &declared, module, modules);
        if exported.is_empty()
        {
            let unfollowed = Unfollowed(&route, &declared, &exported_as);
            unresolved.insert(unfollowed);
            continue;
        }

        for item in exported
        {
            let renamed = Renamed(&item, &declared, &exported_as);
            Emit(&renamed, &prefix, into);
        }
    }
}

/// The two sets a re-export writes into: the declarations it exports, and the names it
/// could not be followed to.
pub(crate) struct Emitting<'a>
{
    pub(crate) into: &'a mut BTreeSet<String>,
    pub(crate) unresolved: &'a mut BTreeSet<String>,
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
    let (route, names) = Route_And_Names(cleaned);
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

/// The module route a use declaration names, and the names it takes from that route.
fn Route_And_Names(cleaned: &str) -> (String, Vec<String>)
{
    if let Some((route, list)) = cleaned.split_once("::{")
    {
        let names = list
            .trim_end_matches('}')
            .split(',')
            .map(str::trim)
            .filter(|name| return !name.is_empty())
            .map(str::to_owned)
            .collect();

        return (route.to_owned(), names);
    }
    let Some((route, name)) = cleaned.rsplit_once("::")
    else
    {
        return (String::new(), vec![cleaned.to_owned()]);
    };

    return (route.to_owned(), vec![name.trim().to_owned()]);
}

/// The module tree a lookup runs against, and the module it starts from.
///
/// One value because a route is resolved relative to somewhere: the same `build::Build`
/// means different declarations depending on which module wrote it, so the map and the
/// starting module are never separately useful.
#[derive(Clone, Copy)]
pub(crate) struct Tree<'a>
{
    pub(crate) from: &'a [String],
    pub(crate) modules: &'a BTreeMap<Vec<String>, Module>,
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
    return Located(route, name, Tree { from, modules }, 0);
}

/// [`Locate`], carrying how many re-exports have already been followed.
fn Located(route: &[String], name: &str, tree: Tree<'_>, depth: u32) -> Vec<Item>
{
    let Tree { from, modules } = tree;

    for candidate in Candidates(route, from)
    {
        let reached = Answered_By(&candidate, name, modules, depth);
        if !reached.is_empty()
        {
            return reached;
        }
    }

    return Vec::new();
}

/// The module paths a route could mean, in the order Rust would try them.
fn Candidates(route: &[String], from: &[String]) -> Vec<Vec<String>>
{
    let mut trimmed = route.to_vec();
    let anchored_at_root = matches!(trimmed.first().map(String::as_str), Some("crate"));
    if anchored_at_root || matches!(trimmed.first().map(String::as_str), Some("self"))
    {
        trimmed.remove(0);
    }
    if anchored_at_root
    {
        return vec![trimmed];
    }
    // Uniform paths: a bare route is tried against the current module first and then against
    // the crate root, which is what `pub use build::Build` in a lib.rs means.
    let mut local = from.to_vec();
    local.extend(trimmed.iter().cloned());
    let mut candidates = vec![local, trimmed.clone()];
    // `super::` and a parent's sibling, spelled without an anchor.
    if let Some((_, parent)) = from.split_last()
    {
        let mut beside = parent.to_vec();
        beside.extend(trimmed.iter().cloned());
        candidates.push(beside);
    }

    return candidates;
}

/// What one candidate module answers for a name: what it declares, and failing that, what its
/// own `pub use` declarations reach.
///
/// A module that names it without declaring it re-exported it from somewhere further down.
/// `nomos-contracts` is built that way — `src/determinism/mod.rs` gathers four types from
/// four files and the crate root re-exports the gathering — and stopping at the first hop
/// left the whole of that crate's determinism vocabulary out of its snapshot while reporting
/// the re-export as unresolvable.
fn Answered_By(
    candidate: &[String],
    name: &str,
    modules: &BTreeMap<Vec<String>, Module>,
    depth: u32,
) -> Vec<Item>
{
    let Some(module) = modules.get(candidate)
    else
    {
        return Vec::new();
    };
    let declared = Declared_As(module, name);
    if !declared.is_empty() || depth >= CHAIN_LIMIT
    {
        return declared;
    }
    let reachable = Tree {
        from: candidate,
        modules,
    };

    return Through_A_Re_Export(module, name, reachable, depth).unwrap_or_default();
}

/// Everything one module declares under a name.
///
/// More than the item itself, because re-exporting a type re-exports what is written on it.
/// `pub use catalogue::Catalogue` exports the struct and every `pub fn` in its `impl` block,
/// and a snapshot holding only the struct would not notice a method being added to it.
fn Declared_As(module: &Module, name: &str) -> Vec<Item>
{
    let owned = format!("{name}::");

    return module
        .items
        .iter()
        .filter(|item| return item.name == name || item.name.starts_with(&owned))
        .cloned()
        .collect();
}

/// Follows a module's own `pub use` declarations looking for `name`.
fn Through_A_Re_Export(
    module: &Module,
    name: &str,
    tree: Tree<'_>,
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

            let found = Located(&route, &declared, tree, depth.saturating_add(1));
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

#[cfg(test)]
mod tests
{
    use super::*;

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
}
