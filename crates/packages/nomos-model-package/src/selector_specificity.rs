//! How specifically a selector matched, when more than one matches at the same scope.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-018`'s closed six-level specificity ranking: "When more than one
/// selector matches an operation at the same effective scope, the resolver shall rank
/// selector specificity deterministically in this order: exact judgment-implementation
/// ID; exact logical-operation ID; exact rule/check identity plus operation stage;
/// rule-family selector plus operation stage where present; task-class selector;
/// inherited default."
///
/// Six variants, in the corpus's own order -- the same test `OD-PACKAGE-011` v3
/// licenses. This type does not rank anything -- it names the order a resolver would
/// walk, the same declared-not-computed boundary `crate::ExecutionScope` already draws
/// for the coarser scope tier this ranking breaks ties within. "A selector with
/// additional compatible target constraints may refine a match but shall not bypass
/// this ordering" and the equal-rank conflict rule are both a resolver's obligations,
/// not facts this enum states about itself.
#[allow(clippy::doc_markdown)] // the corpus statement is quoted verbatim, not code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectorSpecificity
{
    ExactJudgmentImplementationId,
    ExactLogicalOperationId,
    ExactRuleCheckIdentityPlusStage,
    RuleFamilySelectorPlusStage,
    TaskClassSelector,
    InheritedDefault,
}

impl SelectorSpecificity
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::ExactJudgmentImplementationId => "ExactJudgmentImplementationId",
            Self::ExactLogicalOperationId => "ExactLogicalOperationId",
            Self::ExactRuleCheckIdentityPlusStage => "ExactRuleCheckIdentityPlusStage",
            Self::RuleFamilySelectorPlusStage => "RuleFamilySelectorPlusStage",
            Self::TaskClassSelector => "TaskClassSelector",
            Self::InheritedDefault => "InheritedDefault",
        };
    }
}

impl core::fmt::Display for SelectorSpecificity
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

    const ALL: [SelectorSpecificity; 6] = [
        SelectorSpecificity::ExactJudgmentImplementationId,
        SelectorSpecificity::ExactLogicalOperationId,
        SelectorSpecificity::ExactRuleCheckIdentityPlusStage,
        SelectorSpecificity::RuleFamilySelectorPlusStage,
        SelectorSpecificity::TaskClassSelector,
        SelectorSpecificity::InheritedDefault,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|rank| return rank.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two ranks share a label");
    }
}
