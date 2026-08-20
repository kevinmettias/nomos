//! What `nomos spec render` produced, or why it did not.

mod render_refusal;

pub use render_refusal::RenderRefusal;

use nomos_spec_project::Stamp;
use std::path::PathBuf;

/// Both halves of a built projection, placed where the run asked for them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderAnswer
{
    /// The profile identifier that was built.
    pub id: String,
    /// Where the body landed.
    pub body: PathBuf,
    /// Where its sidecar landed.
    pub sidecar: PathBuf,
    /// What the build selected, and what it hashes to.
    pub stamp: Stamp,
}
