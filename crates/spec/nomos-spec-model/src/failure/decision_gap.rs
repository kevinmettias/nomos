//! A decision the submission needs that nobody has taken yet.

use crate::failure::Severity;

/// A decision the submission needs that nobody has taken yet.
///
/// A first-class row rather than a note in prose, because `OD-SPEC-008` says a feature
/// request names the decisions it needs and a note cannot block acceptance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionGap
{
    /// What has to be decided.
    pub question: String,
    /// The fields it blocks.
    pub blocks: Vec<String>,
    /// Whether it prevents acceptance.
    pub severity: Severity,
    /// The citation that closed it, if one has.
    ///
    /// A citation and not a boolean: `OD-SPEC-010` says a gap is never closed by supplying
    /// the value it blocks, and only a citation can name what answered the question.
    pub closed_by: Option<String>,
}

impl DecisionGap
{
    /// Whether this gap is still open.
    #[must_use]
    pub const fn Is_Open(&self) -> bool
    {
        return self.closed_by.is_none();
    }
}
