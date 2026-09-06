//! Loading a crate through `ra_ap_hir` -- rust-analyzer's own semantic-analysis engine,
//! published as a library rather than vendored -- and asking it, not a syntax tree,
//! which `.clone()` calls duplicate a value that already implements `Copy`.
//!
//! # Why this reader resolves the whole call expression's type, not the receiver's
//!
//! A receiver spelled `some_reference` almost always has a reference type (`&T`), and a
//! shared reference is unconditionally `Copy` regardless of whether `T` is -- checking
//! `Type::is_copy` on the receiver's own type would report every `.clone()` call on a
//! borrowed value as cloning a `Copy` type, which is the opposite of what this capability
//! promises. `Clone::clone(&self) -> Self` means the *call expression's* resolved type is
//! always `Self`, already dereferenced past whatever reference the receiver was written
//! through -- verified directly against a real fixture crate before this reader was
//! written, not assumed from the trait's signature alone, in
//! `tests::Test_Discover_Crate_Should_Find_Exactly_The_Real_Clone_On_Copy_Call`.
//!
//! # Why this reader loads a real sysroot
//!
//! `Type::is_copy` resolves `Copy` through `LangItem::Copy`, which only exists once
//! `core` itself is part of the loaded crate graph -- without a sysroot, every type in
//! every crate reads as not `Copy`, silently, which is the specific failure this
//! reader's own test would have shipped undetected had it not asserted a real positive
//! case rather than only the absence of false ones.
//!
//! # Why this reader filters by path
//!
//! `ra_ap_load-cargo::load_workspace_at` loads the whole resolved crate graph a real
//! sysroot pulls in -- `core`, `std`, `alloc`, `test` -- and `Vfs::iter` walks every file
//! in it, sysroot source included. Restricting to files under `root` is what makes this
//! reader answer for the one crate it was asked about rather than for the standard
//! library it had to load to answer honestly.
//!
//! # `Load_Crate` is shared with a second capability
//!
//! Everything above this line is what makes loading a crate through `ra_ap_hir` honest --
//! sysroot discovery, the Windows verbatim-path trap, restricting to the one crate that
//! was asked about. None of that is specific to `.clone()`/`Copy`. [`Load_Crate`] is that
//! shared part, factored out so `crate::nested_lock_reading::Discover_Nested_Locks` asks
//! the same loaded [`Semantics`] a different question rather than re-solving sysroot
//! discovery a second time. It hands back [`ra_ap_hir::EditionedFileId`] rather than the
//! raw `Vfs`/`FileId` pair `ra_ap_load-cargo` returns, because `EditionedFileId` is the
//! one of the two this crate can name without adding `ra_ap_span` as a direct dependency
//! only to spell one type.

use crate::payload::cloned_copy_type::ClonedCopyType;
use line_index::LineIndex;
use ra_ap_hir::{EditionedFileId, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, RustLibSource};
use ra_ap_syntax::ast::{self, AstNode};
use std::path::Path;

/// This provider's own analysis could not be run, or could not be trusted once it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerError
{
    pub reason: String,
}

impl core::fmt::Display for CompilerError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Loads the Cargo project rooted at `root` through `ra_ap_hir` and returns its own
/// database alongside every real `.rs` file that belongs to it -- sysroot source and
/// every other crate the sysroot pulled in already excluded, each file already carrying
/// the edition its own crate resolved to.
///
/// Shared by every capability this crate answers: seeing this once, honestly, is what
/// lets [`Discover_Crate`] and `crate::nested_lock_reading::Discover_Nested_Locks` differ
/// only in which question they ask the same loaded [`Semantics`], not in how they got
/// there.
///
/// # Errors
///
/// [`CompilerError`] if `root` cannot be loaded as a Cargo project, or if loading it
/// could not discover a real sysroot -- without one, a lang-item or well-known-path
/// lookup resolves nothing for anything, which would make every answer built on this
/// loader a silent, unearned "no".
pub(crate) fn Load_Crate(root: &Path) -> Result<(RootDatabase, Vec<(EditionedFileId, String)>), CompilerError>
{
    // The exact absolutization `ra_ap_load_cargo::load_workspace_at` performs on `root`
    // internally before resolving it -- not `std::fs::canonicalize`, whose Windows
    // implementation returns a `\\?\`-prefixed verbatim path that a plain `VfsPath`
    // rendering never carries, which silently broke every prefix match below until this
    // was verified against a real fixture crate rather than assumed to line up.
    let absolute_root = std::env::current_dir()
        .map_err(|error| CompilerError { reason: format!("the current directory could not be read: {error}") })?
        .join(root);

    let cargo_config = CargoConfig { sysroot: Some(RustLibSource::Discover), ..CargoConfig::default() };
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
    };

    let (db, vfs, _proc_macro) = load_workspace_at(root, &cargo_config, &load_config, &|_progress| {}).map_err(|error| CompilerError {
        reason: format!("`{}` could not be loaded as a Cargo project: {error}", root.display()),
    })?;

    let sema: Semantics<'_, RootDatabase> = Semantics::new(&db);

    let mut files: Vec<(EditionedFileId, String)> = Vec::new();
    for (file_id, vfs_path) in vfs.iter()
    {
        let path_str = vfs_path.to_string();
        let is_rust_source = std::path::Path::new(&path_str).extension().is_some_and(|extension| return extension.eq_ignore_ascii_case("rs"));
        if !is_rust_source || !std::path::Path::new(&path_str).starts_with(&absolute_root)
        {
            continue;
        }

        let Some(editioned) = sema.attach_first_edition(file_id)
        else
        {
            continue;
        };
        files.push((editioned, path_str));
    }

    files.sort_by(|left, right| return left.1.cmp(&right.1));

    return Ok((db, files));
}

