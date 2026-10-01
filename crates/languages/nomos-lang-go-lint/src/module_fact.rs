//! [`ModuleFact`], one Go module's lint fact.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One Go module's `nomos.cap.lint.diagnostics` fact, and where it belongs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleFact
{
    /// The module's directory -- the one holding its `go.mod` -- relative to the repository root
    /// with forward slashes, and empty for a module at the root itself.
    pub path: String,
    /// The subject the fact is filed under: that directory's own.
    pub subject: SubjectId,
    /// The fact.
    pub fact: MaterializedFact,
}
