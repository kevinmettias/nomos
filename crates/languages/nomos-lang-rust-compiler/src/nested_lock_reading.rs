//! Asking the same loaded `ra_ap_hir` analysis [`crate::reading`] built for `.clone()`
//! calls a different question: does a `std::sync::Mutex<T>` or `std::sync::RwLock<T>`
//! guard a `T` that is itself already a `Mutex` or `RwLock`.
//!
//! # Why this needs resolved semantics, not a syntax tree
//!
//! Direct nesting -- `Mutex<Mutex<i32>>` written out in full -- is visible to a syntax
//! tree already. What is not is `Mutex<GuardedCounter>` where `GuardedCounter` is a type
//! alias for `Mutex<i32>` declared anywhere else in the crate, or reached through a
//! re-export: nothing at the nesting site is spelled `Mutex` a second time, so a
//! syntax-only scan comparing type names literally cannot see it. `ra_ap_hir`'s own type
//! lowering substitutes a type alias away before this reader ever asks a question, since
//! an alias has no existence at the type level -- verified against a real fixture crate
//! rather than assumed from how aliases are documented to behave, in
//! `tests::Test_Discover_Nested_Locks_Should_Find_Exactly_The_Real_Nested_Lock`.
//!
//! # Why there is no lang item for `Mutex`/`RwLock` to resolve through
//!
//! `Type::is_copy` (`crate::reading::Clone_On_Copy`) resolves `Copy` through
//! `LangItem::Copy` because `Copy` is one of the traits `rustc`, and therefore
//! `ra_ap_hir`, treats as load-bearing to the language itself. `Mutex` and `RwLock` are
//! ordinary library structs with no such standing -- there is no `LangItem::Mutex` to
//! ask for. [`Lock_Structs`] resolves them the way rust-analyzer's own IDE-facing code
//! resolves any other "well-known" library item it has no lang item for
//! (`ra_ap_ide_db::famous_defs::FamousDefs`, which this reader's own technique mirrors
//! rather than depends on, since `FamousDefs` itself only exposes marker traits like
//! `Copy`/`Sized`, not library structs): walk the standard library's own module tree by
//! name and compare the resolved item's identity, not its spelling.

use crate::payload::nested_lock_finding::NestedLockFinding;
use crate::reading::{CompilerError, Load_Crate};
use nomos_platform::Environment;
use line_index::LineIndex;
use ra_ap_hir::{Adt, EditionedFileId, ModuleDef, ScopeDef, Semantics, Struct};
use ra_ap_ide_db::RootDatabase;
use ra_ap_ide_db::famous_defs::FamousDefs;
use ra_ap_syntax::ast::{self, AstNode};
use std::path::Path;

/// Every `std::sync::Mutex<T>` or `std::sync::RwLock<T>` in the crate rooted at `root`
/// whose `T` a real compiler frontend resolved to another `std::sync::Mutex<U>` or
/// `std::sync::RwLock<U>`.
///
/// # Errors
///
/// [`CompilerError`] if `root` cannot be loaded as a Cargo project, if loading it could
/// not discover a real sysroot, or if no loaded crate's own dependency graph reached a
/// real `std` this reader could resolve `Mutex`/`RwLock` against -- in every one of these
/// cases the honest answer is a refusal, not a silent "no nesting found".
pub fn Discover_Nested_Locks<Env: Environment>(root: &Path, environment: &Env) -> Result<Vec<NestedLockFinding>, CompilerError>
{
    let (db, files) = Load_Crate(root, environment)?;
    let sema: Semantics<'_, RootDatabase> = Semantics::new(&db);

    let Some((mutex, rwlock)) = Lock_Structs(&sema)
    else
    {
        return Err(CompilerError {
            reason: "could not resolve `std::sync::Mutex`/`std::sync::RwLock` -- no loaded crate's own dependency graph reached a real standard library".to_owned(),
        });
    };

    let mut locations = Locations_Of(&sema, &files, mutex, rwlock);
    locations.sort();

    return Ok(locations
        .into_iter()
        .map(|(path, line, col)| return NestedLockFinding { location: format!("{path}:{}:{}", line.saturating_add(1), col.saturating_add(1)) })
        .collect());
}

