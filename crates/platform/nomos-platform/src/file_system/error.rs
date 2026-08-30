//! Every way a file operation fails, kept apart from what it was trying to do.

/// Why a filesystem operation failed.
#[derive(Debug)]
pub enum Error
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

impl Error
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

impl core::fmt::Display for Error
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

impl std::error::Error for Error
{}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Path_Should_Read_Back_Whichever_Variant_Carries_It()
    {
        assert_eq!(Error::NotFound { path: "a/missing".to_owned() }.Path(), "a/missing");
        assert_eq!(
            Error::Denied { path: "b/locked".to_owned(), cause: "permission denied".to_owned() }.Path(),
            "b/locked"
        );
        assert_eq!(
            Error::Other { path: "c/odd".to_owned(), cause: "disk full".to_owned() }.Path(),
            "c/odd"
        );
    }
}
