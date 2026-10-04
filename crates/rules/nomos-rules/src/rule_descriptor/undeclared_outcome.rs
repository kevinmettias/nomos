//! [`UndeclaredOutcome`], what a rule did with a value the repository never declared.

/// What a rule did with a value the repository never declared, as `OD-RULES-011` version 3
/// decision 1 names the answers: the value it judged against, that it judged nothing, or that it
/// reported the value undeclared -- and a goal ceiling's own fourth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UndeclaredOutcome
{
    /// The rule judged against a value this workspace substituted for the one nobody declared.
    JudgedAgainst
    {
        /// The substituted value, spelled as a repository would declare it.
        value: String,
    },
    /// The rule judged nothing, for want of a norm nobody declared.
    JudgedNothing,
    /// The rule judged nothing and reported the value undeclared in a finding of its own, as an
    /// axis no default is defensible for does (`OD-RULES-035` decision 3).
    ReportedUndeclared,
    /// Goals were declared and no ceiling, or a ceiling of zero: the rule judged the two-way audit
    /// and not the spread bound.
    SpreadBoundNotJudged,
}
