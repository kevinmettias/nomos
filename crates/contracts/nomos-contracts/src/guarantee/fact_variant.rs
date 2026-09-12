//! The resolution level a fact was established at, and nothing else about it.

use serde::{Deserialize, Serialize};

const SYNTACTIC_LABEL: &str = "Syntactic";
const SEMANTICALLY_RESOLVED_LABEL: &str = "SemanticallyResolved";
const RUNTIME_OBSERVED_LABEL: &str = "RuntimeObserved";
const APPROXIMATE_LABEL: &str = "Approximate";
const PREDICTED_LABEL: &str = "Predicted";

/// The resolution level at which a fact was established.
///
/// Ordered weakest first. A rule that needs name resolution cannot accept a syntactic
/// answer, and the comparison that decides so is this ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FactVariant
{
    /// Modelled rather than measured.
    Predicted,
    /// Established by a method that trades accuracy for cost.
    Approximate,
    /// Read from the text or its parse tree, with no name resolution.
    Syntactic,
    /// Established with resolved names, types and references.
    SemanticallyResolved,
    /// Established by observing execution.
    RuntimeObserved,
}

impl FactVariant
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Syntactic => SYNTACTIC_LABEL,
            Self::SemanticallyResolved => SEMANTICALLY_RESOLVED_LABEL,
            Self::RuntimeObserved => RUNTIME_OBSERVED_LABEL,
            Self::Approximate => APPROXIMATE_LABEL,
            Self::Predicted => PREDICTED_LABEL,
        };
    }
}

impl core::fmt::Display for FactVariant
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use alloc::vec::Vec;
    use super::*;

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [
            FactVariant::Predicted,
            FactVariant::Approximate,
            FactVariant::Syntactic,
            FactVariant::SemanticallyResolved,
            FactVariant::RuntimeObserved,
        ];

        let mut labels: Vec<&str> = all.iter().map(|variant| return variant.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two fact variants share a wire spelling");
    }
}
