//! What `nomos spec render` was asked for.

use std::path::PathBuf;
/// Which projection is being built, and where it lands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderRequest
{
    /// A shipped profile identifier.
    pub profile: String,
    /// The build root the profile's own relative output is placed under.
    pub into: PathBuf,
    /// The node a subject-addressed profile is pointed at.
    ///
    /// Absent for the whole-store profiles, which have nowhere to put it. Which kind a
    /// profile is is decided by the profile, so this is not a mode the caller selects.
    pub subject: Option<String>,
}
