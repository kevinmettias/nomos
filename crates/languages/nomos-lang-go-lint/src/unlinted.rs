//! [`Unlinted`], a part of what was asked about that no fact answers.

use crate::VetFailure;

/// A part of the Go sources asked about that no fact answers, and why -- reported by whoever holds
/// the answer, and never read as a module that `go vet` found clean.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unlinted
{
    /// A module `go vet` gave no answer for.
    Module
    {
        /// The module's directory, relative to the root, empty for the root itself.
        path: String,
        /// Which kind of failure.
        failure: VetFailure,
        /// What happened, in the tool's own words where it gave any.
        reason: String,
    },
    /// Go sources that sit under no `go.mod`, so no module holds them and `go vet` has no package
    /// to load them in.
    Outside
    {
        /// Each such source, relative to the root.
        files: Vec<String>,
    },
}
