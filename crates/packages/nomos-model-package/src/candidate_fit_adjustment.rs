//! Changes the engine may not make silently just to make a candidate fit.

use serde::{Deserialize, Serialize};

use crate::ModelInputAssemblyIdentity;

/// `MODEL-ROUTE-036`: "The engine shall not silently truncate context or evidence,
/// remove tools, weaken structured-output guarantees, change data region, lower
/// telemetry or replay guarantees, alter executor authority, or substitute a model to
/// make a candidate fit."
///
/// Seven named adjustments, in the corpus's own order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandidateFitAdjustment
{
    TruncateContextOrEvidence,
    RemoveTools,
    WeakenStructuredOutputGuarantees,
    ChangeDataRegion,
    LowerTelemetryOrReplayGuarantees,
    AlterExecutorAuthority,
    SubstituteModel,
}

/// `MODEL-ROUTE-036`: "Such changes require an explicit context policy, approved
/// degradation mode, or admissible fallback edge."
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeAuthorization
{
    ExplicitContextPolicy,
    ApprovedDegradationMode,
    AdmissibleFallbackEdge,
}

/// `MODEL-ROUTE-036`: "... and shall produce a new `ModelInputAssemblyIdentity` and
/// `ResolvedModelExecution`." Only `ModelInputAssemblyIdentity` (`MODEL-ROUTE-029`) is
/// carried here -- `ResolvedModelExecution` itself remains unbuilt in this workspace
/// (see `ModelInputAssemblyIdentity`'s own doc comment); this type does not invent it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedCandidateFitAdjustment
{
    pub adjustment: CandidateFitAdjustment,
    pub authorization: ChangeAuthorization,
    pub result: ModelInputAssemblyIdentity,
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Example_Identity() -> ModelInputAssemblyIdentity
    {
        return ModelInputAssemblyIdentity {
            task_envelope_schema_version: "nomos.agent.task_envelope.v1".to_owned(),
            system_instruction_projection_version: "v3".to_owned(),
            rule_guidance_fragment_identities_and_versions: vec![],
            composite_skill_context_generator_identity_and_version: "skill@1".to_owned(),
            retrieval_query_identity: "q-1".to_owned(),
            ordered_result_set_identity: "r-1".to_owned(),
            filtering_strategy_identities_and_versions: vec![],
            tokenizer_or_accounting_implementation: "cl100k_base".to_owned(),
            tool_manifest_version_set: vec![],
            structured_output_schema_hash: "sha256:aaa".to_owned(),
            redaction_truncation_transforms: vec![],
            final_request_envelope_hash: "sha256:bbb".to_owned(),
        };
    }

    #[test]
    fn Test_An_Authorized_Adjustment_Names_A_Real_Authorization_And_Result()
    {
        let authorized = AuthorizedCandidateFitAdjustment {
            adjustment: CandidateFitAdjustment::TruncateContextOrEvidence,
            authorization: ChangeAuthorization::ApprovedDegradationMode,
            result: Example_Identity(),
        };

        assert_eq!(authorized.adjustment, CandidateFitAdjustment::TruncateContextOrEvidence);
        assert_eq!(authorized.authorization, ChangeAuthorization::ApprovedDegradationMode);
    }
}
