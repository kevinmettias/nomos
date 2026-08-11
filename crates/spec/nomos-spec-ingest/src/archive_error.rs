//! Every way an archive refuses to be opened or read.

use std::path::PathBuf;

/// Why an archive could not be read, always naming which one.
///
/// A caller iterating twenty archives has to be able to say which one failed without
/// correlating by position, and an error that only says "invalid zip" turns a corrupt file
/// into a mystery.
///
/// The path is the type's and not the kind's. Every refusal here is a refusal *of one
/// archive*, so a caller reads which one without matching on a reason it does not
/// otherwise care about, and a new reason cannot forget to carry the path.
#[derive(Debug)]
pub struct ArchiveError
{
    pub archive: PathBuf,
    pub kind: ArchiveErrorKind,
}

impl core::fmt::Display for ArchiveError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let archive = self.archive.display();

        return match self.kind
        {
            ArchiveErrorKind::Unreadable { ref cause } =>
            {
                write!(formatter, "{archive}: cannot be read as an archive: {cause}")
            }
            ArchiveErrorKind::Empty => write!(
                formatter,
                "{archive}: holds no files. Refused rather than reported as empty, because an \
                 archive that lists nothing reads exactly like one that was never opened"
            ),
            ArchiveErrorKind::NoSuchEntry { ref entry } =>
            {
                write!(formatter, "{archive}: has no entry {entry}")
            }
            ArchiveErrorKind::NotText {
                ref entry,
                ref cause,
            } => write!(formatter, "{archive}: {entry} is not text: {cause}"),
        };
    }
}

impl std::error::Error for ArchiveError
{}

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
