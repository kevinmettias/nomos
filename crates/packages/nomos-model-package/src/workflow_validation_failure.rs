//! Why a workflow's model-routing configuration would fail validation before execution.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-009`'s seven closed validation-failure categories: "Workflow validation
/// shall detect unavailable models, incompatible effort levels, prohibited backend
/// combinations, unobservable usage/cost requirements, context-limit violations,
/// unsupported structured-output guarantees, and missing fallback routes before
/// execution where possible."
///
/// Seven variants, in the corpus's own order -- the same `RustEdition`-shaped test
/// `OD-PACKAGE-011` v3 licenses. This type does not detect anything -- it names what a
/// validation step would report, the same declared-not-computed boundary
/// `crate::EffortLevel` already draws. "Before execution where possible" is a timing
/// constraint on a validator this workspace does not have, not a fact this enum states
/// about itself.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowValidationFailure
{
    UnavailableModel,
    IncompatibleEffortLevel,
    ProhibitedBackendCombination,
    UnobservableUsageOrCostRequirement,
    ContextLimitViolation,
    UnsupportedStructuredOutputGuarantee,
    MissingFallbackRoute,
}

impl WorkflowValidationFailure
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::UnavailableModel => "UnavailableModel",
            Self::IncompatibleEffortLevel => "IncompatibleEffortLevel",
            Self::ProhibitedBackendCombination => "ProhibitedBackendCombination",
            Self::UnobservableUsageOrCostRequirement => "UnobservableUsageOrCostRequirement",
            Self::ContextLimitViolation => "ContextLimitViolation",
            Self::UnsupportedStructuredOutputGuarantee => "UnsupportedStructuredOutputGuarantee",
            Self::MissingFallbackRoute => "MissingFallbackRoute",
        };
    }
}

impl core::fmt::Display for WorkflowValidationFailure
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

    const ALL: [WorkflowValidationFailure; 7] = [
        WorkflowValidationFailure::UnavailableModel,
        WorkflowValidationFailure::IncompatibleEffortLevel,
        WorkflowValidationFailure::ProhibitedBackendCombination,
        WorkflowValidationFailure::UnobservableUsageOrCostRequirement,
        WorkflowValidationFailure::ContextLimitViolation,
        WorkflowValidationFailure::UnsupportedStructuredOutputGuarantee,
        WorkflowValidationFailure::MissingFallbackRoute,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|failure| return failure.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two failures share a label");
    }
}
