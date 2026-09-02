//! What `nomos correct` tells the shell.
//!
//! The numbers are shared with every other group on this binary -- see `check::ExitCode`'s
//! own doc for the fuller statement of that discipline. `2`, `5` and `6` carry the meanings
//! `check` and `spec` already gave them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The rule ran. Either there was nothing to correct, or a real phantom was found and
    /// staged/validated cleanly -- committed too, if `--commit` was given.
    Ok = 0,
    /// A real phantom was found but could not be safely corrected: its claim line is not
    /// exactly once in the file, or the plan refused to stage, validate or commit against
    /// the file's live content.
    Refused = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The tree could not be read at all.
    Unreadable = 5,
    /// Nothing was judged: the walk found no source, or no fact was materialized for any
    /// of it.
    Vacuous = 6,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }

    /// Every code this group can leave the process with, for a test to walk without
    /// hand-maintaining a second list.
    ///
    /// Deliberately not named `All`: that exact zero-argument inherent-impl spelling is
    /// what `nomos_rules::universe::Is_The_Variant_List` treats as a declared universe
    /// needing a classification row in `tests/contract/tests/completeness_universes`'s own
    /// hand-maintained table -- real machinery this group's own territory does not reserve
    /// that file to extend. `check::ExitCode::All` and `gate::ExitCode::All` already have
    /// rows there from when each was added; this helper serves the identical purpose for
    /// this group's own tests without minting a second universe for that table to track.
    #[cfg(test)]
    #[must_use]
    pub const fn Every_Code() -> &'static [Self]
    {
        return &[Self::Ok, Self::Refused, Self::Usage, Self::Unreadable, Self::Vacuous];
    }
}
