//! A decision the submission needs that nobody has taken yet.

use crate::Severity;

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Is_Open_Should_Be_True_Until_A_Closing_Citation_Is_Recorded()
    {
        assert!(Gap().Is_Open());

        let closed = DecisionGap {
            closed_by: Some("OD-SPEC-008".to_owned()),
            ..Gap()
        };

        assert!(!closed.Is_Open());
    }

    fn Gap() -> DecisionGap
    {
        return DecisionGap {
            question: "which substrate is canonical".to_owned(),
            blocks: vec!["behaviour".to_owned()],
            severity: Severity::Blocking,
            closed_by: None,
        };
    }
}
