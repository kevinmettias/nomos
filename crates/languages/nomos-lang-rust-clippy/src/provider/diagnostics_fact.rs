//! One package's fact, as [`Materialize_Workspace`](super::Materialize_Workspace) produces it.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One package's fact, together with the subject it was filed under and the
/// repository-relative path that subject addresses — the same three-field shape
/// `nomos_lang_rust_cargo::PackageFact` carries, for the identical reason: a caller
/// building a rule's subject list needs a reporting path without knowing this
/// capability's own payload schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsFact
{
    pub subject: SubjectId,
    /// Repository-relative, forward slashes.
    pub path: String,
    pub fact: MaterializedFact,
}
