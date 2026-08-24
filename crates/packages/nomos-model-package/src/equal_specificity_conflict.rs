//! When two equally-specific selectors disagree, and nothing may quietly pick one.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-023`: "Equal-scope and equal-specificity selectors that assign
/// incompatible values shall produce an `EqualSpecificityConflict` and prevent
/// resolution. The diagnostic shall enumerate conflicting profile IDs, fields,
/// selectors, authority, and candidate operations; clients shall not resolve the
/// conflict through declaration order, client preference, package load order,
/// provider ranking, or a hidden last-write-wins rule."
///
/// Five fields, directly naming the diagnostic's own "shall enumerate A, B, C, D, E"
/// clause -- the cleanest field-enumeration sentence in this vocabulary family.
/// Every field stays raw: no `ProfileId`, `ScopeSelector`, or resolution-authority
/// type exists anywhere in this workspace to resolve any of the five against.
/// `authority` here is deliberately not [`nomos_contracts::AuthorityClass`] -- that
/// answers who may invoke an operation, a different question from which policy's
/// precedence backed a conflicting assignment. The prohibition clause (declaration
/// order, client preference, etc. must not resolve it) is a behavioral constraint on
/// callers, not a field, and is not encoded here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EqualSpecificityConflict
{
    pub profile_ids: Vec<String>,
    pub fields: Vec<String>,
    pub selectors: Vec<String>,
    pub authority: Vec<String>,
    pub candidate_operations: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Conflict_Carries_Exactly_What_It_Was_Given()
    {
        let conflict = EqualSpecificityConflict {
            profile_ids: vec!["rule-profile-a".to_owned(), "rule-profile-b".to_owned()],
            fields: vec!["effort".to_owned()],
            selectors: vec!["check-naming-convention@check".to_owned()],
            authority: vec!["organization-policy".to_owned()],
            candidate_operations: vec!["judge-finding".to_owned()],
        };

        assert_eq!(conflict.profile_ids.len(), 2);
        assert_eq!(conflict.fields, ["effort"]);
        assert_eq!(conflict.candidate_operations, ["judge-finding"]);
    }
}
