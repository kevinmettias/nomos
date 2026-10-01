//! Why `nomos spec render` did not place a projection.

use nomos_platform::FileSystemError;
use nomos_spec_project::ProjectError;
use nomos_spec_store::StoreError;
use std::path::PathBuf;

/// Why `nomos spec render` did not place a projection.
///
/// Moved from `nomos-cli::spec::verb::render`'s own `Declared` and `Report_Build_Error`:
/// which of these happened is a fact about the catalogue, the store or the destination, not
/// about how a terminal reports it.
#[derive(Debug)]
pub enum RenderRefusal
{
    /// The catalogue does not carry a profile by this name.
    NoSuchProfile
    {
        requested: String,
        known: Vec<String>,
    },
    /// Resolving the subject or building the projection failed.
    ///
    /// Carries [`ProjectError::Empty`] the same way a store failure carries any other
    /// variant: distinguishing "this section is empty because the corpus is not whole" from
    /// "this section is empty and the corpus is whole" needs [`crate::corpus::Assembly`],
    /// which only a renderer holds, so that distinction stays a rendering decision.
    Project(ProjectError),
    /// The built projection could not be written where it was asked to go.
    Unwritable
    {
        path: PathBuf,
        error: FileSystemError,
    },
    /// The store could not be assembled at all.
    Store(StoreError),
}
