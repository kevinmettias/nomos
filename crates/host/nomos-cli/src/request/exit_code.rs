//! What `nomos request` tells the shell.

/// What the process exits with.
///
/// Shares its numbers with every other group on this binary, so an agent that runs more than
/// one does not have to know which it ran before reading the number — see `spec::ExitCode` for
/// the fuller statement of that discipline. `9` is `spec`'s own "an edit was refused"; a
/// submission refused is the same shape one level up the same seam, and is deliberately not a
/// fresh number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The submission was accepted.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// The store could not be built or read at all.
    StoreError = 5,
    /// The submission was accepted and the projection asked for could not be written.
    Unwritable = 7,
    /// The submission failed the rule set. Nothing was stored.
    Refused = 9,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}
