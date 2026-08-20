//! One package's fact, as [`Materialize_Workspace`](super::Materialize_Workspace) produces it.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One package's fact, together with the subject it was filed under and the
/// repository-relative path that subject addresses.
///
/// The path is carried rather than left for a caller to recover from the payload, the same
/// reason `nomos_rules::SourceFile::path` is a field and not a decode: a caller building a
/// rule's subject list needs a reporting path without knowing this capability's payload
/// schema, and a caller that decoded the payload just to get the path it already computed
/// would be a second reader of a schema owned by `nomos_cap_dependency`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFact
{
    pub subject: SubjectId,
    /// Repository-relative, forward slashes — the same path `Subject_Of_Path` addressed to
    /// produce `subject`.
    pub path: String,
    pub fact: MaterializedFact,
}
