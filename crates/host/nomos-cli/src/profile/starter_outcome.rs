//! What came of being asked to write a starting gate policy file.

use std::path::PathBuf;

/// Written, refused because something is already there, or not writable at all.
///
/// The middle one is a refusal rather than an overwrite, and that is the decision this type
/// exists to make visible. A repository that already declares a gate policy has said
/// something, possibly at length and possibly some time ago; replacing it with a file that
/// declares nothing would silently retire every suppression, baseline and phase it carried,
/// and the run after that would block on findings the repository had already dispositioned.
/// What to do about an existing declaration is the author's call, so the verb stops and
/// names the path instead of making it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StarterOutcome
{
    /// The file was not there and now is.
    Written
    {
        /// Where it was written.
        path: PathBuf,
    },
    /// Something is already at that path, so nothing was written.
    AlreadyDeclared
    {
        /// The path that was left alone.
        path: PathBuf,
    },
    /// Nothing was there and it could not be written.
    Unwritable
    {
        /// The path that could not be written.
        path: PathBuf,
        /// The filesystem's own account of the failure.
        reason: String,
    },
}
