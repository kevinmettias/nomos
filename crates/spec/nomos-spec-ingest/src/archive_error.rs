//! Every way an archive refuses to be opened or read.

use std::path::PathBuf;
/// Why an archive could not be read, always naming which one.
///
/// Every arm carries the archive path. A caller iterating twenty archives has to be able
/// to say which one failed without correlating by position, and an error that only says
/// "invalid zip" turns a corrupt file into a mystery.
#[derive(Debug)]
pub enum ArchiveError
{
    Unreadable
    {
        archive: PathBuf,
        cause: String,
    },
    /// An archive holding no files at all.
    ///
    /// Refused rather than returned, because at every later call site an archive that
    /// lists nothing is indistinguishable from one that was never read. This is §C8.8 —
    /// a missing path is not an empty repository — applied to the archives.
    Empty
    {
        archive: PathBuf,
    },
    NoSuchEntry
    {
        archive: PathBuf,
        entry: String,
    },
    NotText
    {
        archive: PathBuf,
        entry: String,
        cause: String,
    },
}

impl core::fmt::Display for ArchiveError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unreadable { archive, cause } => {
                write!(formatter, "{}: cannot be read as an archive: {cause}", archive.display())
            }
            Self::Empty { archive } => write!(
                formatter,
                "{}: holds no files. Refused rather than reported as empty, because an \
                 archive that lists nothing reads exactly like one that was never opened",
                archive.display()
            ),
            Self::NoSuchEntry { archive, entry } => {
                write!(formatter, "{}: has no entry {entry}", archive.display())
            }
            Self::NotText {
                archive,
                entry,
                cause,
            } => write!(formatter, "{}: {entry} is not text: {cause}", archive.display()),
        };
    }
}

impl std::error::Error for ArchiveError {}
