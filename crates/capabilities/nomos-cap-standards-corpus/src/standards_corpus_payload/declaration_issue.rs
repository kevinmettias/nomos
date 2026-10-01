//! A document whose declaration could not be read.

/// A document whose own declaration could not be read, and why.
///
/// This is the one thing in the payload that is a defect rather than a fact, and it exists
/// because the alternative is the failure mode the whole capability is built against: a
/// document that will not parse must not be dropped from the population silently. A reader
/// that skipped it would report a smaller corpus and look clean.
///
/// It is carried as data here rather than raised as an error because one unreadable document
/// is not a reason to stop reading the other 1,499 — the same reasoning that made an empty
/// `enforced_by` a finding rather than a panic in the corpus's own reference implementation,
/// where a single malformed document used to take the whole gate down and leave every *other*
/// document unjudged.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclarationIssue
{
    /// The document whose declaration could not be read, repository-relative.
    pub path: String,
    /// What was wrong, naming the line or the field rather than only the document.
    pub reason: String,
}
