//! What this process exits with.
//!
//! Deliberately not `nomos-cli`'s exit-code table — this is a separate binary nothing
//! else invokes, run by a human on demand. **A finding existing is never one of these
//! codes.** `OD-STORE-002` reserves the judgment "record-worthy or not" for the person
//! reading the report; this exit code answers only "did the report finish running".

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The report ran to completion — whether or not it found anything.
    Ok = 0,
    /// A `git` call could not be run, or ran and refused (a bad revision, most
    /// commonly).
    QueryFailed = 1,
    /// The command line, or the repository root it named, was wrong.
    Usage = 2,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub(crate) const fn Value(self) -> i32
    {
        return self as i32;
    }
}
