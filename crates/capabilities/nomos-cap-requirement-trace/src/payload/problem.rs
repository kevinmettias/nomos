//! One already-judged disagreement between a committed assessment and the workspace it was
//! checked against.

use super::ProblemKind;

/// One already-judged disagreement between a committed assessment and the workspace (or the
/// assessment's own completeness) it was checked against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem
{
    /// Which of the five checks this is.
    pub kind: ProblemKind,
    /// The requirement the stale assessment names -- the file stem, so a rule can point a
    /// `Finding` at `tests/contract/requirements/{requirement}.assessment`.
    pub requirement: String,
    /// The full, human-readable explanation -- composed here, by the provider that has the
    /// filesystem access to tell a missing file from a renamed symbol, not by the rule.
    pub message: String,
}
