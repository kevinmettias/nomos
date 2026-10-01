//! What `nomos agent` tells the shell.

/// What the process exits with.
///
/// Shares its numbers with every other group on this binary — see `check::ExitCode`'s own
/// doc for the fuller statement of that discipline. `5` already carries "could not be read
/// / run / answered at all" for `check`'s `Unreadable` and `spec`'s `StoreError`; a process
/// that could not be started, exited non-zero, timed out, or answered with something other
/// than the JSON it promised is the identical shape one seam over, not a fresh meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The executor ran and answered.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// The executor could not be started, exited non-zero, timed out or stalled, or
    /// answered with something other than the JSON `--output-format json` promises.
    Unavailable = 5,
    /// `judge-role` named a crate `README.md`'s band table does not list, or one with no
    /// committed surface snapshot. `6` already carries "the answer is empty because
    /// something expected was not there" for `spec`'s `Absent`.
    NotFound = 6,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}
