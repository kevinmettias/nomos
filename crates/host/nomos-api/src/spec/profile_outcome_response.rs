//! [`ProfileOutcomeResponse`], carried only by [`super::freshness_response::FreshnessResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::Verdict;
    use nomos_spec_project::Format;

    #[test]
    fn Test_From_Should_Carry_The_Profile_And_Map_The_Nested_Verdict()
    {
        let profile = Profile {
            id: "domain-specification".to_owned(),
            title: "Domain Specification".to_owned(),
            format: Format::Markdown,
            output: "spec/domain-specification.md".to_owned(),
            sections: Vec::new(),
        };
        let outcome = ProfileOutcome { profile: profile.clone(), verdict: Verdict::Absent };

        let response = ProfileOutcomeResponse::From(outcome);

        assert_eq!(response.profile.id, profile.id);
        assert!(matches!(response.verdict, VerdictResponse::Absent));
    }
}
