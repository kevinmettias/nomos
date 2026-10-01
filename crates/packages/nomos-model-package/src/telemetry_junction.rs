//! One observable step of context assembly, causally linked to the step before it.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-016`'s nine closed context-assembly stages: "Retrieval, filtering,
/// authority resolution, deduplication, compression, prompt or task-envelope
/// projection, request construction, model inference, and post-response validation
/// shall emit separate causally linked `TelemetryJunctions`."
///
/// Nine variants, in the corpus's own order -- the same test `OD-PACKAGE-011` v3
/// licenses.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage
{
    Retrieval,
    Filtering,
    AuthorityResolution,
    Deduplication,
    Compression,
    PromptOrTaskEnvelopeProjection,
    RequestConstruction,
    ModelInference,
    PostResponseValidation,
}

impl PipelineStage
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Retrieval => "Retrieval",
            Self::Filtering => "Filtering",
            Self::AuthorityResolution => "AuthorityResolution",
            Self::Deduplication => "Deduplication",
            Self::Compression => "Compression",
            Self::PromptOrTaskEnvelopeProjection => "PromptOrTaskEnvelopeProjection",
            Self::RequestConstruction => "RequestConstruction",
            Self::ModelInference => "ModelInference",
            Self::PostResponseValidation => "PostResponseValidation",
        };
    }
}

impl core::fmt::Display for PipelineStage
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// `MODEL-ROUTE-016`'s own second half: "...shall emit separate causally linked
/// TelemetryJunctions with resource, latency, cache, usage, and evidence status where
/// available."
///
/// `stage` names which of [`PipelineStage`]'s nine steps this junction is about.
/// `caused_by` is what makes a chain of junctions causally linked rather than merely
/// co-located -- the prior junction's own identity, absent for the first junction in a
/// chain. The five named observability fields are each `Option`, matching "where
/// available" directly: a junction that could not observe one is not the same claim as
/// a junction that observed it as empty.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryJunction
{
    pub stage: PipelineStage,
    pub caused_by: Option<String>,
    pub resource: Option<String>,
    pub latency: Option<String>,
    pub cache: Option<String>,
    pub usage: Option<String>,
    pub evidence_status: Option<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_STAGES: [PipelineStage; 9] = [
        PipelineStage::Retrieval,
        PipelineStage::Filtering,
        PipelineStage::AuthorityResolution,
        PipelineStage::Deduplication,
        PipelineStage::Compression,
        PipelineStage::PromptOrTaskEnvelopeProjection,
        PipelineStage::RequestConstruction,
        PipelineStage::ModelInference,
        PipelineStage::PostResponseValidation,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL_STAGES.iter().map(|stage| return stage.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two stages share a label");
    }

    #[test]
    fn Test_A_Junction_With_Nothing_Available_Reports_Every_Field_As_Absent()
    {
        let junction = TelemetryJunction {
            stage: PipelineStage::Retrieval,
            caused_by: None,
            resource: None,
            latency: None,
            cache: None,
            usage: None,
            evidence_status: None,
        };

        assert!(junction.caused_by.is_none());
        assert!(junction.latency.is_none());
    }

    #[test]
    fn Test_A_Chained_Junction_Names_What_Caused_It()
    {
        let first = TelemetryJunction {
            stage: PipelineStage::Retrieval,
            caused_by: None,
            resource: Some("query-4f9a1c3e".to_owned()),
            latency: Some("120ms".to_owned()),
            cache: Some("miss".to_owned()),
            usage: None,
            evidence_status: Some("sound".to_owned()),
        };
        let second = TelemetryJunction {
            stage: PipelineStage::Filtering,
            caused_by: Some("query-4f9a1c3e".to_owned()),
            resource: None,
            latency: Some("4ms".to_owned()),
            cache: None,
            usage: None,
            evidence_status: None,
        };

        assert_eq!(second.caused_by, first.resource);
        assert_ne!(first.stage, second.stage);
    }
}
