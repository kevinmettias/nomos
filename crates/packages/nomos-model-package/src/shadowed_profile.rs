//! A profile that can never win a match, no matter what is asked.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-021`: "The validator shall diagnose profiles that are permanently
/// shadowed or unreachable under the normative scope-precedence, selector-specificity,
/// policy-cap, and merge rules. A `ShadowedProfile` diagnostic shall identify the
/// winning profile or policy that dominates every possible match, the affected fields,
/// and whether the shadowing is intentional, redundant, or erroneous. Declaration
/// order shall not be used to make an otherwise unreachable profile effective."
///
/// Three fields, directly naming the diagnostic's own "shall identify A, B, and C"
/// clause. `dominant`/`affected_fields` stay raw `String`/`Vec<String>`: no
/// `ProfileId` or `ProfileField` type exists anywhere in this workspace to resolve
/// against, the same raw-identity precedent [`crate::ModelSelector::BackendFamily`]
/// already set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowedProfile
{
    pub dominant: String,
    pub affected_fields: Vec<String>,
    pub classification: ShadowClassification,
}

/// `MODEL-ROUTE-021`'s own three-way classification of a shadowed profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShadowClassification
{
    Intentional,
    Redundant,
    Erroneous,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Shadowed_Profile_Carries_Exactly_What_It_Was_Given()
    {
        let shadowed = ShadowedProfile {
            dominant: "organization-default".to_owned(),
            affected_fields: vec!["effort".to_owned()],
            classification: ShadowClassification::Redundant,
        };

        assert_eq!(shadowed.dominant, "organization-default");
        assert_eq!(shadowed.affected_fields, ["effort"]);
        assert_eq!(shadowed.classification, ShadowClassification::Redundant);
    }
}
