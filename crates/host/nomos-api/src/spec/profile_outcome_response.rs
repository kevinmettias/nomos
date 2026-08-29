//! [`ProfileOutcomeResponse`], carried only by [`super::spec_freshness_response::SpecFreshnessResponse`].

use nomos_spec_orchestration::ProfileOutcome;
use nomos_spec_project::Profile;
use serde::Serialize;

use super::VerdictResponse;

/// A serializable twin of [`nomos_spec_orchestration::ProfileOutcome`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct ProfileOutcomeResponse
{
    /// The profile examined, resolved and (if subject-addressed) already narrowed. Already
    /// `Serialize` -- reused directly.
    pub profile: Profile,
    pub verdict: VerdictResponse,
}

impl ProfileOutcomeResponse
{
    pub(crate) fn From(outcome: ProfileOutcome) -> Self
    {
        return Self { profile: outcome.profile, verdict: VerdictResponse::From(outcome.verdict) };
    }
}
