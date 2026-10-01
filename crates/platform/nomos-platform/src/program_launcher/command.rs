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
    /// How long to allow before giving up, regardless of whether the process is still
    /// producing output.
    pub timeout: std::time::Duration,
    /// How long the process may go without producing any new output before it is judged
    /// to have stalled, rather than to be doing legitimately slow work.
    ///
    /// Defaults to [`Command::timeout`] by [`Command::From_String_Arguments`], so a caller that never asks
    /// for the distinction gets exactly the wait it asked for before: the two bounds
    /// coincide and the process is judged only once, at the wall bound. Setting this
    /// shorter than `timeout` is what makes the idle bound able to fire first.
    pub idle_timeout: std::time::Duration,
    /// A line the process prints, on either stream, to say it is waiting on something outside
    /// itself rather than working or hung -- cargo's `Blocking waiting for file lock` while
    /// another cargo holds the same build directory.
    ///
    /// While the process's latest output is a line containing this text, its silence is a
    /// wait and does not count toward [`Command::idle_timeout`] -- for at most
    /// [`Command::waiting_timeout`] at a stretch, after which it counts again. `None` from
    /// [`Command::From_String_Arguments`], so no caller's idle bound changes unless it names a
    /// line with [`Command::While_Waiting_Prints`].
    pub waiting_line: Option<String>,
    /// How long one unbroken declared wait may last before its silence counts toward
    /// [`Command::idle_timeout`] again.
    ///
    /// A wait is legitimate only while whatever it waits on can finish, and nothing a launcher can
    /// see tells a lock its holder will release from one it never will: measured on 2026-09-27, a
    /// nested `cargo clippy` printed its lock-wait line and then waited ninety minutes on a holder
    /// that never let go, holding a machine-wide lock of its own the whole time. With only the wall
    /// bound behind it, such a wait lasts as long as the wall bound does. This bound is the wait's
    /// own, set by the caller that knows how long the thing it waits on legitimately takes; once it
    /// runs out, the silence is judged at the idle bound exactly as undeclared silence is.
    pub waiting_timeout: std::time::Duration,
}

impl Command
{
    /// Constructs a command from an argument vector.
    ///
    /// The idle bound starts equal to the wall bound. See
    /// [`Command::With_Idle_Timeout`] to give it one of its own.
    #[must_use]
    pub fn From_String_Arguments(argv: Vec<String>, timeout: std::time::Duration) -> Self
    {
        return Self {
            argv,
            working_directory: None,
            timeout,
            idle_timeout: timeout,
            waiting_line: None,
            waiting_timeout: std::time::Duration::ZERO,
        };
    }

    /// The program to run, if the vector is not empty.
    #[must_use]
    pub fn Program(&self) -> Option<&str>
    {
        return self.argv.first().map(String::as_str);
    }

    /// Gives the command an idle bound distinct from its wall bound.
    ///
    /// A process that keeps producing output resets this bound and is judged only
    /// against the wall bound; a process that goes silent for this long is judged to
    /// have stalled even though the wall bound has not yet expired.
    #[must_use]
    pub fn With_Idle_Timeout(mut self, idle_timeout: std::time::Duration) -> Self
    {
        self.idle_timeout = idle_timeout;

        return self;
    }

    /// Declares `line` as what the process prints while it waits on something outside
    /// itself, and `at_most` as how long one such wait may last before it is judged as silence
    /// again. See [`Command::waiting_line`] and [`Command::waiting_timeout`]: there is no way to
    /// declare a wait without bounding it.
    #[must_use]
    pub fn While_Waiting_Prints(mut self, line: &str, at_most: std::time::Duration) -> Self
    {
        self.waiting_line = Some(line.to_owned());
        self.waiting_timeout = at_most;

        return self;
    }
}

#[cfg(test)]
mod tests
{
    use std::time::Duration;

    use super::*;

    /// A wall bound for the case that asserts only that the two bounds coincide.
    const A_WALL_BOUND_SECONDS: u64 = 10;

    /// The wall bound of the case that sets the idle bound apart from it.
    const A_LONG_WALL_BOUND_SECONDS: u64 = 30;

    /// The idle bound of that case. Shorter than the wall bound, so an assertion
    /// that read the wrong field would fail rather than pass.
    const A_SHORT_IDLE_BOUND_SECONDS: u64 = 5;

    #[test]
    fn Test_An_Empty_Argv_Should_Have_No_Program()
    {
        let empty = Command::From_String_Arguments(Vec::new(), Duration::from_secs(1));

        assert_eq!(empty.Program(), None);
    }

    #[test]
    fn Test_From_String_Arguments_Should_Start_The_Idle_Bound_Equal_To_The_Wall_Bound()
    {
        let command = Command::From_String_Arguments(vec!["prog".to_owned()], Duration::from_secs(A_WALL_BOUND_SECONDS));

        assert_eq!(
            command.idle_timeout, command.timeout,
            "a caller that never asks for the distinction must get the same bound twice"
        );
    }

    #[test]
    fn Test_With_Idle_Timeout_Should_Set_The_Idle_Bound_Independently_Of_The_Wall_Bound()
    {
        let command = Command::From_String_Arguments(vec!["prog".to_owned()], Duration::from_secs(A_LONG_WALL_BOUND_SECONDS))
            .With_Idle_Timeout(Duration::from_secs(A_SHORT_IDLE_BOUND_SECONDS));

        assert_eq!(command.timeout, Duration::from_secs(A_LONG_WALL_BOUND_SECONDS));
        assert_eq!(command.idle_timeout, Duration::from_secs(A_SHORT_IDLE_BOUND_SECONDS));
    }
}
