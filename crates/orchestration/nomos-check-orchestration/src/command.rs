//! What `nomos check` was asked for, independent of how it was spelled.
//!
//! This is the request vocabulary a caller above the walk hands to a composition root. An
//! argument parser is one way to produce one; it is not the only way this crate expects one
//! to arrive, which is the point of stating the vocabulary here rather than leaving it a
//! shape only `nomos-cli`'s own parser produces. Moved verbatim from
//! `nomos-cli::check::command` -- there is exactly one verb today, so this is a struct
//! rather than an enum, unlike `nomos_work_orchestration::WorkCommand`.

use std::path::PathBuf;

/// What to check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckCommand
{
    /// The tree to judge.
    pub root: PathBuf,
}
