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

#[cfg(test)]
mod tests
{
    use super::*;

    /// `main()` feeds this straight into `u8::try_from`, so a code that silently drifted
    /// from the discriminant it was declared with would arrive as the wrong process exit
    /// rather than as a compile error.
    #[test]
    fn Test_Value_Should_Return_The_Declared_Numeric_Discriminant()
    {
        assert_eq!(ExitCode::Ok.Value(), 0);
        assert_eq!(ExitCode::QueryFailed.Value(), 1);
        assert_eq!(ExitCode::Usage.Value(), 2);
    }
}
