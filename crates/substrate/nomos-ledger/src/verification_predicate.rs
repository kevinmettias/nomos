//! The command that decides whether an item is done.

use serde::Deserialize;
use serde::Serialize;
/// A command that decides whether an item is actually finished.
///
/// An argument vector, never a command string. The prototype stored a string and split
/// it on whitespace with quote handling, which put a bespoke parser between what an
/// author wrote and what ran — in a file several agents write concurrently.
///
/// It is also a *predicate*, not a description. `done_when` on the item is prose for a
/// human; this is the thing that gets run, and an item cannot report itself finished
/// because somebody typed that it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationPredicate
{
    /// The program and its arguments.
    pub argv: Vec<String>,
    /// How long to allow before giving up.
    pub timeout_seconds: u64,
}

impl VerificationPredicate
{
    /// A predicate running the given argument vector.
    #[must_use]
    pub fn New(argv: Vec<String>) -> Self
    {
        return Self {
            argv,
            timeout_seconds: 600,
        };
    }

    /// Whether this predicate could actually be run.
    ///
    /// An empty argument vector is not a predicate; it is a field somebody filled in
    /// to satisfy a schema.
    #[must_use]
    pub fn Is_Runnable(&self) -> bool
    {
        return self.argv.first().is_some_and(|program| !program.is_empty());
    }
}
