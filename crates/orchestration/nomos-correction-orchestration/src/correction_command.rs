//! What a correction run was asked to do.

use std::path::PathBuf;

/// What `phantom-mirrors` was asked to do — the one correction verb this seam answers
/// today, moved here unchanged from `nomos-cli`'s own `CorrectCommand` so both hosts
/// construct the identical command shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionCommand
{
    pub root: PathBuf,
    /// Whether to actually commit and write the corrected file, or stop after staging and
    /// validating it.
    pub commit: bool,
}
