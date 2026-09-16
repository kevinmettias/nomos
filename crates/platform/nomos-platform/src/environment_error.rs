//! Every way reading the environment fails.
//!
//! # Why this file sits at the crate root rather than inside `environment/`
//!
//! It was `environment/environment_error.rs`, and its own name repeated its parent folder —
//! `check-tree-legibility` reports a redundant parent prefix. The remedy that rule states for
//! a repeated word that is a declared type's own name is to rename the *type*, not the file,
//! because `file-name-matches-declared-type` would put the repeated word straight back. A type
//! called `EnvironmentError` inside `environment/` is unsatisfiable for that reason; at the
//! crate root the name repeats no parent folder and both rules pass on the same file.
//!
//! The public path is unchanged: `lib.rs` re-exports this, so `nomos_platform::EnvironmentError`
//! is still the name every caller uses.

/// Why an environment read failed.
///
/// One variant, because the port has one fallible operation. [`Environment::Variable`] is
/// infallible by construction — an absent variable and an unreadable one are both `None`,
/// which is what every caller in this workspace already did with them before the port
/// existed — so there is nothing for it to report here.
///
/// [`Environment::Variable`]: crate::Environment::Variable
#[derive(Debug)]
pub enum EnvironmentError
{
    /// The working directory could not be read.
    ///
    /// Carries no path, unlike [`crate::FileSystemError`]: the whole failure is that
    /// there is no path to name. A process whose working directory has been removed or
    /// made unreadable underneath it cannot say which one it was.
    WorkingDirectoryUnreadable
    {
        /// What the operating system said.
        cause: String,
    },
}

impl core::fmt::Display for EnvironmentError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::WorkingDirectoryUnreadable { cause } =>
            {
                write!(formatter, "the current directory could not be read: {cause}")
            }
        };
    }
}

impl std::error::Error for EnvironmentError
{}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The rendering is load-bearing rather than cosmetic: `nomos-lang-rust-clippy`
    /// wraps this into its own `ClippyError::reason`, and that crate's message was the
    /// wording before the port existed. A change here silently rewords a provider's
    /// diagnostic.
    #[test]
    fn Test_Display_Should_Name_The_Operating_System_Cause()
    {
        let error = EnvironmentError::WorkingDirectoryUnreadable {
            cause: "permission denied".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "the current directory could not be read: permission denied"
        );
    }
}
