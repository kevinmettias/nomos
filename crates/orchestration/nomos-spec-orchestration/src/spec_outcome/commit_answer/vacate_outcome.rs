//! What became of the path a rename vacated.

/// What became of the path a rename vacated.
///
/// Read through [`nomos_platform::FileSystem::Remove_File`], the port's fourth operation.
/// See `crate::run::commit`'s own documentation for why removal is a defaulted method there
/// rather than a required one.
#[derive(Debug)]
pub enum VacateOutcome
{
    /// The old path was removed.
    Removed,
    /// The old path was already gone.
    AlreadyGone,
    /// The old path could not be removed, so two files now declare this record.
    Failed(String),
}