/// Every `.clone()` call in the crate rooted at `root` whose call expression --
/// `Clone::clone`'s own `Self` return type, not the receiver's possibly-reference type --
/// a real compiler frontend resolved to a type that implements `Copy`.
///
/// # Errors
///
/// [`CompilerError`] if `root` cannot be loaded as a Cargo project, or if loading it
/// could not discover a real sysroot -- without one, [`ra_ap_hir::Type::is_copy`] cannot
/// resolve the `Copy` lang item for anything, which would make every answer this reader
/// gives a silent, unearned "no".
pub fn Discover_Crate(root: &Path) -> Result<Vec<ClonedCopyType>, CompilerError>
{
    let (db, files) = Load_Crate(root)?;
    let sema: Semantics<'_, RootDatabase> = Semantics::new(&db);

    let mut locations: Vec<(String, u32, u32)> = Vec::new();
    for (editioned, path_str) in files
    {
        let source_file = sema.parse(editioned);
        let line_index = LineIndex::new(&source_file.syntax().text().to_string());

        for call in source_file.syntax().descendants().filter_map(ast::MethodCallExpr::cast)
        {
            if Clone_On_Copy(&sema, &call)
            {
                let start = call.syntax().text_range().start();
                let position = line_index.line_col(start);
                locations.push((path_str.clone(), position.line, position.col));
            }
        }
    }

    locations.sort();

    return Ok(locations
        .into_iter()
        .map(|(path, line, col)| return ClonedCopyType { location: format!("{path}:{}:{}", line.saturating_add(1), col.saturating_add(1)) })
        .collect());
}

/// Whether `call` is a `.clone()` call whose own resolved return type -- `Self`, per
/// `Clone::clone(&self) -> Self` -- already implements `Copy`.
fn Clone_On_Copy(sema: &Semantics<'_, RootDatabase>, call: &ast::MethodCallExpr) -> bool
{
    let Some(name_ref) = call.name_ref()
    else
    {
        return false;
    };
    if name_ref.text() != "clone"
    {
        return false;
    }

    let Some(call_expr) = ast::Expr::cast(call.syntax().clone())
    else
    {
        return false;
    };
    let Some(type_info) = sema.type_of_expr(&call_expr)
    else
    {
        return false;
    };

    return type_info.original.is_copy(sema.db);
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Fixture_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.join("fixtures").join("clone_on_copy_sample");
    }

    /// The one real, end-to-end assertion this reader owes: given a crate with one
    /// `.clone()` call on a `Copy` type and one on a `Clone`-but-not-`Copy` type, a real
    /// `ra_ap_hir` analysis finds exactly the first -- not zero (the sysroot-not-loaded
    /// failure this reader's own module doc names), not both (the receiver-type failure
    /// the same doc names), exactly one.
    #[test]
    fn Test_Discover_Crate_Should_Find_Exactly_The_Real_Clone_On_Copy_Call()
    {
        let findings = Discover_Crate(&Fixture_Root()).expect("this crate's own fixture is a real, loadable Cargo project");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert!(found.location.ends_with("lib.rs:23:12"), "{}", found.location);
    }

    #[test]
    fn Test_Discover_Crate_Should_Refuse_A_Root_With_No_Cargo_Project()
    {
        // A source directory nested under a real crate (this crate's own `src/`,
        // say) would not do: `ProjectManifest::discover_single` searches upward
        // through ancestors for a `Cargo.toml`, so it would find this crate's own and
        // report success for the wrong reason. A directory under the platform's own
        // temporary directory has no ancestor Cargo project to discover instead.
        let root = std::env::temp_dir().join("nomos-lang-rust-compiler-no-cargo-project-test");
        std::fs::create_dir_all(&root).expect("the platform temporary directory is writable");

        let error = Discover_Crate(&root).expect_err("a directory with no Cargo.toml anywhere above it is not a loadable project");

        assert!(error.reason.contains("could not be loaded"), "{}", error.reason);
    }
}
