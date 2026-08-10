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
pub enum ExitCode
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
