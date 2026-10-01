//! Putting the helper where `go` can run it.
//!
//! Its two files are compiled into this crate and written under the host's temporary directory, in
//! a directory named for their digest, so two builds of this crate carrying different helpers never
//! share one and the same helper is written once however many runs ask for it. Nothing is built
//! here: `go run` builds it, and `go`'s own build cache is keyed by the toolchain, so a host that
//! changes its Go never runs a helper built by the old one -- whose export-data reader would not
//! read what the new one writes.

use crate::TypesFailure;
use crate::typecheck::TypesError;
use nomos_platform::{Environment, FileSystem};
use std::path::{Path, PathBuf};

/// The helper's source.
const MAIN: &str = include_str!("../helper/main.go");
/// The helper's module file: a module of its own, needing nothing beyond the standard library.
const MODULE: &str = include_str!("../helper/go.mod");

/// The variables a host names its temporary directory in, in the order they are asked.
const TEMPORARY_VARIABLES: [&str; 3] = ["TMPDIR", "TEMP", "TMP"];

/// The directory holding the helper's two files, written there if they are not already.
pub(crate) fn Install_Helper<Fs: FileSystem, Env: Environment>(filesystem: &Fs, environment: &Env) -> Result<PathBuf, TypesError>
{
    let temporary = Temporary_Directory(environment).ok_or_else(|| {
        return TypesError::New(TypesFailure::HelperUnwritable, "the environment names no temporary directory (TMPDIR, TEMP or TMP) to write the helper in");
    })?;
    let directory = temporary.join(format!("nomos-go-types-{}", nomos_model::Digest_Of_Parts(&[MAIN.as_bytes(), MODULE.as_bytes()])));
    for (name, contents) in [("main.go", MAIN), ("go.mod", MODULE)]
    {
        Written(filesystem, &directory.join(name), contents)?;
    }

    return Ok(directory);
}

/// `path` holding exactly `contents`, written only when it does not already.
fn Written<Fs: FileSystem>(filesystem: &Fs, path: &Path, contents: &str) -> Result<(), TypesError>
{
    let holds = || return filesystem.Read_To_String(path).is_ok_and(|held| return held == contents);
    if holds()
    {
        return Ok(());
    }
    let written = filesystem.Replace_Atomically(path, contents);
    // Another run writing the same helper at the same moment can refuse this write and still
    // leave exactly these bytes behind, which is all this needs.
    if written.is_err() && holds()
    {
        return Ok(());
    }

    return written.map_err(|error| return TypesError::New(TypesFailure::HelperUnwritable, &format!("the helper could not be written to {}: {error}", path.display())));
}

/// The temporary directory the environment names, or `/tmp` where the platform has one to fall
/// back on.
fn Temporary_Directory<Env: Environment>(environment: &Env) -> Option<PathBuf>
{
    let named = TEMPORARY_VARIABLES.iter().find_map(|name| return environment.Variable(name).filter(|value| return !value.is_empty()));
    if let Some(named) = named
    {
        return Some(PathBuf::from(named));
    }

    return (!cfg!(windows)).then(|| return PathBuf::from("/tmp"));
}
