//! What `nomos spec` tells the shell.

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group, or an agent that runs both has to
/// know which command it ran before it can read the number. `3` and `4` are `work`'s
/// claim codes and are deliberately not reused here.
///
/// [`ExitCode::Absent`] is the distinction that earns its own code. An agent told the
/// corpus is absent should configure one; an agent told the identifier is unknown should
/// correct the identifier. Collapsing those into "non-zero" makes the fixable case
/// indistinguishable from the mistaken one — which is the confusion this whole group was
/// written to end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The question was answered.
    Ok = 0,
    /// The store does not hold it, and everything it would have come from was read.
    NotFound = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The store could not be built or read at all.
    StoreError = 5,
    /// The answer is empty because something the store expected was not there.
    Absent = 6,
    /// The answer was produced and could not be written where it was asked to go.
    Unwritable = 7,
    /// A governed output on disk is no longer what the store and its stamp say it is.
    ///
    /// Apart from [`ExitCode::Absent`] on purpose. Absent is "nobody could find out";
    /// this is "somebody found out, and the answer is that the file drifted". A gate
    /// collapsing the two would report a machine without a corpus exactly as it reports
    /// an edited output, which is the confusion `OD-GATE-001` is already about.
    Stale = 8,
    /// An authoring step refused the edit it was given.
    ///
    /// Apart from [`ExitCode::Usage`] because the command line was right: the author asked
    /// for exactly what they meant and the *content* was refused — a text this surface would
    /// not have written, a front matter naming a different record, a rename onto an occupied
    /// path. An agent told its arguments were wrong will retype them; an agent told its edit
    /// was refused will read the reason.
    Refused = 9,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `spec`'s exit codes as the shell sees them. `Value()` is what a caller branches on, so
    /// the numbers are named here and pinned below, rather than left as bare literals inside
    /// the assertions that read them.
    const USAGE_CODE: i32 = 2;
    const STORE_ERROR_CODE: i32 = 5;
    const ABSENT_CODE: i32 = 6;
    const UNWRITABLE_CODE: i32 = 7;
    const STALE_CODE: i32 = 8;
    const REFUSED_CODE: i32 = 9;

    /// Every variant's discriminant, as declared above. `spec::tests::
    /// Test_Value_Should_Be_Stable_And_Not_Collide_With_Works_Claim_Codes` is the cross-cutting half
    /// of this claim -- that these numbers also do not collide with `work`'s own claim codes; this
    /// test's concern is narrower and stays with the declaration it reads.
    #[test]
    fn Test_Value_Should_Return_The_Declared_Discriminant()
    {
        assert_eq!(ExitCode::Ok.Value(), 0);
        assert_eq!(ExitCode::NotFound.Value(), 1);
        assert_eq!(ExitCode::Usage.Value(), USAGE_CODE);
        assert_eq!(ExitCode::StoreError.Value(), STORE_ERROR_CODE);
        assert_eq!(ExitCode::Absent.Value(), ABSENT_CODE);
        assert_eq!(ExitCode::Unwritable.Value(), UNWRITABLE_CODE);
        assert_eq!(ExitCode::Stale.Value(), STALE_CODE);
        assert_eq!(ExitCode::Refused.Value(), REFUSED_CODE);
    }
}
