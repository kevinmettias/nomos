//! [`UndeclaredOutcomeResponse`], what a rule did with a value the repository never declared, as
//! [`super::undeclared_value_response::UndeclaredValueResponse`] carries it.

use nomos_rules::UndeclaredOutcome;
use serde::Serialize;

/// A serializable twin of [`nomos_rules::UndeclaredOutcome`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other one in
/// this module. Tagged by `outcome` and flattened into the value it describes, so a caller reads
/// `"outcome": "judged_against"` beside the `value` judged against, and an outcome with no value
/// carries no `value` key at all.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum UndeclaredOutcomeResponse
{
    /// The rule judged against a value this workspace substituted for the one nobody declared.
    JudgedAgainst
    {
        /// The substituted value, spelled as a repository would declare it.
        value: String,
    },
    /// The rule judged nothing, for want of a norm nobody declared.
    JudgedNothing,
    /// The rule judged nothing and reported the value undeclared in a finding of its own.
    ReportedUndeclared,
    /// Goals were declared and no ceiling, or a ceiling of zero: the rule judged the two-way audit
    /// and not the spread bound.
    SpreadBoundNotJudged,
}

impl UndeclaredOutcomeResponse
{
    /// The twin of `outcome`. The match is exhaustive, so a fifth outcome does not compile until
    /// a caller can be told it.
    pub(crate) fn From(outcome: &UndeclaredOutcome) -> Self
    {
        return match outcome
        {
            UndeclaredOutcome::JudgedAgainst { value } => Self::JudgedAgainst { value: value.clone() },
            UndeclaredOutcome::JudgedNothing => Self::JudgedNothing,
            UndeclaredOutcome::ReportedUndeclared => Self::ReportedUndeclared,
            UndeclaredOutcome::SpreadBoundNotJudged => Self::SpreadBoundNotJudged,
        };
    }
}
