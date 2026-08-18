//! Resolving `nomos spec preview`'s request against an assembled store: reading the staged
//! text and asking the store what committing it would change.
//!
//! # Why this is generic over [`FileSystem`]
//!
//! `--from` names a single file at a path an author chose, not one this crate already knows
//! the way [`crate::run::render`]'s output path is known from a profile's own declaration --
//! the "different shape" this crate's own top-level documentation flagged as increment 3's
//! open question. It is still exactly [`FileSystem::Read_To_String`]'s shape regardless of
//! who chose the path: one file, read whole, and the port does not care which caller named
//! it. Going through it keeps this crate's whole I/O boundary uniform -- a test can hand
//! `Preview` a fake `FileSystem` and exercise `--from` missing without touching a real disk,
//! the same benefit `Render`'s own tests already get for the write half.

use nomos_platform::FileSystem;
use nomos_spec_store::EditPreview;
use std::path::Path;

use crate::corpus::Assembly;
use crate::outcome::PreviewRefusal;
use crate::request::EditRequest;

/// What committing `request` would change, without writing anything.
///
/// Moved from `nomos-cli::spec::verb::editing`'s own `Preview`, `Staged_Text` and
/// `Previewed`, minus the writing: only the preview itself is built here.
///
/// # Errors
///
/// Returns [`PreviewRefusal::Unreadable`] when `request.from` could not be read, and
/// [`PreviewRefusal::Edit`] when the store refuses the staged edit.
pub fn Preview<F: FileSystem>(
    assembly: &Assembly,
    request: &EditRequest,
    filesystem: &F,
) -> Result<EditPreview, PreviewRefusal>
{
    let staged = Staged_Text(&request.from, filesystem)?;

    return Previewed(assembly, request, &staged);
}

/// The bytes an author staged at `from`, read through `filesystem`.
fn Staged_Text<F: FileSystem>(from: &Path, filesystem: &F) -> Result<String, PreviewRefusal>
{
    return filesystem
        .Read_To_String(from)
        .map_err(|error| return PreviewRefusal::Unreadable { path: from.to_owned(), error });
}

/// The staged text, checked against the store and turned into a preview.
fn Previewed(assembly: &Assembly, request: &EditRequest, staged: &str) -> Result<EditPreview, PreviewRefusal>
{
    let rename = request.rename.as_deref();

    return assembly
        .store
        .Claim_For_Edit(&request.id, None)
        .and_then(|claimed| return claimed.Stage(staged, rename))
        .and_then(|edit| return edit.Preview(&assembly.store))
        .map_err(PreviewRefusal::Edit);
}
