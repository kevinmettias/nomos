//! How a process ended, including the ways that are not an exit code.

/// How a process ended.
///
/// Not an exit code and a bool. A process that was killed for exceeding its timeout has
/// not failed its predicate — nobody found out whether the predicate holds — and
/// reporting that as a non-zero exit would turn "we did not learn anything" into "the
/// check failed", which is the same conflation this whole system exists to avoid one
/// level up.
///
/// [`ExitOutcome::TimedOut`] and [`ExitOutcome::Stalled`] are both that non-verdict, and
/// they are kept apart because their remedies differ. A `TimedOut` process was still
/// producing output when the wall bound expired — the honest case, whose only remedy is
/// patience or a bigger budget. A `Stalled` one produced nothing for the whole of its
/// idle bound while the wall bound still had time left — the case whose remedy is to
/// find what stopped reading it, not to wait longer for a process that has already gone
/// quiet. Collapsing them into one value is what let a stalled child and a slow-but-honest
/// one read identically; see `docs/records/OD-PLATFORM-001` for the cost of that.
///
/// Every variant here stays [`Copy`]. A caller that already destructures this type by
/// value and falls through to read the same place again — `nomos-ledger` does exactly
/// that when a run produces no verdict — depends on that; a variant carrying owned data
/// would silently break that call site, which is not this crate's territory to fix.
/// `idle_elapsed` on [`ExitOutcome::Stalled`] is therefore a [`std::time::Duration`]
/// rather than a snapshot of the output itself. The output itself is not discarded: it
/// is still on [`super::ProcessOutput`], captured up to the moment of the kill exactly as
/// it is for every other outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitOutcome
{
    /// The process ran to completion with this exit code.
    Exited
    {
        /// The exit code.
        code: i32,
    },
    /// The process was still producing output when the wall bound expired.
    TimedOut,
    /// The process produced no new output for the idle bound, before the wall bound
    /// expired. The wall bound and the idle bound coincide unless a caller asked for a
    /// shorter idle bound with [`super::Command::With_Idle_Timeout`], so this is only
    /// reachable when they did.
    Stalled
    {
        /// How long the process went without producing anything before it was judged
        /// stalled and killed.
        idle_elapsed: std::time::Duration,
    },
    /// The process was killed by a signal or otherwise ended abnormally.
    Terminated,
}

impl ExitOutcome
{
    /// Whether the process ran to completion and reported success.
    ///
    /// [`ExitOutcome::TimedOut`] and [`ExitOutcome::Terminated`] are not successes and
    /// are not failures of what was being checked. They are the absence of a result.
    #[must_use]
    pub const fn Is_Successful(self) -> bool
    {
        return matches!(self, Self::Exited { code: 0 });
    }

    /// Whether anything was actually learned about the thing being checked.
    #[must_use]
    pub const fn Has_A_Verdict(self) -> bool
    {
        return matches!(self, Self::Exited { .. });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// An arbitrary idle duration for the `Stalled` fixtures below -- its only role is to
    /// be nonzero and shared, so the two tests that build a stalled outcome agree on it.
    const ARBITRARY_IDLE_ELAPSED_SECS: u64 = 30;

    #[test]
    fn Test_Only_A_Zero_Exit_Should_Succeed()
    {
        assert!(ExitOutcome::Exited { code: 0 }.Is_Successful());
        assert!(!ExitOutcome::Exited { code: 1 }.Is_Successful());
        assert!(!ExitOutcome::TimedOut.Is_Successful());
        assert!(!ExitOutcome::Terminated.Is_Successful());
        assert!(!ExitOutcome::Stalled {
            idle_elapsed: std::time::Duration::from_secs(1)
        }
        .Is_Successful());
    }

    /// The distinction the enum exists for. A timeout must not be recorded as the
    /// predicate having been checked and found false.
    #[test]
    fn Test_A_Timeout_Should_Not_Count_As_A_Verdict()
    {
        assert!(!ExitOutcome::TimedOut.Has_A_Verdict());
        assert!(!ExitOutcome::Terminated.Has_A_Verdict());
        assert!(ExitOutcome::Exited { code: 1 }.Has_A_Verdict());
    }

    /// A stall is no more a verdict than a wall-bound timeout is, and it must stay that
    /// way for the same reason: nobody found out whether the predicate holds.
    #[test]
    fn Test_A_Stall_Should_Not_Count_As_A_Verdict_Either()
    {
        let stalled = ExitOutcome::Stalled {
            idle_elapsed: std::time::Duration::from_secs(ARBITRARY_IDLE_ELAPSED_SECS),
        };

        assert!(!stalled.Has_A_Verdict());
        assert!(!stalled.Is_Successful());
    }

    /// The two non-verdicts this type exists to keep apart must actually be different
    /// values, or `work show` has nothing to tell them apart with.
    #[test]
    fn Test_A_Stall_And_A_Timeout_Should_Be_Different_Values()
    {
        let stalled = ExitOutcome::Stalled {
            idle_elapsed: std::time::Duration::from_secs(ARBITRARY_IDLE_ELAPSED_SECS),
        };

        assert_ne!(stalled, ExitOutcome::TimedOut);
    }
}
