//! One file, as the slice sees it.
//!
//! The record a walk produces. Kept apart from the walk itself so that a test which only
//! needs to say what a subject looks like does not have to read how a tree is descended.

use nomos_contracts::SubjectId;

/// One file, as the slice sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Identity of the file itself.
    pub subject: SubjectId,
    /// Corpus-relative path, normalized. Kept for reporting: a failure that names a
    /// digest is a failure nobody can go and look at.
    pub path: String,
    /// The directory this file sits in, normalized. The subject of the rollup that reads
    /// it, and the reason a change to one file has a descendant at all.
    pub group: String,
    /// Identity of that directory.
    pub group_subject: SubjectId,
    pub source: String,
}
