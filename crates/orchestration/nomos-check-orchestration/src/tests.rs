//! What this crate promises, exercised directly against [`crate::Run`] and, for the one
//! guarantee `Run`'s single entry point cannot isolate on its own, against its private
//! pieces.
//!
//! The composition here is the real one -- the real syntax provider, the real rule -- over
//! sources a test wrote by hand, which is what makes every assertion a statement about the
//! shipped seam rather than about a fixture. `nomos-cli`'s own `check` suite exercises the
//! same paths again through the compiled binary; this file is the crate's own guarantee,
//! independent of that caller ever existing.
//!
//! Split by subject, one child module each: `composition` covers the direct `Run` seam and
//! the run table a caller selects through, `materialization` covers the subprocess-backed
//! capabilities over this repository's own root, `reuse` covers the workspace and store
//! carried across calls, and `overrides` covers the repository-declared policies a real
//! `standards.json` reaches. The fixtures every child shares live here.

mod composition;
mod materialization;
mod overrides;
mod reuse;

use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

/// The text half of a [`Source_File`]. A distinct type from the path half, so the two adjacent
/// string positions cannot be transposed at a call site and still compile.
struct SourceText<'a>(&'a str);

fn Source_File(path: &str, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text.0);
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own real root, for `root` -- `Run`'s dependency step runs `cargo
/// metadata` against it regardless of what `sources` a test hands in, since the dependency
/// capability is a workspace-wide fact and not a fact about any one of `sources`'s files.
/// Real on purpose, the same choice this crate's own doc comment already states for the
/// syntax half: "the real composition... which is what makes every assertion a statement
/// about the shipped seam rather than about a fixture."
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// A fresh, empty directory under the OS temp root, unique per test name and process --
/// `nomos-cli::work`'s own `Scratch_Directory` fixture shape, needed here for the same reason:
/// `StdFileSystem` reads real bytes from a real path, so proving a real override changes real
/// behavior needs a real file on disk rather than a fixture handed in by value.
fn Scratch_Directory(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-check-orchestration-test-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");
    return root;
}
