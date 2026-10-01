//! What a rule reading a policy axis does when the repository declares nothing for it.

/// What a rule reading an axis does when the repository declares nothing for it.
///
/// `OD-RULES-035` decided this is stated on the axis rather than inferred from a constant in the
/// rule's body, and that a meaning is added by the first axis that needs it rather than ahead of
/// it. Every axis that existed when its axis table was written is judged against a default, the
/// meaning every naming and limits rule already had; `CYCLOMATIC_COMPLEXITY_MAX` was the first to
/// need the second. The third meaning the record names -- the rule judges nothing -- has no axis
/// that needs it, so it is not here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Undeclared<Value: 'static>
{
    /// The rule judges against a stated default.
    JudgedAgainstDefault
    {
        /// The default for a rule reading the axis repository-wide, and for a rule reading it
        /// for a language `languages` does not list.
        repository: Value,
        /// A language's own default, for a rule reading the axis for that language -- the way
        /// Go's hard file-size trigger is lower than the repository-wide one.
        languages: &'static [(&'static str, Value)],
    },
    /// The rule judges nothing and reports that the axis is undeclared, because no value
    /// anybody could defend as a default exists: a limit there is no common practice for is a
    /// choice the repository makes, and one this crate made for it would be judged as though
    /// somebody had.
    ReportedAsUndeclared,
}
