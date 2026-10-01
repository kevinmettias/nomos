//! The scope tiers a model-execution profile resolves through, in precedence order.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-005`'s eleven-tier scope-precedence order: "Configuration resolution
/// shall use one normative scope-precedence order for every client, daemon, worker,
/// backend, and executor: Invocation override; exact logical-operation profile; exact
/// judgment-implementation profile; rule/check profile; phase profile; gate profile;
/// workflow profile; task-class profile; repository profile; organization profile;
/// backend default."
///
/// Eleven variants, in the corpus's own order -- the same `RustEdition`-shaped test
/// `OD-PACKAGE-011` v3 licenses: the requirement closes the enumeration itself, nothing
/// here is invented past what it states. This type does not resolve anything -- it
/// names the ranking a resolver would walk, the same declared-not-computed boundary
/// `crate::EffortLevel` already draws. It also does not build `ResolvedModelExecution`,
/// the record `MODEL-ROUTE-005`'s own second sentence says preserves "each candidate,
/// match reason, scope rank, selected value, inherited value, rejected override,
/// authority, and rejection reason": `crate::ModelInputAssemblyIdentity`'s own doc
/// comment already established that `ResolvedModelExecution` stays unbuilt until every
/// requirement naming one of its fields is accounted for (`029`, `030`, `031`, `032`,
/// `036` each name more of it), and this scope-rank field is one more piece contributed
/// to that eventual assembly, not a license to type a partial wrapper now.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionScope
{
    InvocationOverride,
    ExactLogicalOperationProfile,
    ExactJudgmentImplementationProfile,
    RuleCheckProfile,
    PhaseProfile,
    GateProfile,
    WorkflowProfile,
    TaskClassProfile,
    RepositoryProfile,
    OrganizationProfile,
    BackendDefault,
}

impl ExecutionScope
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::InvocationOverride => "InvocationOverride",
            Self::ExactLogicalOperationProfile => "ExactLogicalOperationProfile",
            Self::ExactJudgmentImplementationProfile => "ExactJudgmentImplementationProfile",
            Self::RuleCheckProfile => "RuleCheckProfile",
            Self::PhaseProfile => "PhaseProfile",
            Self::GateProfile => "GateProfile",
            Self::WorkflowProfile => "WorkflowProfile",
            Self::TaskClassProfile => "TaskClassProfile",
            Self::RepositoryProfile => "RepositoryProfile",
            Self::OrganizationProfile => "OrganizationProfile",
            Self::BackendDefault => "BackendDefault",
        };
    }
}

impl core::fmt::Display for ExecutionScope
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

    const ALL: [ExecutionScope; 11] = [
        ExecutionScope::InvocationOverride,
        ExecutionScope::ExactLogicalOperationProfile,
        ExecutionScope::ExactJudgmentImplementationProfile,
        ExecutionScope::RuleCheckProfile,
        ExecutionScope::PhaseProfile,
        ExecutionScope::GateProfile,
        ExecutionScope::WorkflowProfile,
        ExecutionScope::TaskClassProfile,
        ExecutionScope::RepositoryProfile,
        ExecutionScope::OrganizationProfile,
        ExecutionScope::BackendDefault,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|scope| return scope.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two scopes share a label");
    }
}
