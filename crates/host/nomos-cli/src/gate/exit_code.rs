//! What `nomos gate` tells the shell.

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `3` and `4` are `work`'s claim codes
/// and are not reused here. `5` carries the meaning `check`'s `Unreadable` and `spec`'s
/// `StoreError` already gave it -- the foundational thing this group depends on could not
/// be assembled -- because [`Contradictory`](ExitCode::Contradictory) is the same shape one
/// layer down: this gate's own rule registry refused its own composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The plan was composed and reported.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// This gate's own rule composition is self-contradictory. Not reachable today -- see
    /// `nomos_gate_orchestration::Registered`'s own doc -- but a real code all the same,
    /// the same discipline `check`'s and `spec`'s foundational-failure codes already hold to.
    Contradictory = 5,
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

/// Every code this group can leave the process with.
///
/// Deliberately not `ExitCode::All()`: an inherent, zero-argument `All()` is exactly the
/// shape `nomos-rules::Universes_In` recognizes as a declared universe, and promoting one
/// costs a row in `tests/contract/tests/completeness_universes/table.rs` -- a file outside
/// this item's own territory, the same reason `check::ExitCode`'s own census once stayed a
/// private array under this same name before `OD-GATE-004` paid that cost on purpose.
/// Mirrored by `Test_Every_ExitCode_Should_Be_Matched_Exhaustively` in `gate/tests.rs`.
#[cfg(test)]
pub(super) const fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[ExitCode::Ok, ExitCode::Usage, ExitCode::Contradictory];
}
