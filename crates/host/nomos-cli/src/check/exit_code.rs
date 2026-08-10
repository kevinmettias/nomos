//! What `nomos check` tells the shell.

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `3` and `4` are `work`'s claim
/// codes and are not reused here, and `5` and `6` carry the meanings `spec` gave them —
/// "could not be read at all" and "the answer is empty because something expected was
/// not there".
///
/// [`ExitCode::Vacuous`] is the one that earns its own code rather than folding into
/// `Ok`. A run that judged nothing and a run that judged everything and approved are the
/// two states this binary must never render the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The rules ran and nothing they found can fail a build.
    Ok = 0,
    /// At least one finding can fail a build.
    Violations = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The tree could not be read at all.
    Unreadable = 5,
    /// Nothing was judged: the walk found no source, or no fact was materialized for any
    /// of the source it found. Either way a clean result would mean nothing.
    Vacuous = 6,
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
