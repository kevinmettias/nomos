//! Independent descriptive labels a correction candidate may carry.

/// `COR-010`: "Correction candidates shall be classified as mechanically safe,
/// behavior-preserving, policy-compliant, agent-proposed, interactive, architectural, or
/// speculative. These labels are independent of ranking."
///
/// Seven labels, and deliberately not exclusive with one another -- a candidate may
/// carry any subset, which is why [`crate::CorrectionCandidate::Labels`] returns a
/// slice rather than a single value. "Independent of ranking" is `COR-010`'s own
/// distinction from `COR-011`'s objective-weighted ranking: a label describes what a
/// candidate *is*, not how good it is, and the two are deliberately kept as different
/// concepts here rather than one field trying to answer both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CandidateLabel
{
    /// The edit is deterministic and judgment-free.
    MechanicallySafe,
    /// The edit provably does not change observable behavior.
    BehaviorPreserving,
    /// The edit conforms to every applicable repository or organization policy.
    PolicyCompliant,
    /// An agent generated the edit.
    AgentProposed,
    /// The edit came from direct, real-time interaction with an operator.
    Interactive,
    /// The edit changes the shape of a public interface, module boundary, or
    /// architectural decision, not only an implementation detail.
    Architectural,
    /// The edit is a hypothesis about a fix, not a verified one.
    Speculative,
}

impl CandidateLabel
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::MechanicallySafe => "mechanically_safe",
            Self::BehaviorPreserving => "behavior_preserving",
            Self::PolicyCompliant => "policy_compliant",
            Self::AgentProposed => "agent_proposed",
            Self::Interactive => "interactive",
            Self::Architectural => "architectural",
            Self::Speculative => "speculative",
        };
    }
}

impl core::fmt::Display for CandidateLabel
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

    const ALL: [CandidateLabel; 7] = [
        CandidateLabel::MechanicallySafe,
        CandidateLabel::BehaviorPreserving,
        CandidateLabel::PolicyCompliant,
        CandidateLabel::AgentProposed,
        CandidateLabel::Interactive,
        CandidateLabel::Architectural,
        CandidateLabel::Speculative,
    ];

    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|label| return label.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two labels share a wire spelling");
    }
}
