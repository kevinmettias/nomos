//! [`ExaminedResponse`], the file and fact counts a judged check reports.

use serde::Serialize;

/// A serializable twin of [`nomos_check_orchestration::Examined`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExaminedResponse
{
    /// Files the walk read.
    pub files: usize,
    /// Files a syntax fact was materialized for.
    pub facts: usize,
}

impl ExaminedResponse
{
    pub(crate) fn From(examined: nomos_check_orchestration::Examined) -> Self
    {
        return Self { files: examined.files, facts: examined.facts };
    }
}
