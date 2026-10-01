use nomos_contracts::{Applicability, SubjectId};
use serde::{Deserialize, Serialize};

/// Something a run did not look at, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageGap
{
    /// What was not covered.
    pub subject: SubjectId,
    /// Why not.
    pub reason: Applicability,
}
