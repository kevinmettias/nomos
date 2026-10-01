//! What a first run finds of one repository policy file, before any reader opens it.

use std::path::Path;

/// Present, absent, or present and not readable as text.
///
/// Three answers rather than two, because every reader in this workspace gives the middle
/// one a meaning of its own: an absent `standards.json`, `nomos-gate.json`,
/// `nomos-architecture.json` or `nomos-test-material.json` reads as a repository that
/// declares nothing for that concern, and the run proceeds on defaults, while a present one
/// that cannot be read is a refusal the reader owes its author. A profile that folded the
/// third into either of the others would tell a person adopting nomos that a run will
/// proceed when it will refuse, or the reverse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyFilePresence
{
    /// The file is there and its bytes read as text. Whether that text parses as the
    /// reader expects is the reader's own answer: this crate cannot depend on every
    /// reader's shape, and a profile is what a first run knows before any reader is run.
    Present,
    /// Nothing is at the path. Every reader treats this as a repository that declares
    /// nothing for that file's concern.
    Absent,
    /// Something is at the path and it could not be read as text -- a directory of that
    /// name, a permission refusal, bytes that are not UTF-8, or a path whose existence the
    /// operating system would not even confirm. Reported here rather than as absent because
    /// absent means a run proceeds, and that is exactly what is not known.
    PresentButUnreadable
    {
        /// The operating system's own account of the failure, as text.
        reason: String,
    },
}

impl PolicyFilePresence
{
    /// What is at `path`.
    #[must_use]
    pub fn Of_Path(path: &Path) -> Self
    {
        match path.try_exists()
        {
            Ok(false) => return Self::Absent,
            Ok(true) => {}
            Err(error) => return Self::PresentButUnreadable { reason: error.to_string() },
        }

        return match std::fs::read_to_string(path)
        {
            Ok(_text) => Self::Present,
            Err(error) => Self::PresentButUnreadable { reason: error.to_string() },
        };
    }
}
