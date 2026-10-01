//! The two trees a run reads from and writes into.

use std::path::{Path, PathBuf};

/// Where an intent's source is read from, and where its target is written.
///
/// Two roots rather than one, because an intent's source and its target are relative to
/// different trees in the general case: the content comes from whatever authored or rendered
/// it and the target lands in the repository being integrated. That they coincide for a
/// repository materializing its own assets is a property of that caller, not of this
/// mechanism.
///
/// Both are the caller's. This crate never discovers a root, because discovering one means
/// knowing what kind of tree it is looking for, and that is knowledge a generic mechanism
/// does not have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationRoots
{
    /// The tree every intent's `source` is resolved against.
    pub source_root: PathBuf,
    /// The tree every intent's `target` is resolved against, and the only tree written.
    pub target_root: PathBuf,
}

impl MaterializationRoots
{
    /// The two roots a run works between.
    #[must_use]
    pub fn New(source_root: impl Into<PathBuf>, target_root: impl Into<PathBuf>) -> Self
    {
        return Self { source_root: source_root.into(), target_root: target_root.into() };
    }

    /// One intent's source, resolved.
    #[must_use]
    pub fn Source_Path(&self, source: &str) -> PathBuf
    {
        return self.source_root.join(Path::new(source));
    }

    /// One intent's target, resolved.
    #[must_use]
    pub fn Target_Path(&self, target: &str) -> PathBuf
    {
        return self.target_root.join(Path::new(target));
    }
}
