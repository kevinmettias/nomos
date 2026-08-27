//! Vocabulary shared with an external knowledge system across the process boundary
//! `ARC-ECOSYSTEM-001` names -- what role a piece of borrowed context plays, and what
//! Nomos must carry about it without owning its content.

// A borrowed context item's own citation shape, kept in its own file.
mod context_item;

pub use context_item::KnowledgeContextItem;

use serde::{Deserialize, Serialize};

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
}
