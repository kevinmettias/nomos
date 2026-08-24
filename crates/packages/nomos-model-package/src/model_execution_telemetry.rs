//! What a model-assisted operation actually did, as one telemetry record.

use crate::EffortLevel;
use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-010`: "Telemetry shall record requested and effective profiles at
/// every model-assisted check, fix, and workflow step, including backend/executor
/// package, model identity, effort, parameters, inherited settings, overrides,
/// rejected candidates, selection rationale, usage, cost, latency, retries, and
/// fallback lineage."
///
/// Thirteen fields, each a direct transcription of one named part of the sentence.
/// "Requested and effective profiles" is read as two separately constructed instances
/// of this same flat record rather than a wrapper naming both -- the same
/// no-invented-nesting choice `crate::ModelInputAssemblyIdentity`'s own doc comment
/// already made for a comparably flat corpus sentence. `effort` reuses
/// [`crate::EffortLevel`]; every other field stays a raw `String`/`Vec<String>`/`u32`,
/// the same opacity this crate's other flat records already choose for underspecified
/// domains.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelExecutionTelemetry
{
    pub backend_executor_package: String,
    pub model_identity: String,
    pub effort: EffortLevel,
    pub parameters: Vec<String>,
    pub inherited_settings: Vec<String>,
    pub overrides: Vec<String>,
    pub rejected_candidates: Vec<String>,
    pub selection_rationale: String,
    pub usage: String,
    pub cost: String,
    pub latency: String,
    pub retries: u32,
    pub fallback_lineage: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Sample() -> ModelExecutionTelemetry
    {
        return ModelExecutionTelemetry {
            backend_executor_package: "acme-model-backend/2.1.0".to_owned(),
            model_identity: "acme-large-2024-06".to_owned(),
            effort: EffortLevel::High,
            parameters: vec!["temperature=0".to_owned()],
            inherited_settings: vec!["organization-default-effort".to_owned()],
            overrides: vec!["invocation --effort high".to_owned()],
            rejected_candidates: vec!["acme-small".to_owned()],
            selection_rationale: "highest effort candidate the backend exposes".to_owned(),
            usage: "1200 input tokens, 340 output tokens".to_owned(),
            cost: "0.014 USD".to_owned(),
            latency: "2.3s".to_owned(),
            retries: 0,
            fallback_lineage: Vec::new(),
        };
    }

    #[test]
    fn Test_A_Record_Carries_Exactly_What_It_Was_Given()
    {
        let requested = Sample();
        let mut effective = Sample();
        effective.model_identity = "acme-large-2024-09".to_owned();
        effective.retries = 1;

        assert_ne!(requested, effective, "requested and effective are two distinct instances");
        assert_eq!(effective.retries, 1);
        assert_eq!(requested.effort, EffortLevel::High);
    }
}
