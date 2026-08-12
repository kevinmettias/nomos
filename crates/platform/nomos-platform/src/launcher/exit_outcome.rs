//! How a process ended, including the ways that are not an exit code.

/// How a process ended.
///
/// Three outcomes, not an exit code and a bool. A process that was killed for exceeding
/// its timeout has not failed its predicate — nobody found out whether the predicate
/// holds — and reporting that as a non-zero exit would turn "we did not learn anything"
/// into "the check failed", which is the same conflation this whole system exists to
/// avoid one level up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitOutcome
{
    /// The process ran to completion with this exit code.
    Exited
    {
        /// The exit code.
        code: i32,
    },
    /// The process exceeded its timeout and was terminated.
    TimedOut,
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
    pub const fn Succeeded(self) -> bool
    {
        return matches!(self, Self::Exited { code: 0 });
    }

    /// Whether anything was actually learned about the thing being checked.
    #[must_use]
    pub const fn Produced_A_Verdict(self) -> bool
    {
        return matches!(self, Self::Exited { .. });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Only_A_Zero_Exit_Should_Succeed()
    {
        assert!(ExitOutcome::Exited { code: 0 }.Succeeded());
        assert!(!ExitOutcome::Exited { code: 1 }.Succeeded());
        assert!(!ExitOutcome::TimedOut.Succeeded());
        assert!(!ExitOutcome::Terminated.Succeeded());
    }

    /// The distinction the enum exists for. A timeout must not be recorded as the
    /// predicate having been checked and found false.
    #[test]
    fn Test_A_Timeout_Should_Not_Count_As_A_Verdict()
    {
        assert!(!ExitOutcome::TimedOut.Produced_A_Verdict());
        assert!(!ExitOutcome::Terminated.Produced_A_Verdict());
        assert!(ExitOutcome::Exited { code: 1 }.Produced_A_Verdict());
    }
}
