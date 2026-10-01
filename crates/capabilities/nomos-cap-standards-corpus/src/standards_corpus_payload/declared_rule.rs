//! One document that declares itself a rule, and the whole of what it declares.

use super::RuleSeverity;
use nomos_contracts::GateCategory;

/// One document that declares itself a rule, and the whole of what it declares about itself.
///
/// Every field is read from the document's own front matter. Nothing here is inferred from
/// the prose, and nothing is defaulted: a rule whose declaration is incomplete is an issue
/// rather than a row with a field this workspace supplied.
///
/// # Why the gate is a [`GateCategory`] and not a fourth vocabulary
///
/// The corpus's declaration and this workspace's report already name the identical four
/// states — `blocking`, `advisory`, `unreachable`, `review` — and [`GateCategory`]'s own doc
/// defines each one the way the corpus's schema does, down to `Review` being *"the correct
/// declaration for anything a machine cannot judge."* Two vocabularies that agree this exactly
/// are one vocabulary, and a second type would be the place the two drifted apart.
///
/// That identity is what "the gate a document declares reaches the verdict unchanged" means
/// structurally: there is no mapping step to flatten it, because there is no second type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredRule
{
    /// The document's path, repository-relative with forward slashes.
    pub path: String,
    /// The rule's stable identifier, which the corpus's schema requires to equal the file's
    /// own stem, so a rule's identity and its location cannot drift apart.
    pub id: String,
    /// The rule's canonical name.
    pub title: String,
    /// The severity the document declares.
    pub severity: RuleSeverity,
    /// The gate category the document declares.
    pub gate: GateCategory,
    /// The mechanical owners the document names. The corpus's schema requires at least one
    /// entry, so an empty list is an [issue](super::DeclarationIssue) rather than a row: a rule
    /// whose document names no owner is a rule nothing enforces, and recording it here as
    /// enforced by nobody would be the reader supplying a declaration the document did not
    /// write. The schema's own guidance to an *author* — write `["review"]` when no tool enforces
    /// this — is the author's act, not this reader's.
    pub enforced_by: Vec<String>,
}
