//! Every way a file operation fails, kept apart from what it was trying to do.
//!
//! # Why this file sits at the crate root rather than inside `file_system/`
//!
//! It was `file_system/error.rs`, declaring `Error`, and `file_system.rs` published it as
//! `pub use error::Error as FileSystemError;`. Three rules pull against that shape, and
//! moving the file here is the only arrangement that satisfies all three at once:
//!
//! - `file-name-matches-declared-type` requires a file to be named after the public type it
//!   declares, so a type called `FileSystemError` forces the file to be `file_system_error`.
//! - `check-tree-legibility` forbids a file repeating its parent folder, so
//!   `file_system/file_system_error.rs` is out.
//! - `check-facade-surface` counts `pub use x::Y as Z;` a second public name for one item, so
//!   keeping `Error` and qualifying it at the facade is out too.
//!
//! At the crate root the name repeats no parent, the file matches the type, and no alias is
//! needed. The public path is unchanged -- `lib.rs` re-exports it, so `nomos_platform::FileSystemError`
//! is still what every caller names.

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

impl std::error::Error for FileSystemError
{}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Path_Should_Read_Back_Whichever_Variant_Carries_It()
    {
        assert_eq!(FileSystemError::NotFound { path: "a/missing".to_owned() }.Path(), "a/missing");
        assert_eq!(
            FileSystemError::Denied { path: "b/locked".to_owned(), cause: "permission denied".to_owned() }.Path(),
            "b/locked"
        );
        assert_eq!(
            FileSystemError::Other { path: "c/odd".to_owned(), cause: "disk full".to_owned() }.Path(),
            "c/odd"
        );
    }
}
