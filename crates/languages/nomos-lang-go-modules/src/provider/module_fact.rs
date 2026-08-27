//! One module's fact, as [`Materialize_Workspace`](super::Materialize_Workspace) produces it.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One module's fact, together with the subject it was filed under and the
/// repository-relative path that subject addresses.
///
/// The same shape `nomos_lang_rust_cargo::PackageFact` carries, for the same reason: a
/// caller building a rule's subject list needs a reporting path without decoding this
/// capability's payload schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleFact
{
    pub subject: SubjectId,
    /// Repository-relative, forward slashes — the same path `Subject_Of_Path` addressed
    /// to produce `subject`.
    pub path: String,
    pub fact: MaterializedFact,
}
