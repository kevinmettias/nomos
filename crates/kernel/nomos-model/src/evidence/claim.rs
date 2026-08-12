use nomos_contracts::{EvidenceClass, ProviderId};
use serde::{Deserialize, Serialize};

use super::EvidenceRef;

/// A claim together with how it was come by and who produced it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence
{
    /// How this was come by.
    pub class: EvidenceClass,
    /// Who produced it.
    pub producer: ProviderId,
    /// What supports it.
    pub supporting: Vec<EvidenceRef>,
}

impl Evidence
{
    /// Whether this evidence may be reported as a mechanical result.
    ///
    /// Delegates to [`EvidenceClass::Is_Mechanical`] rather than restating the rule, so
    /// there is one place that decides what counts as a machine having checked
    /// something.
    #[must_use]
    pub const fn Is_Mechanical(&self) -> bool
    {
        return self.class.Is_Mechanical();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Agent_Evidence_Should_Not_Be_Mechanical()
    {
        let agent_claim = Evidence {
            class: EvidenceClass::AgentJudged,
            producer: ProviderId::New("some-executor"),
            supporting: Vec::new(),
        };

        assert!(!agent_claim.Is_Mechanical());
    }
}
