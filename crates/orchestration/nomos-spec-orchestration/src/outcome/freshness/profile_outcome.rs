//! One profile a `freshness` run examined, and what it found.

use nomos_spec_project::Profile;

use crate::outcome::Verdict;

/// One profile a `freshness` run examined, and what it found.
#[derive(Debug)]
pub struct ProfileOutcome
{
    /// The profile examined, resolved and (if subject-addressed) already narrowed.
    pub profile: Profile,
    /// What was found for it.
    pub verdict: Verdict,
}
