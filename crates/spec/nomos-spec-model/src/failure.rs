//! One rule one field failed, and what would satisfy it.

// What a failure is: the refusal that names it, how severe it is, and the decision gap
// it leaves behind.
mod decision_gap;
mod refusal;
mod severity;

pub use decision_gap::DecisionGap;
pub use refusal::Refusal;
pub use severity::Severity;

/// One rule one field failed, and what would satisfy it.
///
/// `OD-SPEC-010` requires a refusal to name the submission, every field that failed, the rule
/// each one failed and what would satisfy it. This is one of those, and a refusal carries all
/// of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure
{
    /// The field that failed, or the rule's subject when no single field owns it.
    pub field: String,
    /// The rule, named so it can be looked up rather than guessed at.
    pub rule: String,
    /// What would satisfy it.
    pub remedy: String,
}
