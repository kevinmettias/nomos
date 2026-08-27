//! [`VerdictResponse`], carried only by [`super::profile_outcome::ProfileOutcomeResponse`].

use nomos_spec_orchestration::Verdict;
use nomos_spec_project::Freshness;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_orchestration::Verdict`], which does not derive
/// `Serialize`. `Verdict::Compared`'s own `Result<Freshness, ProjectError>` is split into two
/// variants here rather than nested, so a wire caller can match on `kind` alone.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VerdictResponse
{
    /// Neither the body nor its sidecar is on disk.
    Absent,
    /// A body is there and no sidecar beside it.
    Unstamped,
    /// A sidecar is there and no body beside it.
    Unbodied,
    /// Both halves are there, compared against what the store would produce now.
    Compared
    {
        /// Whether every comparison below found nothing to report.
        fresh: bool,
        stale: Option<(String, String)>,
        edited: Option<(String, String)>,
        diverged: Option<(String, String)>,
    },
    /// Rebuilding the profile to compare against failed.
    BuildFailed
    {
        /// What went wrong, as `ProjectError`'s own `Display` renders it.
        cause: String,
    },
}

impl VerdictResponse
{
    pub(crate) fn From(verdict: Verdict) -> Self
    {
        return match verdict
        {
            Verdict::Absent => Self::Absent,
            Verdict::Unstamped => Self::Unstamped,
            Verdict::Unbodied => Self::Unbodied,
            Verdict::Compared(Ok(freshness)) => Self::From_Freshness(&freshness),
            Verdict::Compared(Err(error)) => Self::BuildFailed { cause: error.to_string() },
        };
    }

    fn From_Freshness(freshness: &Freshness) -> Self
    {
        return Self::Compared {
            fresh: freshness.Is_Fresh(),
            stale: freshness.stale.clone(),
            edited: freshness.edited.clone(),
            diverged: freshness.diverged.clone(),
        };
    }
}
