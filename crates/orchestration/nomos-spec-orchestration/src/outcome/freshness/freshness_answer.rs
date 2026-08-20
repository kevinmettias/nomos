//! What a `freshness` run examined, and what it was promised.

use crate::outcome::ProfileOutcome;

/// What a `freshness` run examined, and what it was promised.
#[derive(Debug)]
pub struct FreshnessAnswer
{
    /// Every profile this run looked at, in catalogue order.
    pub examined: Vec<ProfileOutcome>,
    /// The profiles this run was told must be present, by identifier.
    pub required: Vec<String>,
}
