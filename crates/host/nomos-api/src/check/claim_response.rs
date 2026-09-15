//! [`ClaimResponse`], whether every rule a judged check asked could actually look.

use serde::Serialize;

/// A serializable twin of [`nomos_check_orchestration::Claim`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimResponse
{
    Complete,
    Incomplete,
}

impl ClaimResponse
{
    pub(crate) fn From(claim: nomos_check_orchestration::Claim) -> Self
    {
        return match claim
        {
            nomos_check_orchestration::Claim::Complete => Self::Complete,
            nomos_check_orchestration::Claim::Incomplete => Self::Incomplete,
        };
    }
}
