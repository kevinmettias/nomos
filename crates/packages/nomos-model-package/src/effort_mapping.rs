//! What actually happened when a canonical effort request met a specific backend.

use crate::EffortLevel;
use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-015`'s closed four-value mapping quality: "mapping quality of Exact,
/// Approximate, Unsupported, or ExecutorControlled."
///
/// Transcribed directly, the same test `OD-PACKAGE-011` v3 licenses. "Approximate
/// mappings shall remain distinguishable in UI, evidence, telemetry, and replay" is a
/// downstream consumer's obligation, not a fact this enum states about itself.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MappingQuality
{
    Exact,
    Approximate,
    Unsupported,
    ExecutorControlled,
}

impl MappingQuality
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Exact => "Exact",
            Self::Approximate => "Approximate",
            Self::Unsupported => "Unsupported",
            Self::ExecutorControlled => "ExecutorControlled",
        };
    }
}

impl core::fmt::Display for MappingQuality
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// `MODEL-ROUTE-015`: "Every resolved effort request shall emit an EffortMappingRecord
/// containing the canonical requested effort, provider-native setting or
/// executor-controlled disposition, mapping quality of Exact, Approximate,
/// Unsupported, or ExecutorControlled, package/version, rationale, and limitations."
///
/// Six fields, each a direct transcription of one named part of the sentence.
/// `requested_effort` reuses [`crate::EffortLevel`] rather than a raw string, since that
/// is exactly what `MODEL-ROUTE-004` already names it. Every other field stays a raw
/// `String`/`Vec<String>` -- no resolved provider-disposition, package-version, or
/// rationale type exists anywhere in this workspace to reuse, the same opacity
/// `crate::ModelInputAssemblyIdentity`'s own fields already chose for comparably
/// underspecified domains.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffortMappingRecord
{
    pub requested_effort: EffortLevel,
    pub provider_disposition: String,
    pub quality: MappingQuality,
    pub package_version: String,
    pub rationale: String,
    pub limitations: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_QUALITIES: [MappingQuality; 4] =
        [MappingQuality::Exact, MappingQuality::Approximate, MappingQuality::Unsupported, MappingQuality::ExecutorControlled];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL_QUALITIES.iter().map(|quality| return quality.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two qualities share a label");
    }

    #[test]
    fn Test_A_Record_Carries_Exactly_What_It_Was_Given()
    {
        let record = EffortMappingRecord {
            requested_effort: EffortLevel::High,
            provider_disposition: "native reasoning_effort=high".to_owned(),
            quality: MappingQuality::Exact,
            package_version: "acme-model-backend/2.1.0".to_owned(),
            rationale: "the provider exposes a matching native control".to_owned(),
            limitations: vec!["only available on the acme-large family".to_owned()],
        };

        assert_eq!(record.requested_effort, EffortLevel::High);
        assert_eq!(record.quality, MappingQuality::Exact);
        assert_eq!(record.limitations.len(), 1);
    }
}
