//! How a claim was come by.

use serde::{Deserialize, Serialize};

const AUTHORITATIVE_LABEL: &str = "Authoritative";
const VERIFIED_LABEL: &str = "Verified";
const OBSERVED_LABEL: &str = "Observed";
const DERIVED_LABEL: &str = "Derived";
const APPROXIMATE_LABEL: &str = "Approximate";
const PREDICTED_LABEL: &str = "Predicted";
const AGENT_JUDGED_LABEL: &str = "AgentJudged";
const HUMAN_ASSERTED_LABEL: &str = "HumanAsserted";

/// The provenance class of a claim.
///
/// Deliberately not reducible to one confidence number. A prediction at 0.9 and a
/// measurement at 0.9 are not interchangeable, and any scheme that renders them
/// identically will eventually be used to justify a decision that only one of them
/// supports.
///
/// The ordering is by strength, weakest first, so that combining evidence takes the
/// `min` rather than consulting a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceClass
{
    /// A model produced this and no tool corroborated it.
    ///
    /// The floor, and deliberately so. A claim arrives here whenever the evidence
    /// attached to it does not support a stronger class, and it cannot be promoted
    /// afterwards — promotion would have to happen somewhere that no longer holds the
    /// evidence, which is how "the agent said the tests pass" becomes "the tests pass".
    AgentJudged,
    /// A person asserted this. Weight it as you would weight that person.
    HumanAsserted,
    /// Modelled rather than measured, for a state that has not occurred.
    Predicted,
    /// Measured or computed with a method that trades accuracy for cost, within a
    /// documented tolerance.
    Approximate,
    /// Computed from other facts by a deterministic rule. No stronger than its inputs.
    Derived,
    /// Directly observed at runtime.
    Observed,
    /// Checked by a mechanism that would have detected the negation.
    Verified,
    /// Definitional — true because the system defines it so.
    Authoritative,
}

impl EvidenceClass
{
    /// The variant's stable `PascalCase` name, for display and diagnostics.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Authoritative => AUTHORITATIVE_LABEL,
            Self::Verified => VERIFIED_LABEL,
            Self::Observed => OBSERVED_LABEL,
            Self::Derived => DERIVED_LABEL,
            Self::Approximate => APPROXIMATE_LABEL,
            Self::Predicted => PREDICTED_LABEL,
            Self::AgentJudged => AGENT_JUDGED_LABEL,
            Self::HumanAsserted => HUMAN_ASSERTED_LABEL,
        };
    }

    /// Whether this class may be reported as a mechanical result.
    ///
    /// A model's judgment and a person's assertion are both legitimate evidence and
    /// neither is a check having passed. Presenting them as one is the failure mode
    /// this method exists to make expressible in code rather than in a review comment.
    #[must_use]
    pub const fn Is_Mechanical(self) -> bool
    {
        return matches!(
            self,
            Self::Authoritative | Self::Verified | Self::Observed | Self::Derived | Self::Approximate
        );
    }

    /// The class of a conclusion drawn from evidence of both classes.
    ///
    /// Always the weaker. A derivation over an agent's judgment is an agent's judgment
    /// with extra steps.
    #[must_use]
    pub fn Weaker_Of(self, other: Self) -> Self
    {
        return core::cmp::min(self, other);
    }
}

impl core::fmt::Display for EvidenceClass
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

    #[test]
    fn Test_Agent_And_Human_Claims_Should_Not_Be_Mechanical()
    {
        assert!(!EvidenceClass::AgentJudged.Is_Mechanical());
        assert!(!EvidenceClass::HumanAsserted.Is_Mechanical());
    }

    #[test]
    fn Test_Is_Mechanical_Should_Be_True_For_Measured_And_Derived_Classes()
    {
        assert!(EvidenceClass::Authoritative.Is_Mechanical());
        assert!(EvidenceClass::Verified.Is_Mechanical());
        assert!(EvidenceClass::Observed.Is_Mechanical());
        assert!(EvidenceClass::Derived.Is_Mechanical());
        assert!(EvidenceClass::Approximate.Is_Mechanical());
    }

    /// Combining must never manufacture strength. This is the property that stops a
    /// chain of derivations from laundering a guess into a measurement.
    #[test]
    fn Test_Weaker_Of_Should_Never_Exceed_The_Weaker_Input()
    {
        for left in All_Evidence_Classes()
        {
            for right in All_Evidence_Classes()
            {
                let combined = left.Weaker_Of(right);

                assert!(combined <= left);
                assert!(combined <= right);
                assert_eq!(combined, right.Weaker_Of(left));
            }
        }
    }

    #[test]
    fn Test_Agent_Judged_Should_Be_The_Floor()
    {
        for class in Non_Floor_Evidence_Classes()
        {
            assert_eq!(
                class.Weaker_Of(EvidenceClass::AgentJudged),
                EvidenceClass::AgentJudged
            );
        }
    }

    /// Every evidence class except the floor, [`EvidenceClass::AgentJudged`] itself.
    fn Non_Floor_Evidence_Classes() -> [EvidenceClass; 7]
    {
        return [
            EvidenceClass::HumanAsserted,
            EvidenceClass::Predicted,
            EvidenceClass::Approximate,
            EvidenceClass::Derived,
            EvidenceClass::Observed,
            EvidenceClass::Verified,
            EvidenceClass::Authoritative,
        ];
    }

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let mut labels: Vec<&str> = All_Evidence_Classes().iter().map(|class| return class.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two evidence classes share a wire spelling");
    }

    /// Every declared evidence class, once.
    fn All_Evidence_Classes() -> [EvidenceClass; 8]
    {
        return [
            EvidenceClass::AgentJudged,
            EvidenceClass::HumanAsserted,
            EvidenceClass::Predicted,
            EvidenceClass::Approximate,
            EvidenceClass::Derived,
            EvidenceClass::Observed,
            EvidenceClass::Verified,
            EvidenceClass::Authoritative,
        ];
    }
}
