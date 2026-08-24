//! What pins the exact inputs a model-assisted operation was assembled from.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-029`: "Every `ResolvedModelExecution` shall contain a
/// `ModelInputAssemblyIdentity` that pins the `TaskEnvelope` schema/version,
/// system-instruction projection version, rule-guidance fragment identities and
/// versions, composite skill/context-generator identity and version, retrieval query
/// identity, ordered result-set identity, filtering/deduplication/compression/ordering
/// strategy identities and versions, tokenizer or accounting implementation, tool
/// manifest/version set, structured-output schema hash, redaction/truncation
/// transforms, and final request-envelope hash."
///
/// Twelve fields, each a direct transcription of one named pinned component. Every
/// component stays a raw `String`/`Vec<String>` -- no resolved schema-version,
/// tokenizer, or tool-manifest type exists anywhere in this workspace to reuse, the
/// same opacity `crate::ModelSelector`'s own variants already chose for comparably
/// underspecified domains.
///
/// This type is the shape `MODEL-ROUTE-029` itself grounds. `ResolvedModelExecution`,
/// the record this type is a field of, is not built here: `030`, `031`, `032` and
/// `036` each reference further fields of it that this requirement alone does not
/// name, and typing a near-empty wrapper ahead of those would invent structure none
/// of them states yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInputAssemblyIdentity
{
    pub task_envelope_schema_version: String,
    pub system_instruction_projection_version: String,
    pub rule_guidance_fragment_identities_and_versions: Vec<String>,
    pub composite_skill_context_generator_identity_and_version: String,
    pub retrieval_query_identity: String,
    pub ordered_result_set_identity: String,
    pub filtering_strategy_identities_and_versions: Vec<String>,
    pub tokenizer_or_accounting_implementation: String,
    pub tool_manifest_version_set: Vec<String>,
    pub structured_output_schema_hash: String,
    pub redaction_truncation_transforms: Vec<String>,
    pub final_request_envelope_hash: String,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Identity_Carries_Exactly_What_It_Was_Given()
    {
        let identity = ModelInputAssemblyIdentity {
            task_envelope_schema_version: "nomos.agent.task_envelope.v1".to_owned(),
            system_instruction_projection_version: "v3".to_owned(),
            rule_guidance_fragment_identities_and_versions: vec!["check-naming-convention@2".to_owned()],
            composite_skill_context_generator_identity_and_version: "code-review-skill@5".to_owned(),
            retrieval_query_identity: "q-4f9a1c3e".to_owned(),
            ordered_result_set_identity: "r-8b2d5a70".to_owned(),
            filtering_strategy_identities_and_versions: vec!["dedupe-by-path@1".to_owned()],
            tokenizer_or_accounting_implementation: "cl100k_base".to_owned(),
            tool_manifest_version_set: vec!["nomos.cap.example.model_package_test_only@1".to_owned()],
            structured_output_schema_hash: "sha256:abc".to_owned(),
            redaction_truncation_transforms: vec!["truncate-to-200k-tokens".to_owned()],
            final_request_envelope_hash: "sha256:def".to_owned(),
        };

        assert_eq!(identity.task_envelope_schema_version, "nomos.agent.task_envelope.v1");
        assert_eq!(identity.rule_guidance_fragment_identities_and_versions.len(), 1);
        assert_eq!(identity.tool_manifest_version_set.len(), 1);
        assert_eq!(identity.final_request_envelope_hash, "sha256:def");
    }
}
