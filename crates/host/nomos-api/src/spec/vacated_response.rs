//! [`VacatedResponse`], carried only by [`super::commit_response::CommitResponse`].

use nomos_spec_orchestration::Vacated;
use serde::Serialize;
use std::path::PathBuf;

use super::VacateOutcomeResponse;

/// A serializable twin of [`nomos_spec_orchestration::Vacated`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct VacatedResponse
{
    /// The path the rename moved the record away from.
    pub path: PathBuf,
    pub outcome: VacateOutcomeResponse,
}

impl VacatedResponse
{
    pub(crate) fn From(vacated: Vacated) -> Self
    {
        return Self { path: vacated.path, outcome: VacateOutcomeResponse::From(vacated.outcome) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::VacateOutcome;

    #[test]
    fn Test_From_Should_Copy_The_Path_And_Map_The_Nested_Outcome()
    {
        let expected_path = PathBuf::from("docs/records/old-name.md");
        let vacated = Vacated { path: expected_path.clone(), outcome: VacateOutcome::Removed };

        let response = VacatedResponse::From(vacated);

        assert_eq!(response.path, expected_path);
        assert!(matches!(response.outcome, VacateOutcomeResponse::Removed));
    }
}
