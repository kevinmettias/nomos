//! Which of the four refusals it was.

/// Which of the four refusals it was.
#[derive(Debug)]
pub enum ArchiveErrorKind
{
    Unreadable
    {
        cause: String,
    },
    /// An archive holding no files at all.
    ///
    /// Refused rather than returned, because at every later call site an archive that
    /// lists nothing is indistinguishable from one that was never read. This is §C8.8 —
    /// a missing path is not an empty repository — applied to the archives.
    Empty,
    NoSuchEntry
    {
        entry: String,
    },
    NotText
    {
        entry: String,
        cause: String,
    },
}
