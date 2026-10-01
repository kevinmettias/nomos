//! What `nomos guard` tells the shell, and so what git does with the commit or push.

/// What the process exits with. The numbers mean what they mean on every group of this binary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// Nothing refused, the repository is the party's own, or the scripts were written.
    Clean = 0,
    /// The policy refuses something; git stops the commit or push.
    Refused = 1,
    /// The command line was wrong.
    Usage = 2,
    /// No policy was named, it could not be read or acted on, or git could not be asked. At a
    /// transition this stops git too: a guard that cannot judge is not a guard that found
    /// nothing (`OD-POLICY-002` decision 4).
    Unusable = 5,
    /// `scan` judged nothing -- no policy at all, or no text file to read -- which is kept
    /// apart from a clean audit (`OD-ANALYSIS-012`).
    NothingJudged = 6,
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
