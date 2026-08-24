//! Vocabulary shared with an external knowledge system across the process boundary
//! `ARC-ECOSYSTEM-001` names -- what role a piece of borrowed context plays, and what
//! Nomos must carry about it without owning its content.

use serde::{Deserialize, Serialize};

use crate::{BuildVariantId, KnowledgeReferenceId, SnapshotId};

const NORMATIVE_REPOSITORY_POLICY_LABEL: &str = "NormativeRepositoryPolicy";
const APPROVED_ARCHITECTURE_DECISION_LABEL: &str = "ApprovedArchitectureDecision";
const IMPLEMENTATION_DOCUMENTATION_LABEL: &str = "ImplementationDocumentation";
const CODE_OR_TEST_EVIDENCE_LABEL: &str = "CodeOrTestEvidence";
const RUNTIME_OR_BENCHMARK_OBSERVATION_LABEL: &str = "RuntimeOrBenchmarkObservation";
const EXTERNAL_REFERENCE_LABEL: &str = "ExternalReference";
const HISTORICAL_RATIONALE_LABEL: &str = "HistoricalRationale";
const AGENT_GENERATED_INTERPRETATION_LABEL: &str = "AgentGeneratedInterpretation";

/// `AGT-011`: "`KnowledgeSourceRole` values shall include at minimum
/// `NormativeRepositoryPolicy`, `ApprovedArchitectureDecision`,
/// `ImplementationDocumentation`, `CodeOrTestEvidence`, `RuntimeOrBenchmarkObservation`,
/// `ExternalReference`, `HistoricalRationale`, and `AgentGeneratedInterpretation`."
///
/// Declared in the corpus's own listed order: `AGT-012` ranks knowledge "according to
/// role" before other dimensions, and a derived [`Ord`] over this declaration order is
/// a direct transcription of that ranking for the role dimension alone -- see
/// [`KnowledgeSourceRole::May_Constrain`] for the one further distinction `AGT-012`
/// draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KnowledgeSourceRole
{
    NormativeRepositoryPolicy,
    ApprovedArchitectureDecision,
    ImplementationDocumentation,
    CodeOrTestEvidence,
    RuntimeOrBenchmarkObservation,
    ExternalReference,
    HistoricalRationale,
    AgentGeneratedInterpretation,
}

impl KnowledgeSourceRole
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::NormativeRepositoryPolicy => NORMATIVE_REPOSITORY_POLICY_LABEL,
            Self::ApprovedArchitectureDecision => APPROVED_ARCHITECTURE_DECISION_LABEL,
            Self::ImplementationDocumentation => IMPLEMENTATION_DOCUMENTATION_LABEL,
            Self::CodeOrTestEvidence => CODE_OR_TEST_EVIDENCE_LABEL,
            Self::RuntimeOrBenchmarkObservation => RUNTIME_OR_BENCHMARK_OBSERVATION_LABEL,
            Self::ExternalReference => EXTERNAL_REFERENCE_LABEL,
            Self::HistoricalRationale => HISTORICAL_RATIONALE_LABEL,
            Self::AgentGeneratedInterpretation => AGENT_GENERATED_INTERPRETATION_LABEL,
        };
    }

    /// `AGT-012`: "`NormativeRepositoryPolicy` and currently effective approved
    /// executable decisions may constrain work; lower-authority sources may explain,
    /// motivate, or support bounded claims but shall not silently override current
    /// contracts."
    #[must_use]
    pub const fn May_Constrain(self) -> bool
    {
        return matches!(self, Self::NormativeRepositoryPolicy);
    }
}

impl core::fmt::Display for KnowledgeSourceRole
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// `AGT-010`: "Every knowledge item included in a `TaskEnvelope`, `PrepareChangeContext`,
/// `ContextualizedFinding`, composite agent-guidance projection, or agent-facing
/// explanation shall carry a `KnowledgeSourceRole`, source identity/version, authority
/// scope, applicable snapshot or environment, freshness, provenance, and permitted-use
/// classification. Missing role or authority metadata shall be represented as unresolved
/// and shall not be treated as normative."
///
/// A citation, not the referenced content -- `provenance` and every identity field here
/// name what an external knowledge system said and how, the same bounded-projection
/// role [`KnowledgeReferenceId`] already carries, never the claim's substance itself.
/// `role` and `authority_scope` are `Option` specifically so a missing one can be
/// represented rather than defaulted; [`KnowledgeContextItem::Is_Unresolved`] is the
/// corpus's own "shall not be treated as normative" test.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeContextItem
{
    pub source: Option<KnowledgeReferenceId>,
    pub source_version: Option<String>,
    pub role: Option<KnowledgeSourceRole>,
    pub authority_scope: Option<String>,
    pub applicable_snapshot: Option<SnapshotId>,
    pub applicable_build_variant: Option<BuildVariantId>,
    pub freshness: Option<String>,
    pub provenance: String,
    pub permitted_use: Option<String>,
    pub contradiction_links: Vec<KnowledgeReferenceId>,
}

impl KnowledgeContextItem
{
    /// `AGT-010`: "Missing role or authority metadata shall be represented as
    /// unresolved and shall not be treated as normative."
    #[must_use]
    pub const fn Is_Unresolved(&self) -> bool
    {
        return self.role.is_none() || self.authority_scope.is_none();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [KnowledgeSourceRole; 8] = [
        KnowledgeSourceRole::NormativeRepositoryPolicy,
        KnowledgeSourceRole::ApprovedArchitectureDecision,
        KnowledgeSourceRole::ImplementationDocumentation,
        KnowledgeSourceRole::CodeOrTestEvidence,
        KnowledgeSourceRole::RuntimeOrBenchmarkObservation,
        KnowledgeSourceRole::ExternalReference,
        KnowledgeSourceRole::HistoricalRationale,
        KnowledgeSourceRole::AgentGeneratedInterpretation,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|role| return role.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two roles share a wire spelling");
    }

    #[test]
    fn Test_Only_Normative_Repository_Policy_May_Constrain()
    {
        assert!(KnowledgeSourceRole::NormativeRepositoryPolicy.May_Constrain());

        for role in ALL
        {
            if role != KnowledgeSourceRole::NormativeRepositoryPolicy
            {
                assert!(!role.May_Constrain(), "{role} should not be able to constrain work");
            }
        }
    }

    #[test]
    fn Test_A_Missing_Role_Or_Authority_Scope_Should_Be_Unresolved()
    {
        let missing_role = KnowledgeContextItem {
            source: None,
            source_version: None,
            role: None,
            authority_scope: Some("repository".to_owned()),
            applicable_snapshot: None,
            applicable_build_variant: None,
            freshness: None,
            provenance: "test".to_owned(),
            permitted_use: None,
            contradiction_links: vec![],
        };
        assert!(missing_role.Is_Unresolved());

        let missing_authority = KnowledgeContextItem {
            role: Some(KnowledgeSourceRole::CodeOrTestEvidence),
            authority_scope: None,
            ..missing_role.clone()
        };
        assert!(missing_authority.Is_Unresolved());

        let resolved = KnowledgeContextItem {
            role: Some(KnowledgeSourceRole::CodeOrTestEvidence),
            authority_scope: Some("repository".to_owned()),
            ..missing_role
        };
        assert!(!resolved.Is_Unresolved());
    }
}
