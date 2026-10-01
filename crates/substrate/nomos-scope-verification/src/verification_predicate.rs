//! The command that decides whether something is done.

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

/// Ten minutes: long enough for a workspace `cargo test`, short enough that a predicate which
/// hangs gives the lease back rather than holding an item until it lapses.
const DEFAULT_TIMEOUT_SECONDS: u64 = 600;

impl VerificationPredicate
{
    /// A predicate running the given argument vector.
    #[must_use]
    pub fn From_String_Arguments(argv: Vec<String>) -> Self
    {
        return Self {
            argv,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_String_Arguments_Should_Apply_The_Default_Timeout()
    {
        let predicate = VerificationPredicate::From_String_Arguments(vec!["cargo".to_owned(), "test".to_owned()]);

        assert_eq!(predicate.argv, vec!["cargo".to_owned(), "test".to_owned()]);
        assert_eq!(predicate.timeout_seconds, DEFAULT_TIMEOUT_SECONDS);
    }

    #[test]
    fn Test_Is_Runnable_Should_Refuse_An_Empty_Or_Blank_Program()
    {
        assert!(VerificationPredicate::From_String_Arguments(vec!["cargo".to_owned()]).Is_Runnable());
        assert!(!VerificationPredicate::From_String_Arguments(vec![]).Is_Runnable());
        assert!(!VerificationPredicate::From_String_Arguments(vec![String::new()]).Is_Runnable());
    }
}
