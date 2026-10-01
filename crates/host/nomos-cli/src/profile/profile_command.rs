//! What `nomos profile` was asked for.

use std::path::PathBuf;

/// The root to profile, and whether a starter gate policy file was asked for.
///
/// Two fields rather than two verbs, because the write is not a separate question: a person
/// who asks for a starter file wants to see what the root already holds in the same breath,
/// and a verb that wrote one without first printing which policy files are already there
/// would be answering the second half of the question while hiding the first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProfileCommand
{
    /// The tree to profile.
    pub(crate) root: PathBuf,
    /// Whether to write a starting `nomos-gate.json` for a root that has none.
    pub(crate) write_gate_policy: bool,
}
