//! What one subscription, chat, CLI, or IDE `AgentExecutorPackage` actually exposes.

use serde::{Deserialize, Serialize};

use crate::EffortLevel;

/// `MODEL-ROUTE-007`: "`AgentExecutorPackage`s for subscription, chat, CLI, or IDE
/// assistants shall expose only the model identities, families, effort controls, and
/// switching operations actually supported by that environment. A workflow using such
/// an executor may mix models within that exposed family but shall not claim access
/// to models outside it."
///
/// Four named fields, the same enumerable-list pattern that already licensed
/// [`crate::ModelSelection`] for `MODEL-ROUTE-037`'s opening clause. `model_identities`
/// and `model_families` stay raw strings -- the same choice
/// [`crate::ModelSelector::BackendFamily`]/[`crate::ModelSelection`] already made for
/// the identical domain. `effort_controls` reuses the already-built, closed
/// [`EffortLevel`] enum directly. `switching_operations` has no named vocabulary
/// anywhere in the corpus, so it stays raw too. Kept standalone rather than wired into
/// [`crate::ModelRoutePackage`]'s manifest -- the second sentence's mixing/claiming
/// rule is a validation constraint over this data, not an additional field, and is not
/// enforced here.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorExposure
{
    pub model_identities: Vec<String>,
    pub model_families: Vec<String>,
    pub effort_controls: Vec<EffortLevel>,
    pub switching_operations: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Exposure_Carries_Exactly_What_It_Was_Given()
    {
        let exposure = ExecutorExposure {
            model_identities: vec!["claude-sonnet-5".to_owned()],
            model_families: vec!["claude".to_owned()],
            effort_controls: vec![EffortLevel::Low, EffortLevel::High],
            switching_operations: vec!["/fast".to_owned()],
        };

        assert_eq!(exposure.model_identities, ["claude-sonnet-5"]);
        assert_eq!(exposure.effort_controls, [EffortLevel::Low, EffortLevel::High]);
        assert_eq!(exposure.switching_operations, ["/fast"]);
    }
}
