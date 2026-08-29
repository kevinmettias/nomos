//! What became of the path a rename vacated.

/// What became of the path a rename vacated.
///
/// Outside [`nomos_platform::FileSystem`] on purpose -- deletion is not one of the port's
/// three declared operations, so this is [`std::fs::remove_file`] directly. See
/// `crate::run::commit`'s own documentation for why that is the arrangement rather than a
/// defect.
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
