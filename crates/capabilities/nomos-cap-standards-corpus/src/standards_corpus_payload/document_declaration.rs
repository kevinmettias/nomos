//! One document the reader found, and what it says it is.

use super::DeclarationKind;

/// One document the reader found in a declared corpus, and the kind it declares for itself.
///
/// This is the population row, and it exists so that "every document on disk is reported" is
/// a checkable equality over a set of paths rather than a count a reader could satisfy while
/// dropping one document and inventing another. A count alone cannot tell those apart.
///
/// `path` is repository-relative with forward slashes, so a corpus read on one platform and
/// compared on another names the same document.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentDeclaration
{
    /// The document's path, relative to the repository root that declared the corpus.
    pub path: String,
    /// What the document says it is.
    pub kind: DeclarationKind,
}
