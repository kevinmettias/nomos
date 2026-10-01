//! When a selector reaches outside the executor's exposed family or entitlement.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-026`: "A subscription, chat, CLI, or IDE `AgentExecutorPackage` route
/// shall reference only model identities, families, effort controls, and switching
/// operations currently declared by that executor package for the pinned
/// environment. A selector naming a model outside the exposed family or entitlement
/// shall produce `BackendFamilyViolation` or unavailable-model diagnostics and shall
/// never be simulated through an unrecorded substitution."
///
/// `BackendFamilyViolation` is named directly; "unavailable-model diagnostics" is
/// descriptive rather than a proper noun but unambiguous, the same "shall produce X
/// or Y diagnostics" pattern `MODEL-ROUTE-027`'s
/// [`crate::ImpossibleTelemetryGuarantee`]/[`crate::ImpossibleReplayRequirement`]
/// already used. The first sentence's four-noun subject list
/// (subscription/chat/CLI/IDE) is deliberately not re-typed here as a route-kind
/// enum: it rests only on being a sentence's grammatical subject, not a closing
/// "shall be" clause, and [`crate::ExecutorExposure`] (`MODEL-ROUTE-007`) already
/// covers the same four environments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelSelectorViolation
{
    BackendFamilyViolation,
    UnavailableModel,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_The_Two_Violations_Are_Distinct()
    {
        assert_ne!(ModelSelectorViolation::BackendFamilyViolation, ModelSelectorViolation::UnavailableModel);
    }
}
