//! What `nomos check` was asked for.

use std::path::PathBuf;
/// What to check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckCommand
{
    /// The tree to judge.
    pub root: PathBuf,
}
