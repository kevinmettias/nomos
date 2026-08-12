//! What is to be run, before anything has tried to run it.

use std::path::PathBuf;

/// A program to run, as an argument vector.
///
/// An argument **vector**, never a command string. The prototype stored verification
/// commands as strings and split them on whitespace with quote handling, which meant a
/// bespoke parser standing between what an author wrote and what ran — and a shared
/// file that several agents write became a place where one agent's quoting could change
/// what another agent's process executed.
///
/// Storing `argv` removes the parser and the whole class of defect with it. It also
/// removes shell interpolation, globbing and chaining, which a verification predicate
/// has no business doing: a predicate answers yes or no, and one that can also pipe,
/// redirect and expand is a small script nobody reviewed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command
{
    /// The program and its arguments. The first element is the program.
    pub argv: Vec<String>,
    /// Where to run it.
    pub working_directory: Option<PathBuf>,
    /// How long to allow before giving up.
    pub timeout: std::time::Duration,
}

impl Command
{
    /// Constructs a command from an argument vector.
    #[must_use]
    pub fn New(argv: Vec<String>, timeout: std::time::Duration) -> Self
    {
        return Self {
            argv,
            working_directory: None,
            timeout,
        };
    }

    /// The program to run, if the vector is not empty.
    #[must_use]
    pub fn Program(&self) -> Option<&str>
    {
        return self.argv.first().map(String::as_str);
    }
}

#[cfg(test)]
mod tests
{
    use std::time::Duration;

    use super::*;

    #[test]
    fn Test_An_Empty_Argv_Should_Have_No_Program()
    {
        let empty = Command::New(Vec::new(), Duration::from_secs(1));

        assert_eq!(empty.Program(), None);
    }
}
