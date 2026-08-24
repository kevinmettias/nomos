//! Why a routing selector fails validation, and how severely.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-022`: "The validator shall diagnose selectors that reference unknown
/// or unavailable judgment-implementation IDs or logical-operation IDs, rule/check
/// profiles inherited by no registered model-assisted check, and profiles whose
/// target, stage, task class, or capability predicates match no operation in the
/// resolved inventory."
///
/// Three named conditions, each transcribed directly from the "shall diagnose X, Y,
/// and Z" clause -- no proper noun is given for this type, the same naming pattern
/// [`crate::EffortLevel`] itself used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectorValidationFailure
{
    /// "selectors that reference unknown or unavailable judgment-implementation IDs
    /// or logical-operation IDs"
    UnknownOrUnavailableReference,
    /// "rule/check profiles inherited by no registered model-assisted check"
    UninheritedRuleCheckProfile,
    /// "profiles whose target, stage, task class, or capability predicates match no
    /// operation in the resolved inventory"
    NoMatchingOperation,
}

/// `MODEL-ROUTE-022`: "Missing optional packages may produce a conditional warning;
/// impossible references in the pinned configuration shall block publication or
/// activation according to policy." A clean two-way severity split.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectorValidationSeverity
{
    ConditionalWarning,
    Blocking,
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_FAILURES: [SelectorValidationFailure; 3] = [
        SelectorValidationFailure::UnknownOrUnavailableReference,
        SelectorValidationFailure::UninheritedRuleCheckProfile,
        SelectorValidationFailure::NoMatchingOperation,
    ];

    #[test]
    fn Test_The_Three_Failures_Are_Distinct()
    {
        for (index, failure) in ALL_FAILURES.iter().enumerate()
        {
            for (other_index, other) in ALL_FAILURES.iter().enumerate()
            {
                if index != other_index
                {
                    assert_ne!(failure, other);
                }
            }
        }
    }

    #[test]
    fn Test_Severity_Has_Two_Distinct_States()
    {
        assert_ne!(SelectorValidationSeverity::ConditionalWarning, SelectorValidationSeverity::Blocking);
    }
}
