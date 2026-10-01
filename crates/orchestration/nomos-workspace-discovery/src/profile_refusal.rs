//! Why a root could not be profiled at all.

use std::path::PathBuf;

/// A root no profile can be computed for, as distinct from a root that profiles as empty.
///
/// [`crate::Walked_Sources`] answers `None` for a root that is not a directory and leaves
/// the caller's own outcome type to make that a typed answer; this is that type for a
/// profile. A directory that is walked and turns out to hold nothing is a real profile --
/// every count zero, every manifest and policy file absent, no root marker -- and a person
/// adopting nomos from an empty directory should read exactly that. A path that is not a
/// directory is not an empty repository, and reporting it as one would tell them a run
/// found nothing when a run could not have looked.
///
/// One variant, because one refusal exists. It is an enum rather than a struct so that a
/// caller matches on the closed set of reasons a profile can refuse for, and a second
/// reason, if one is ever measured, arrives as a variant every caller is made to consider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileRefusal
{
    /// `root` is not a directory: nothing to walk, and no root for a manifest or a policy
    /// file to sit at.
    RootIsNotADirectory
    {
        /// The path as the caller gave it.
        root: PathBuf,
    },
}

impl core::fmt::Display for ProfileRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::RootIsNotADirectory { root } =>
            {
                write!(formatter, "{} is not a directory, so there is nothing to profile", root.display())
            }
        };
    }
}
