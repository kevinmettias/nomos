//! [`EvaluationError`], a failed `MSBuild` evaluation and why.

use crate::EvaluationFailure;

/// `MSBuild` gave no definition set: why, and in words a person can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluationError
{
    /// Which kind of failure, which decides how a rule reports it.
    pub failure: EvaluationFailure,
    /// What happened, naming the tool's own words where it gave any.
    pub reason: String,
}

impl core::fmt::Display for EvaluationError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

impl std::error::Error for EvaluationError
{}
