//! [`FileFact`], one Go source's discarded-values fact.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One Go source's `nomos.cap.go.discarded_values` fact, and where it belongs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFact
{
    /// The source, relative to the repository root with forward slashes.
    pub path: String,
    /// The subject the fact is filed under: that path's own.
    pub subject: SubjectId,
    /// The fact.
    pub fact: MaterializedFact,
}
