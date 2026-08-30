//! Which of the two places a [`super::CommitRefusal`] originated: the filesystem, or the
//! store's own edit staging.

use nomos_platform::FileSystemError;
use nomos_spec_store::EditError;

/// Which of the two places a [`super::CommitRefusal`] originated: the filesystem, or the
/// store's own edit staging.
#[derive(Debug)]
pub enum Error
{
    FileSystem(FileSystemError),
    Edit(EditError),
}
