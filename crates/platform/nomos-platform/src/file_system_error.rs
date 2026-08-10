//! Every way a file operation fails, kept apart from what it was trying to do.

/// Why a filesystem operation failed.
#[derive(Debug)]
pub enum FileSystemError
{
    /// The path does not exist.
    NotFound
    {
        /// The path that was not found.
        path: String,
    },
    /// The path exists and could not be read or written.
    Denied
    {
        /// The path involved.
        path: String,
        /// What the operating system said.
        cause: String,
    },
    /// Something else went wrong.
    Other
    {
        /// The path involved.
        path: String,
        /// What went wrong.
        cause: String,
    },
}

impl FileSystemError
{
    /// The path the failure concerns.
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return match self
        {
            Self::NotFound { path } | Self::Denied { path, .. } | Self::Other { path, .. } => path,
        };
    }
}

impl core::fmt::Display for FileSystemError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NotFound { path } => write!(formatter, "{path} does not exist"),
            Self::Denied { path, cause } => write!(formatter, "{path} could not be accessed: {cause}"),
            Self::Other { path, cause } => write!(formatter, "{path}: {cause}"),
        };
    }
}

impl std::error::Error for FileSystemError {}
