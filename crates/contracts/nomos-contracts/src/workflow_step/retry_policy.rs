//! Whether -- and how -- a step's execution may be retried after a failure.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// Whether -- and how -- a step's execution may be retried after a failure.
///
/// `Retry` carries `deduplication_token_required` rather than being a bare attempt
/// count, because `WF-012`'s own requirement is that retries shall not repeat a
/// non-idempotent effect without a compensation or a deduplication token -- a
/// dedup token is this field's own way of satisfying that clause, alongside
/// [`super::Compensation`]. [`super::WorkflowStep::Is_Coherent`] checks the two
/// together against the step's own `idempotent` and `has_side_effects` fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryPolicy
{
    /// A failed execution is never retried.
    NoRetry,
    /// A failed execution may be retried, up to `max_attempts` total attempts
    /// (including the first).
    Retry
    {
        /// The total number of attempts, including the first.
        max_attempts: NonZeroU32,
        /// Whether a caller must supply a fresh, per-attempt deduplication token so
        /// the target can recognize a retried attempt as the same execution rather
        /// than a second one.
        deduplication_token_required: bool,
    },
}

impl RetryPolicy
{
    /// Whether this policy permits any retry at all.
    #[must_use]
    pub const fn Is_Retryable(self) -> bool
    {
        return matches!(self, Self::Retry { .. });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_No_Retry_Should_Not_Be_Retryable()
    {
        assert!(!RetryPolicy::NoRetry.Is_Retryable());
    }

    #[test]
    fn Test_Is_Retryable_Should_Be_True_For_A_Retry_Policy()
    {
        let policy = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: false,
        };

        assert!(policy.Is_Retryable());
    }
}
