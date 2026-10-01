//! What `nomos profile` tells the shell.

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `1` is deliberately absent, because
/// this group judges nothing and so has no finding that could fail a build, and so is `6`
/// — a root that is a directory and holds nothing profiles as a tree with nothing in it,
/// which is a true answer and the one a person adopting this tool from an empty directory
/// needs to read. `crate::vacuity`'s own stance for this group carries that reasoning
/// where the next author is standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The profile was rendered, and any starter file that was asked for was written.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// A starter file was asked for and something is already at that path, so nothing was
    /// written. `4` is this binary's "a human has to resolve it": what a repository that
    /// already declares a policy should do with a starter one is not this verb's decision.
    Refused = 4,
    /// What this verb needed from the filesystem could not be used at all — the root is
    /// not a directory, so there was nothing to profile, or the starter file could not be
    /// written. `5` carries the meaning `check`'s `Unreadable` and `spec`'s `StoreError`
    /// already gave it: the thing this group depended on could not be put to use.
    Unusable = 5,
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