/// Resolves `std::sync::Mutex` and `std::sync::RwLock` as real, identity-comparable
/// `Struct`s -- not by name, which a user crate could shadow with its own `Mutex`, but by
/// walking the real standard library's own `sync` module and taking the item actually
/// declared there.
///
/// Tries every crate the sysroot loader resolved rather than deriving one from a
/// specific file: `FamousDefs::std` only searches a crate's *direct* dependencies for
/// one with a `Lang(Std)` origin, and while every ordinary (non `#![no_std]`) crate the
/// loader builds a graph for has `std` wired in as exactly that, getting from "a file
/// this reader already has" to "the `hir::Crate` that owns it" needs a `FileId` --
/// `Semantics::file_to_module_def`'s own bound resolves to a different re-export of that
/// type than the one this crate's own `EditionedFileId` carries, so rather than force the
/// two to unify, this walks `Crate::all` (small: the sysroot's own handful of crates plus
/// the one real crate under analysis) and asks each in turn, keeping whichever first
/// resolves a real `std` -- sysroot crates like `core`/`alloc` never will, since neither
/// depends on `std`.
fn Lock_Structs(sema: &Semantics<'_, RootDatabase>) -> Option<(Struct, Struct)>
{
    let std_crate = ra_ap_hir::Crate::all(sema.db).into_iter().find_map(|krate| return FamousDefs(sema, krate).std())?;

    let sync_module = std_crate
        .root_module()
        .children(sema.db)
        .find(|child| return child.name(sema.db).is_some_and(|name| return name.as_str() == "sync"))?;

    let mutex = Find_Struct(sema, sync_module, "Mutex")?;
    let rwlock = Find_Struct(sema, sync_module, "RwLock")?;
    return Some((mutex, rwlock));
}

fn Find_Struct(sema: &Semantics<'_, RootDatabase>, module: ra_ap_hir::Module, name: &str) -> Option<Struct>
{
    return module.scope(sema.db, None).into_iter().find_map(|(item_name, def)| {
        if item_name.as_str() != name
        {
            return None;
        }
        return match def
        {
            ScopeDef::ModuleDef(ModuleDef::Adt(Adt::Struct(found))) => Some(found),
            _ => None,
        };
    });
}

/// Every nested-lock site in `files`, still as a raw, zero-based `(path, line, col)`
/// triple: the shape [`Discover_Nested_Locks`] both orders and renders, gathered in one
/// pass so that function reads as the three steps it really is.
fn Locations_Of(sema: &Semantics<'_, RootDatabase>, files: &[(EditionedFileId, String)], mutex: Struct, rwlock: Struct) -> Vec<(String, u32, u32)>
{
    let mut locations: Vec<(String, u32, u32)> = Vec::new();
    for (editioned, path_str) in files
    {
        let source_file = sema.parse(*editioned);
        let line_index = LineIndex::new(&source_file.syntax().text().to_string());

        for type_node in source_file.syntax().descendants().filter_map(ast::Type::cast)
        {
            if Nested_Lock(sema, &type_node, mutex, rwlock)
            {
                let start = type_node.syntax().text_range().start();
                let position = line_index.line_col(start);
                locations.push((path_str.clone(), position.line, position.col));
            }
        }
    }

    return locations;
}

/// Whether `type_node`'s own resolved type is a `Mutex`/`RwLock` whose type argument
/// itself resolved to a `Mutex`/`RwLock`.
fn Nested_Lock(sema: &Semantics<'_, RootDatabase>, type_node: &ast::Type, mutex: Struct, rwlock: Struct) -> bool
{
    let Some(resolved) = sema.resolve_type(type_node)
    else
    {
        return false;
    };
    let Some(Adt::Struct(outer)) = resolved.as_adt()
    else
    {
        return false;
    };
    if outer != mutex && outer != rwlock
    {
        return false;
    }

    return resolved
        .type_arguments()
        .any(|argument| return matches!(argument.as_adt(), Some(Adt::Struct(inner)) if inner == mutex || inner == rwlock));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The one real, end-to-end assertion this reader owes: given a crate with one
    /// `Mutex` nested behind a type alias and one plain, unnested `Mutex`, a real
    /// `ra_ap_hir` analysis finds exactly the first -- not zero (the sysroot-not-loaded
    /// or std-not-resolved failure this reader's own module doc names), not both (a
    /// naive "any two `Mutex<...>` in this file" heuristic would overcount), exactly one.
    #[test]
    fn Test_Discover_Nested_Locks_Should_Find_Exactly_The_Real_Nested_Lock()
    {
        let findings = Discover_Nested_Locks(&Fixture_Root(), &nomos_platform_std::StdEnvironment).expect("this crate's own fixture is a real, loadable Cargo project");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert!(found.location.ends_with("registry.rs:13:19"), "{}", found.location);
    }

    fn Fixture_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.join("fixtures").join("nested_lock_sample");
    }

    #[test]
    fn Test_Discover_Nested_Locks_Should_Refuse_A_Root_With_No_Cargo_Project()
    {
        // The same reasoning `crate::reading::tests` already gives for the identical
        // shape of test: a directory nested under this crate's own `src/` would find
        // this crate's own `Cargo.toml` walking upward, so the negative case needs a
        // directory with no ancestor Cargo project at all.
        let root = std::env::temp_dir().join("nomos-lang-rust-compiler-nested-locks-no-cargo-project-test");
        std::fs::create_dir_all(&root).expect("the platform temporary directory is writable");

        let error = Discover_Nested_Locks(&root, &nomos_platform_std::StdEnvironment).expect_err("a directory with no Cargo.toml anywhere above it is not a loadable project");

        assert!(error.reason.contains("could not be loaded"), "{}", error.reason);
    }
}
