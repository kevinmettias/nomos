//! What `nomos gate` tells the shell.

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `3` and `4` are `work`'s claim codes
/// and are not reused here. [`Violations`](ExitCode::Violations) and
/// [`Vacuous`](ExitCode::Vacuous) carry exactly the meanings `check`'s own `Violations` and
/// `Vacuous` already gave those numbers -- `run` judges a tree the same way `nomos check`
/// does, so a `run` that finds a blocking finding or judges nothing must not read
/// differently at the shell than a `check` that did. [`Contradictory`](ExitCode::Contradictory)
/// carries the meaning `check`'s `Unreadable` and `spec`'s `StoreError` already gave `5` --
/// the foundational thing this group depends on could not be assembled -- and now covers
/// three cases: this gate's own rule registry refusing its own composition (`plan` and
/// `run` alike), for `run` only, the tree beneath it being unreadable or the check
/// registry beneath *that* being self-contradictory, and, for `compare` only, a run whose
/// findings do not each carry an occurrence identity of their own. Each is "nothing here
/// was ever assembled enough to judge", the same claim one layer down each time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The plan was composed and reported, or `run` judged the tree and nothing it found
    /// can fail a build.
    Ok = 0,
    /// `run` found at least one finding that can fail a build.
    Violations = 1,
    /// The command line was wrong.
    Usage = 2,
    /// This gate's own rule composition is self-contradictory (reachable for `plan` and
    /// `run` alike, though not reachable today -- see `nomos_gate_orchestration::
    /// Registered`'s own doc), or `run`'s tree could not be read at all, or the check
    /// registry beneath a `run` was itself self-contradictory. Every one of these is
    /// `GateRunOutcome::Indeterminate` when it comes from `run`; `run` never carries this
    /// distinction any further than that shared exit code, the same discipline `check`'s
    /// and `spec`'s foundational-failure codes already hold to.
    ///
    /// `compare` reports it for a fourth cause: a run whose findings do not yield one
    /// occurrence identity each, so the two sides cannot be put side by side and the
    /// comparison refuses rather than silently dropping a finding. Neither an
    /// `Indeterminate` nor from `run` -- both sides were judged -- but the same claim as
    /// the three above, one layer further in: what `compare` needed in order to answer was
    /// never assembled.
    Contradictory = 5,
    /// `run` found no source under the tree, or no fact was materialized for any of it, so
    /// nothing was judged. Also `GateRunOutcome::Indeterminate`; a clean result here would
    /// mean only that the walk or the analysis found nothing, the same lie `check`'s own
    /// `Vacuous` already refuses to render as `Ok`.
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

/// Every code this group can leave the process with.
///
/// Deliberately not `ExitCode::All()`: an inherent, zero-argument `All()` is exactly the
/// shape `nomos-rules::Universes_In` recognizes as a declared universe, and promoting one
/// costs a row in `tests/contract/tests/completeness_universes/table.rs` -- a file outside
/// this item's own territory, the same reason `check::ExitCode`'s own census once stayed a
/// private array under this same name before `OD-GATE-004` paid that cost on purpose.
/// Mirrored by `Test_Every_Exit_Code_Should_Be_Matched_Exhaustively` in `gate/tests.rs`.
#[cfg(test)]
pub(super) const fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::Violations,
        ExitCode::Usage,
        ExitCode::Contradictory,
        ExitCode::Vacuous,
    ];
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `Every_Exit_Code`'s own count and order, independent of `gate/tests.rs`'s
    /// exhaustiveness match over it (a different file, so a different Unit for coverage
    /// purposes).
    #[test]
    fn Test_Every_Exit_Code_Should_Name_Every_Declared_Variant_In_Declaration_Order()
    {
        assert_eq!(
            Every_Exit_Code(),
            &[ExitCode::Ok, ExitCode::Violations, ExitCode::Usage, ExitCode::Contradictory, ExitCode::Vacuous]
        );
    }
}
